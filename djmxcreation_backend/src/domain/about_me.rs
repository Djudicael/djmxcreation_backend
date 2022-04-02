use crate::domain::content::Content;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
#[derive(sqlx::FromRow, Serialize, Deserialize, Default, Debug, Clone)]
pub struct AboutMe {
    id: Option<i32>,
    first_name: String,
    last_name: String,
    description: Option<Json<Value>>,
    photo: Option<Json<Value>>,
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

    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn description(&self) ->  Option<&Json<Value>> {
        self.description.as_ref()
    }

    pub fn photo(&self) -> Option<&Json<Value>> {
        self.photo.as_ref()
    }
}
