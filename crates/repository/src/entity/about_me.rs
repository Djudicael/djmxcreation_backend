use app_core::dto::{about_me_dto::AboutMeDto, content_dto::ContentDto};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
#[derive(sqlx::FromRow, Serialize, Deserialize, Default, Debug, Clone)]
pub struct AboutMe {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Json<Value>>,
    pub photo: Option<Json<Value>>,
}

impl AboutMe {
    pub fn new(
        id: Option<i32>,
        first_name: String,
        last_name: String,
        description: Option<Json<Value>>,
        photo: Option<Json<Value>>,
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

impl From<AboutMe> for AboutMeDto {
    fn from(val: AboutMe) -> AboutMeDto {
        AboutMeDto::new(
            val.id,
            val.first_name,
            val.last_name,
            val.description.map(|description_json| description_json.0),
            val.photo
                .map(|photo_json| photo_json.0)
                .and_then(to_content),
        )
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}
