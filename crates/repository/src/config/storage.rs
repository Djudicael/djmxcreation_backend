use app_config::storage_configuration::StorageConfiguration;
use app_error::Error;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use tracing::{info, warn};

#[derive(Clone)]
pub struct StorageClient {
    pub inner: Client,
}

#[cfg(target_os = "wasi")]
#[derive(Debug)]
struct WasiSleep;

#[cfg(target_os = "wasi")]
impl aws_sdk_s3::config::AsyncSleep for WasiSleep {
    fn sleep(&self, duration: std::time::Duration) -> aws_sdk_s3::config::Sleep {
        aws_sdk_s3::config::Sleep::new(async move {
            wstd::task::sleep(duration.into()).await;
        })
    }
}

pub fn get_storage_client(cfg: StorageConfiguration) -> Result<StorageClient, Error> {
    let credentials = Credentials::new(&cfg.access_key, &cfg.secret_key, None, None, "Static");

    let region = Region::new(cfg.region);
    let endpoint = cfg.endpoint;

    #[cfg(target_os = "wasi")]
    {
        use crate::config::wasi_http_client::WasiHttpClient;
        let builder = S3ConfigBuilder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .sleep_impl(WasiSleep)
            .credentials_provider(credentials)
            .region(region)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .http_client(WasiHttpClient::new());
        let config = builder.build();
        let client = Client::from_conf(config);
        return Ok(StorageClient { inner: client });
    }

    #[cfg(not(target_os = "wasi"))]
    {
        let builder = S3ConfigBuilder::new()
            .credentials_provider(credentials)
            .region(region)
            .endpoint_url(endpoint)
            .force_path_style(true);
        let config = builder.build();
        let client = Client::from_conf(config);
        Ok(StorageClient { inner: client })
    }
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

/// Call from provisioning, never from a public request. Origins must be explicit.
pub async fn configure_media_cors(
    bucket: &str,
    client: &StorageClient,
    origins: Vec<String>,
) -> Result<(), Error> {
    use aws_sdk_s3::types::{CorsConfiguration, CorsRule};
    if origins.is_empty() {
        return Ok(());
    }
    let rule = CorsRule::builder()
        .set_allowed_origins(Some(origins))
        .allowed_methods("PUT")
        .allowed_methods("GET")
        .allowed_methods("HEAD")
        .allowed_headers("*")
        .expose_headers("ETag")
        .expose_headers("Accept-Ranges")
        .expose_headers("Content-Range")
        .max_age_seconds(3600)
        .build()
        .map_err(|_| Error::StorageUpload)?;
    let cors = CorsConfiguration::builder()
        .cors_rules(rule)
        .build()
        .map_err(|_| Error::StorageUpload)?;
    client
        .inner
        .put_bucket_cors()
        .bucket(bucket)
        .cors_configuration(cors)
        .send()
        .await
        .map_err(|_| Error::StorageUpload)?;
    Ok(())
}
