use serde_json::Value;
use sqlx::types::{chrono, Json};

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct ProjectEntity {
    id: Option<i32>,
    metadata: Option<Json<Value>>,
    description: Option<Json<Value>>,
    visible: bool,
    created_on: Option<chrono::DateTime<chrono::Utc>>,
    updated_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProjectEntity {
    pub fn new(
        id: Option<i32>,
        metadata: Option<Json<Value>>,
        description: Option<Json<Value>>,
        visible: bool,
        created_on: Option<chrono::DateTime<chrono::Utc>>,
        updated_on: Option<chrono::DateTime<chrono::Utc>>,
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

    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn metadata(&self) -> Option<&Json<Value>> {
        self.metadata.as_ref()
    }

    pub fn description(&self) -> Option<&Json<Value>> {
        self.description.as_ref()
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn created_on(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.created_on.as_ref()
    }

    pub fn updated_on(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.updated_on.as_ref()
    }
}
