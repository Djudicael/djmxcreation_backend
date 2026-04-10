use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;
use s3::Bucket;
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

    fn get_bucket(&self, bucket_name: &str) -> Result<Box<Bucket>, Error> {
        let mut bucket = Bucket::new(
            bucket_name,
            self.client.region.clone(),
            self.client.credentials.clone(),
        ).map_err(|e| {
            error!(error = ?e, "Failed to create bucket instance");
            Error::BucketCreation
        })?;
        bucket.set_path_style();
        Ok(bucket)
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
        let bucket = self.get_bucket(bucket_name)?;
        
        bucket.put_object(file_name, file).await.map_err(|e| {
            error!(bucket = bucket_name, key = file_name, error = ?e, "failed to upload file");
            Error::StorageUpload
        })?;

        Ok(())
    }

    async fn get_object_url(&self, bucket_name: &str, file_name: &str) -> Result<String, Error> {
        let bucket = self.get_bucket(bucket_name)?;
        
        // rust-s3 has `presign_get` which returns the generated URL as a String
        let presigned_url = bucket.presign_get(file_name, PRESIGN_EXPIRY_SECS, None).await.map_err(|e| {
            error!(bucket = bucket_name, key = file_name, error = ?e, "failed to generate presigned URL");
            Error::StorageGetObjectUrl
        })?;

        Ok(presigned_url)
    }

    async fn remove_object(&self, bucket_name: &str, file_name: &str) -> Result<(), Error> {
        let bucket = self.get_bucket(bucket_name)?;
        
        bucket.delete_object(file_name).await.map_err(|e| {
            error!(bucket = bucket_name, key = file_name, error = ?e, "failed to delete object");
            Error::StorageDeleteObject
        })?;
        
        Ok(())
    }
}
