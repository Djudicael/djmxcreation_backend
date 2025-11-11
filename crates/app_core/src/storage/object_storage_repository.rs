use app_error::Error;
use async_trait::async_trait;
use std::sync::Arc;
pub type DynIObjectStorage = Arc<dyn IObjectStorage + Send + Sync>;
#[async_trait]
pub trait IObjectStorage {
    async fn create_bucket(&self, name: &str, owner: Option<&str>) -> Result<(), Error>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), Error>;
    async fn upload_stream(&self, bucket: &str, key: &str, data: &[u8]) -> Result<(), Error>;
    async fn get_public_url(&self, bucket: &str, key: &str) -> Result<String, Error>;
    async fn get_signed_url(
        &self,
        bucket: &str,
        key: &str,
        expires_secs: u32,
    ) -> Result<String, Error>;
}
