use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::content_dto::ContentDto;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AboutMeDto {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub photo: Option<ContentDto>,
}

impl AboutMeDto {
    pub fn new(
        id: Option<Uuid>,
        first_name: String,
        last_name: String,
        description: Option<Value>,
        photo: Option<ContentDto>,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            description,
            photo,
        }
    }
}
