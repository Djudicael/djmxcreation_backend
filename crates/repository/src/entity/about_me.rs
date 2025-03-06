use app_core::dto::{about_me_dto::AboutMeDto, content_dto::ContentDto};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AboutMe {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub photo: Option<Value>,
}

impl AboutMe {
    pub fn new(
        id: Option<Uuid>,
        first_name: String,
        last_name: String,
        description: Option<Value>,
        photo: Option<Value>,
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
            val.description,
            val.photo.map(|photo_json| photo_json).and_then(to_content),
        )
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}
