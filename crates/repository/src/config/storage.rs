use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;
use tracing::{info, warn};

#[derive(Clone)]
pub struct StorageClient {
    pub credentials: Credentials,
    pub region: Region,
}

pub fn get_storage_client(cfg: StorageConfiguration) -> Result<StorageClient, Error> {
    let credentials = Credentials::new(
        Some(&cfg.access_key),
        Some(&cfg.secret_key),
        None,
        None,
        None,
    ).map_err(|_| Error::BucketCreation)?;

    let region = Region::Custom {
        region: cfg.region,
        endpoint: cfg.endpoint,
    };

    Ok(StorageClient {
        credentials,
        region,
    })
}

pub async fn ensure_bucket(bucket_name: &str, client: &StorageClient) -> Result<(), Error> {
    let mut bucket = Bucket::new(
        bucket_name,
        client.region.clone(),
        client.credentials.clone(),
    ).map_err(|e| {
        warn!(error = ?e, "error creating bucket config");
        Error::BucketCreation
    })?;

    bucket.set_path_style();

    // Check if it exists
    match bucket.head_object("/").await {
        Ok(_) | Err(s3::error::S3Error::HttpFailWithBody(404, _)) | Err(s3::error::S3Error::HttpFailWithBody(403, _)) => {
            // we will create the bucket if it's 404
            match Bucket::create_with_path_style(
                bucket_name,
                client.region.clone(),
                client.credentials.clone(),
                s3::BucketConfiguration::default(),
            ).await {
                Ok(_) => {
                    info!(bucket = bucket_name, "storage bucket created");
                    Ok(())
                }
                Err(s3::error::S3Error::HttpFailWithBody(409, _)) => {
                    info!(bucket = bucket_name, "storage bucket already exists");
                    Ok(())
                }
                Err(e) => {
                    warn!(error = ?e, "error checking storage bucket");
                    // maybe it already exists
                    Ok(())
                }
            }
        }
        Err(e) => {
            warn!(error = ?e, "error checking storage bucket");
            Ok(())
        }
    }
}
