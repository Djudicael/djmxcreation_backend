use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::dto::about_me_dto::AboutMeDto;
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AboutMeView {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub picture: Option<String>,
}

impl AboutMeView {
    pub fn new(
        id: Option<Uuid>,
        first_name: String,
        last_name: String,
        description: Option<Value>,
        picture: Option<String>,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            description,
            picture,
        }
    }
}

impl From<AboutMeView> for AboutMeDto {
    fn from(val: AboutMeView) -> Self {
        AboutMeDto::new(None, val.first_name, val.last_name, val.description, None)
    }
}

impl From<AboutMeDto> for AboutMeView {
    fn from(dto: AboutMeDto) -> Self {
        Self::new(dto.id, dto.first_name, dto.last_name, dto.description, None)
    }
}
