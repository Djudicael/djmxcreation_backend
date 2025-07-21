use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use repository::config::minio::StorageClient;
use repository::storage_repository::StorageRepository;

use s3::Bucket;
use s3::creds::Credentials;
use test_util::minio::{MinioContainer, init_minio};

struct TestContext {
    repository: StorageRepository,
    bucket_name: &'static str,
    file_name: &'static str,
    file_content: &'static [u8],
    _container: Option<MinioContainer>,
}

impl TestContext {
    async fn new() -> Self {
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
        // Wait for MinIO to be ready (increase to 12s)
        tokio::time::sleep(std::time::Duration::from_secs(12)).await;

        // Get the endpoint (host:port) from the container
        let endpoint = container
            .endpoint()
            .await
            .expect("Failed to get MinIO endpoint");
        println!("Using MinIO endpoint: {}", endpoint);

        // Set up the S3 client with custom endpoint and path-style
        let mut client =
            Bucket::new("data", region, credentials.clone()).expect("Should create bucket");
        Bucket::new_with_path_style("data", region, credentials.clone())
            .expect("Should create bucket");

        client.set_base_url(endpoint.clone());
        client.set_path_style();

        // Try to create the bucket (ignore error if already exists)
        // let _ = client.create_bucket("data", region.clone()).await;

        let repository = StorageRepository::new(client);
        Self {
            repository,
            bucket_name: "data",
            file_name: "test.txt",
            file_content: b"Hello, MinIO!",
            _container: Some(container),
        }
    }
}

#[tokio::test]
async fn test_storage_repository_crud() {
    let ctx = TestContext::new().await;

    // Upload file
    let upload_result = ctx
        .repository
        .upload_file(ctx.bucket_name, ctx.file_name, ctx.file_content)
        .await;
    if let Err(ref e) = upload_result {
        eprintln!("Upload failed with error: {:?}", e);
    }
    assert!(upload_result.is_ok(), "Upload failed: {:?}", upload_result);

    // Get object URL
    let url = ctx
        .repository
        .get_object_url(ctx.bucket_name, ctx.file_name)
        .await
        .expect("Failed to get object URL");
    println!("Object URL: {}", url);
    assert!(
        url.contains(ctx.file_name),
        "Object URL does not contain file name: {}",
        url
    );

    // Remove object
    let remove_result = ctx
        .repository
        .remove_object(ctx.bucket_name, ctx.file_name)
        .await;
    assert!(remove_result.is_ok(), "Remove failed: {:?}", remove_result);

    // Ensure object is gone
    let url_after_removal = ctx
        .repository
        .get_object_url(ctx.bucket_name, ctx.file_name)
        .await;
    assert!(
        url_after_removal.is_err(),
        "Object URL should not be available after removal"
    );
}
