use app_core::dto::{content_dto::ContentDto, project_content_dto::ProjectContentDto};
use serde_json::Value;
use sqlx::types::{chrono, Json};

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct ProjectContent {
    id: Option<i32>,
    project_id: i32,
    content: Option<Json<Value>>,
    created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProjectContent {
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
}

impl From<ProjectContent> for ProjectContentDto {
    fn from(val: ProjectContent) -> ProjectContentDto {
        ProjectContentDto::new(
            val.id,
            val.project_id,
            val.content
                .map(|content_json| content_json.0)
                .and_then(to_content),
            val.created_on,
        )
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}
