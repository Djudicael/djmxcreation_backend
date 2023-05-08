use std::time::Duration;

use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;
use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};

use crate::config::minio::StorageClient;

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
        let body = ByteStream::from(file.to_owned());
        self.client
            .put_object()
            .bucket(bucket_name)
            .key(file_name)
            .body(body)
            .send()
            .await
            .map_err(|_| Error::StorageUpload)?;
        Ok(())
    }

    async fn get_object_url(&self, bucket_name: &str, file_name: &str) -> Result<String, Error> {
        let expires_in = Duration::from_secs(8640);
        let presigned_request = self
            .client
            .get_object()
            .bucket(bucket_name)
            .key(file_name)
            .presigned(PresigningConfig::expires_in(expires_in).unwrap())
            .await
            .map_err(|_| Error::StorageGetObjectUrl)?;

        Ok(presigned_request.uri().to_string())
    }

    async fn remove_object(&self, bucket_name: &str, file_name: &str) -> Result<(), Error> {
        self.client
            .delete_object()
            .bucket(bucket_name)
            .key(file_name)
            .send()
            .await
            .map_err(|_| Error::StorageDeleteObject)?;
        Ok(())
    }
}
