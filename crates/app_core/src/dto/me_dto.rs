use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MeDto {
    pub id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub description: Option<Value>,
    pub photo: Option<Value>,
    pub photo_url: Option<String>,
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
}
