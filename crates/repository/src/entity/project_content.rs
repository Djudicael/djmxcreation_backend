use app_core::dto::project_content_dto::ProjectContentDto;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::value_to_content;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContent {
    id: Option<Uuid>,
    project_id: Uuid,
    content: Option<Value>,
    created_on: Option<DateTime<Utc>>,
}

impl ProjectContent {
    pub fn new(
        id: Option<Uuid>,
        project_id: Uuid,
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

impl From<ProjectContent> for ProjectContentDto {
    fn from(val: ProjectContent) -> ProjectContentDto {
        ProjectContentDto::new(
            val.id,
            val.project_id,
            val.content.and_then(value_to_content),
            val.created_on,
        )
    }
}
