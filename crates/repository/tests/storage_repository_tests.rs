use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;
use repository::config::minio::StorageClient;
use repository::storage_repository::StorageRepository;
use s3::Bucket;
use s3::creds::Credentials;

use test_util::minio::init_minio;

#[tokio::test]
async fn test_storage_repository() -> Result<(), Error> {
    let (podman, minio_image) = init_minio().expect("Failed to initialize MinIO");

    let credentials = Credentials::new(
        Some(&minio_image.secret_access_key()),
        Some(&minio_image.access_key_id()),
        None,
        None,
        None,
    )
    .expect("Should create credentials");
    let region = minio_image.region().parse().expect("Should parse region");
    let container = podman
        .start(minio_image)
        .await
        .expect("Failed to run MinIO container");

    // Wait for MinIO to be ready
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let client = Bucket::new("portfolio", region, credentials).expect("Should create bucket");

    // Initialize StorageRepository
    let repository = StorageRepository::new(client);

    // Test upload_file
    let bucket_name = "test-bucket";
    let file_name = "test.txt";
    let file_content = b"Hello, MinIO!";

    repository
        .upload_file(bucket_name, file_name, file_content)
        .await?;

    // Test get_object_url
    let url = repository.get_object_url(bucket_name, file_name).await?;
    println!("Object URL: {}", url);
    assert!(url.contains(file_name));

    // Test remove_object
    repository.remove_object(bucket_name, file_name).await?;

    Ok(())
}
