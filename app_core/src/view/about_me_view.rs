use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::about_me_dto::AboutMeDto;
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AboutMeView {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub picture: Option<String>,
}

impl AboutMeView {
    pub fn new(
        id: Option<i32>,
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

impl Into<AboutMeDto> for AboutMeView {
    fn into(self) -> AboutMeDto {
        AboutMeDto::new(
            None,
            self.first_name,
            self.last_name,
            self.description,
            None,
        )
    }
}

impl From<AboutMeDto> for AboutMeView {
    fn from(dto: AboutMeDto) -> Self {
        Self::new(dto.id, dto.first_name, dto.last_name, dto.description, None)
    }
}
