use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use aws_sdk_s3::{
    config::{self, Credentials, Region},
    error::SdkError,
    operation::head_bucket::HeadBucketError,
    Client,
};
use tracing::{info, warn};

/// Type alias for the S3-compatible storage client (RustFS / MinIO / AWS S3).
pub type StorageClient = Client;

/// Build an S3-compatible client from the given configuration.
///
/// `force_path_style` is always enabled so the client works with self-hosted
/// servers (RustFS, MinIO, GarageHQ) that use `http://host/bucket/key` URLs
/// instead of the AWS virtual-hosted style `http://bucket.host/key`.
pub fn get_storage_client(cfg: StorageConfiguration) -> Result<StorageClient, Error> {
    let credentials = Credentials::new(
        cfg.access_key.as_str(),
        cfg.secret_key.as_str(),
        None,
        None,
        "djmxcreation-backend",
    );

    let region = Region::new(cfg.region);

    let s3_config = config::Builder::new()
        .endpoint_url(cfg.endpoint.as_str())
        .region(region)
        .credentials_provider(credentials)
        .force_path_style(true) // required for self-hosted S3-compatible servers
        .build();

    Ok(Client::from_conf(s3_config))
}

/// Ensure the target bucket exists, creating it if necessary.
pub async fn ensure_bucket(bucket_name: &str, client: &StorageClient) -> Result<(), Error> {
    match client.head_bucket().bucket(bucket_name).send().await {
        Ok(_) => {
            info!(bucket = bucket_name, "storage bucket already exists");
            Ok(())
        }
        Err(SdkError::ServiceError(sdk_err)) => match sdk_err.err() {
            HeadBucketError::NotFound(_) => {
                client
                    .create_bucket()
                    .bucket(bucket_name)
                    .send()
                    .await
                    .map_err(|e| {
                        warn!(bucket = bucket_name, error = ?e, "failed to create storage bucket");
                        Error::BucketCreation
                    })?;
                info!(bucket = bucket_name, "storage bucket created");
                Ok(())
            }
            _ => {
                warn!(bucket = bucket_name, "unexpected error checking storage bucket");
                Err(Error::BucketCreation)
            }
        },
        Err(e) => {
            warn!(bucket = bucket_name, error = ?e, "error checking storage bucket");
            Err(Error::BucketCreation)
        }
    }
}
