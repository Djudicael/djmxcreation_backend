use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::dto::contact_dto::ContactDto;

pub type DynIContactRepository = Arc<dyn IContactRepository + Send + Sync>;

#[async_trait]
pub trait IContactRepository {
    async fn get_contact(&self) -> Result<ContactDto, Error>;
    async fn update_contact(&self, id: i32, contact: &ContactDto) -> Result<ContactDto, Error>;
}
