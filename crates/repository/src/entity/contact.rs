use app_core::dto::contact_dto::ContactDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Contact {
    pub id: Option<Uuid>,
    pub description: Option<Value>,
}

impl Contact {
    pub fn new(id: Option<Uuid>, description: Option<Value>) -> Self {
        Self { id, description }
    }
}

impl From<Contact> for ContactDto {
    fn from(val: Contact) -> ContactDto {
        ContactDto::new(val.id, val.description)
    }
}
