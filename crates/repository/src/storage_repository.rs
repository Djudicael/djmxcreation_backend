use app_core::storage::storage_repository::IStorageRepository;
use app_error::Error;
use async_trait::async_trait;

use crate::config::minio::StorageClient;

pub struct StorageRepository {
    client: StorageClient,
}

impl StorageRepository {
    pub fn new(client: StorageClient) -> Self {
        Self { client }
    }

    // Helper method to get a bucket with a different name
    async fn get_bucket(&self, bucket_name: &str) -> Result<StorageClient, Error> {
        let credentials = self
            .client
            .credentials()
            .await
            .map_err(|_| Error::BucketCreation)?;
        Ok(
            s3::Bucket::new(bucket_name, self.client.region().clone(), credentials)
                .map_err(|_| Error::BucketCreation)?,
        )
    }
}

#[async_trait]
impl IStorageRepository for StorageRepository {
    async fn upload_file(
        &self,
        _bucket_name: &str,
        file_name: &str,
        file: &[u8],
    ) -> Result<(), Error> {
        self.client
            .put_object(file_name, file)
            .await
            .map_err(|_| Error::StorageUpload)?;
        Ok(())
    }
    async fn upload_file_in_public_bucket(
        &self,
        _bucket_name: &str,
        file_name: &str,
        file: &[u8],
    ) -> Result<(), Error> {
        self.client
            .put_object(file_name, file)
            .await
            .map_err(|_| Error::StorageUpload)?;
        Ok(())
    }

    async fn get_object_url_presigned(
        &self,
        _bucket_name: &str,
        file_name: &str,
    ) -> Result<String, Error> {
        let presigned_url = self
            .client
            .presign_get(file_name, 8640, None)
            .await
            .map_err(|_| Error::StorageGetObjectUrl)?;

        Ok(presigned_url)
    }
    async fn get_object_url(&self, _bucket_name: &str, file_name: &str) -> Result<String, Error> {
        let bucket_url = self.client.url();

        Ok(format!("{}/{}", bucket_url, file_name))
    }

    async fn remove_object(&self, _bucket_name: &str, file_name: &str) -> Result<(), Error> {
        self.client
            .delete_object(file_name)
            .await
            .map_err(|_| Error::StorageDeleteObject)?;
        Ok(())
    }
}
