use serde_json::Value;
use sqlx::types::{chrono, Json};

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct ProjectContentEntity {
    id: Option<i32>,
    project_id: i32,
    content: Option<Json<Value>>,
    created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProjectContentEntity {
    pub fn new(
        id: Option<i32>,
        project_id: i32,
        content: Option<Json<Value>>,
        created_on: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            project_id,
            content,
            created_on,
        }
    }

    pub fn id(&self) -> Option<&i32> {
        self.id.as_ref()
    }

    pub fn project_id(&self) -> i32 {
        self.project_id
    }

    pub fn content(&self) -> Option<&Json<Value>> {
        self.content.as_ref()
    }

    pub fn created_on(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.created_on.as_ref()
    }
}
