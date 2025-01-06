use app_core::{
    contact::{contact_repository::DynIContactRepository, contact_service::IContactService},
    dto::contact_dto::ContactDto,
};
use app_error::Error;
use async_trait::async_trait;

pub struct ContactService {
    pub contact_repository: DynIContactRepository,
}

impl ContactService {
    pub fn new(contact_repository: DynIContactRepository) -> Self {
        Self { contact_repository }
    }
}

#[async_trait]
impl IContactService for ContactService {
    async fn get_contact(&self) -> Result<ContactDto, Error> {
        let contact = self.contact_repository.get_contact().await?;
        Ok(contact)
    }

    async fn update_contact(&self, id: i32, contact: &ContactDto) -> Result<ContactDto, Error> {
        let _ = self.contact_repository.get_contact().await?;
        let contact = self.contact_repository.update_contact(id, contact).await?;
        Ok(contact)
    }
}
