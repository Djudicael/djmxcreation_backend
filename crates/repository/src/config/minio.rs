use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use s3::creds::Credentials;
use s3::{Bucket, Region};

pub type StorageClient = Box<Bucket>;

async fn create_bucket_admin_api(
    admin_url: &str,
    admin_token: &str,
    bucket_name: &str,
) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v0/bucket/{}", admin_url, bucket_name);
    let _ = client
        .post(&url)
        .bearer_auth(admin_token)
        .send()
        .await
        .map_err(|_| Error::BucketCreation);

    // 2. Set bucket ACL to public read
    let acl_url = format!(
        "{}/api/v0/bucket/{}/authorize/public",
        admin_url, bucket_name
    );
    let acl_body = serde_json::json!({
        "read": true,
        "write": false,
        "delete": false,
        "list": true
    });
    client
        .put(&acl_url)
        .bearer_auth(admin_token)
        .json(&acl_body)
        .send()
        .await
        .map_err(|_| Error::PublicBucketAcl);
    Ok(())
}

pub async fn get_storage_client(config: StorageConfiguration) -> Result<StorageClient, Error> {
    let bucket_name = "portfolio";
    // First create the bucket via Admin API
    create_bucket_admin_api(&config.admin_endpoint, &config.admin_token, bucket_name).await?;
    let region = Region::Custom {
        region: config.region,
        endpoint: config.endpoint,
    };

    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )
    .expect("Should create credentials");
    // let config = BucketConfiguration::public();

    let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone())
        .expect("Should create bucket")
        .with_path_style();
    // let exists = bucket.exists().await.unwrap_or(false);
    // if !exists {
    //     bucket = Bucket::create_with_path_style(bucket_name, region, credentials, config)
    //         .await
    //         .expect("Should create bucket")
    //         .bucket;
    // }
    Ok(bucket)
}
