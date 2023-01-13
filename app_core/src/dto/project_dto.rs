use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Default, Debug, Clone)]
pub struct ProjectDto {
    pub id: Option<i32>,
    pub metadata: Option<Value>,
    pub description: Option<Value>,
    pub visible: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
}

impl ProjectDto {
    pub fn new(
        id: Option<i32>,
        metadata: Option<Value>,
        description: Option<Value>,
        visible: bool,
        created_on: Option<DateTime<Utc>>,
        updated_on: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            metadata,
            description,
            visible,
            created_on,
            updated_on,
        }
    }
}
