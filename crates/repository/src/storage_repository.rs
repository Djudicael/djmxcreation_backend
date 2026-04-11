use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use tracing::error;

use crate::config::storage::StorageClient;

/// Presigned URL validity window (seconds).
const PRESIGN_EXPIRY_SECS: u32 = 8_640; // 2.4 hours

pub struct StorageRepository {
    client: StorageClient,
}

impl StorageRepository {
    pub fn new(client: StorageClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IStorageRepository for StorageRepository {
    async fn upload_file(
        &self,
        bucket_name: &str,
        file_name: &str,
        file: &[u8],
    ) -> Result<(), Error> {
        self.client
            .inner
            .put_object()
            .bucket(bucket_name)
            .key(file_name)
            .body(ByteStream::from(file.to_vec()))
            .send()
            .await
            .map_err(|e| {
                error!(bucket = bucket_name, key = file_name, error = ?e, "failed to upload file");
                Error::StorageUpload
            })?;

        Ok(())
    }

    async fn get_object_url(&self, bucket_name: &str, file_name: &str) -> Result<String, Error> {
        let expires_in = std::time::Duration::from_secs(PRESIGN_EXPIRY_SECS as u64);
        let presigning_config = PresigningConfig::expires_in(expires_in).map_err(|e| {
            error!(bucket = bucket_name, key = file_name, error = ?e, "failed to configure presigning");
            Error::StorageGetObjectUrl
        })?;

        let presigned = self.client
            .inner
            .get_object()
            .bucket(bucket_name)
            .key(file_name)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                error!(bucket = bucket_name, key = file_name, error = ?e, "failed to generate presigned URL");
                Error::StorageGetObjectUrl
            })?;

        Ok(presigned.uri().to_string())
    }

    async fn remove_object(&self, bucket_name: &str, file_name: &str) -> Result<(), Error> {
        self.client
            .inner
            .delete_object()
            .bucket(bucket_name)
            .key(file_name)
            .send()
            .await
            .map_err(|e| {
                error!(bucket = bucket_name, key = file_name, error = ?e, "failed to delete object");
                Error::StorageDeleteObject
            })?;

        Ok(())
    }
}
