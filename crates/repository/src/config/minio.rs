use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use s3::creds::Credentials;
use s3::{Bucket, BucketConfiguration, Region};

pub type StorageClient = Box<Bucket>;

pub async fn get_aws_client(config: StorageConfiguration) -> Result<StorageClient, Error> {
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
    let bucket_name = "portfolio";
    let config = BucketConfiguration::public();

    let mut bucket = Bucket::new(bucket_name, region.clone(), credentials.clone())
        .expect("Should create bucket")
        .with_path_style();
    let exists = bucket.exists().await.unwrap_or(false);
    if !exists {
        bucket = Bucket::create_with_path_style(bucket_name, region, credentials, config)
            .await
            .expect("Should create bucket")
            .bucket;
    }
    Ok(bucket)
}
