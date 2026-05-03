use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub type DynIContactRepository = Arc<dyn IContactRepository + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub type DynIContactRepository = Arc<dyn IContactRepository + Sync>;
use uuid::Uuid;

use crate::dto::contact_dto::ContactDto;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait IContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error>;
    async fn update_contact(&self, id: Uuid, contact: &ContactDto) -> Result<ContactDto, Error>;
}
