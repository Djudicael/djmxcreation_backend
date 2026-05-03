use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub type DynIStorageRepository = Arc<dyn IStorageRepository + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type DynIStorageRepository = Arc<dyn IStorageRepository + Sync>;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
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
