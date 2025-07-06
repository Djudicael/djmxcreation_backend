use crate::config::minio::StorageClient;
use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;
use repository::storage_repository::StorageRepository;
use rustainers::ExposedPort;
use rustainers::images::Minio;
use rustainers::runner::Runner;
use test_util::minio::init_minio;

#[tokio::test]
async fn test_storage_repository() -> Result<(), Error> {
    let (podman, minio_image) = init_minio().expect("Failed to initialize MinIO");
    let container = podman
        .start(image)
        .await
        .expect("Failed to run MinIO container");

    // Wait for MinIO to be ready
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Initialize StorageClient
    let client = StorageClient::new("http://127.0.0.1:9000", "minioadmin", "minioadmin")?;

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
    assert!(url.contains(file_name));

    // Test remove_object
    repository.remove_object(bucket_name, file_name).await?;

    // Clean up
    runner.stop(container).await?;

    Ok(())
}
