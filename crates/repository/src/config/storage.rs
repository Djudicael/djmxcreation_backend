use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use tracing::{info, warn};

#[derive(Clone)]
pub struct StorageClient {
    pub inner: Client,
}

pub fn get_storage_client(cfg: StorageConfiguration) -> Result<StorageClient, Error> {
    let credentials = Credentials::new(&cfg.access_key, &cfg.secret_key, None, None, "Static");

    let mut builder = S3ConfigBuilder::new()
        .credentials_provider(credentials)
        .region(Region::new(cfg.region))
        .endpoint_url(cfg.endpoint)
        .force_path_style(true);

    #[cfg(target_os = "wasi")]
    {
        use crate::config::wasi_http_client::WasiHttpClient;
        builder = builder.http_client(WasiHttpClient::new());
    }

    let config = builder.build();
    let client = Client::from_conf(config);

    Ok(StorageClient { inner: client })
}

pub async fn ensure_bucket(bucket_name: &str, client: &StorageClient) -> Result<(), Error> {
    // Check if it exists
    match client.inner.head_bucket().bucket(bucket_name).send().await {
        Ok(_) => {
            info!(bucket = bucket_name, "storage bucket already exists");
            Ok(())
        }
        Err(_) => {
            // we will create the bucket if it's 404
            match client
                .inner
                .create_bucket()
                .bucket(bucket_name)
                .send()
                .await
            {
                Ok(_) => {
                    info!(bucket = bucket_name, "storage bucket created");
                    Ok(())
                }
                Err(err) => {
                    warn!(error = ?err, "error creating storage bucket");
                    // maybe it already exists
                    Ok(())
                }
            }
        }
    }
}
