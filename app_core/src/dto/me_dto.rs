use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MeDto {
    id: Option<i32>,
    first_name: String,
    last_name: String,
    description: Option<Value>,
    photo: Option<Value>,
    photo_url: Option<String>,
}

impl MeDto {
    pub fn new(
        id: Option<i32>,
        first_name: String,
        last_name: String,
        description: Option<Value>,
        photo: Option<Value>,
        photo_url: Option<String>,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            description,
            photo,
            photo_url,
        }
    }

    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn description(&self) -> Option<&Value> {
        self.description.as_ref()
    }

    pub fn photo(&self) -> Option<&Value> {
        self.photo.as_ref()
    }

    pub fn photo_url(&self) -> Option<&String> {
        self.photo_url.as_ref()
    }
}
