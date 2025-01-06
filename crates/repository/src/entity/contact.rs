use app_core::dto::contact_dto::ContactDto;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;

#[derive(sqlx::FromRow, Serialize, Deserialize, Default, Debug, Clone)]
pub struct Contact {
    pub id: Option<i32>,
    pub description: Option<Json<Value>>,
}

impl Contact {
    pub fn new(id: Option<i32>, description: Option<Json<Value>>) -> Self {
        Self { id, description }
    }
}

impl From<Contact> for ContactDto {
    fn from(val: Contact) -> ContactDto {
        ContactDto::new(
            val.id,
            val.description.map(|description_json| description_json.0),
        )
    }
}
