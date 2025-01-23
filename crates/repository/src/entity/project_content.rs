use app_core::dto::{content_dto::ContentDto, project_content_dto::ProjectContentDto};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContent {
    id: Option<i32>,
    project_id: i32,
    content: Option<Value>,
    created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProjectContent {
    pub fn new(
        id: Option<i32>,
        project_id: i32,
        content: Option<Value>,
        created_on: Option<chrono::DateTime<chrono::Utc>>,
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
            val.content.and_then(to_content),
            val.created_on,
        )
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}
