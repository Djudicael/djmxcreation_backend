use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

pub type DynIContactService = Arc<dyn IContactService + Send + Sync>;
use uuid::Uuid;

use crate::dto::contact_dto::ContactDto;

#[async_trait]
pub trait IContactService {
    async fn get_contact(&self) -> Result<ContactDto, Error>;
    async fn update_contact(&self, id: Uuid, contact: &ContactDto) -> Result<ContactDto, Error>;
}
