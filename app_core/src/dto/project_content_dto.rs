use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Default, Debug, Clone)]
pub struct ProjectContentDto {
    pub id: Option<i32>,
    pub project_id: i32,
    pub content: Option<Value>,
    pub created_on: Option<DateTime<Utc>>,
}

impl ProjectContentDto {
    pub fn new(
        id: Option<i32>,
        project_id: i32,
        content: Option<Value>,
        created_on: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            project_id,
            content,
            created_on,
        }
    }
}
