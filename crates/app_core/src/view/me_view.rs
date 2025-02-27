use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::dto::about_me_dto::AboutMeDto;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeView {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub photo_url: Option<String>,
}

impl MeView {
    pub fn new(
        id: Option<Uuid>,
        first_name: String,
        last_name: String,
        description: Option<Value>,

        photo_url: Option<String>,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            description,
            photo_url,
        }
    }
}

impl From<AboutMeDto> for MeView {
    fn from(dto: AboutMeDto) -> Self {
        Self::new(dto.id, dto.first_name, dto.last_name, dto.description, None)
    }
}
