use app_core::storage::storage_repository::IStorageRepository;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::Client;
use repository::config::storage::StorageClient;
use repository::storage_repository::StorageRepository;
use std::sync::Arc;
use test_util::rustfs::RustFS;
use test_util::shared_harness::shared_rustfs;

struct TestContext {
    repository: StorageRepository,
    bucket_name: &'static str,
    file_name: &'static str,
    file_content: &'static [u8],
}

impl TestContext {
    async fn new() -> Self {
        let bucket_name = "data";
        let endpoint = shared_rustfs().await;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let credentials = Credentials::new(
            "rustfsadmin",
            "rustfsadmin",
            None,
            None,
            "Static",
        );

        let config = S3ConfigBuilder::new()
            .credentials_provider(credentials)
            .region(Region::new("us-east-1"))
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(config);

        // Ensure bucket exists
        let _ = client.create_bucket().bucket(bucket_name).send().await;

        let storage_client = StorageClient { inner: client };
        let repository = StorageRepository::new(storage_client);
        Self {
            repository,
            bucket_name,
            file_name: "test.txt",
            file_content: b"Hello, RustFS!",
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
