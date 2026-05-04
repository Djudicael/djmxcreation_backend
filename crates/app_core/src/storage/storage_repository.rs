use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

pub type DynIStorageRepository = Arc<dyn IStorageRepository + Send + Sync>;

#[async_trait]
pub trait IStorageRepository {
    async fn upload_file(
        &self,
        bucket_name: &str,
        file_name: &str,
        file: &[u8],
    ) -> Result<(), Error>;
    async fn get_object_url(&self, bucket_name: &str, file_name: &str) -> Result<String, Error>;
    async fn remove_object(&self, bucket_name: &str, file_name: &str) -> Result<(), Error>;
}
