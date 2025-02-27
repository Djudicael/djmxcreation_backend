use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::content_dto::ContentDto;

#[derive(Default, Debug, Clone)]
pub struct ProjectContentDto {
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub content: Option<ContentDto>,
    pub created_on: Option<DateTime<Utc>>,
}

impl ProjectContentDto {
    pub fn new(
        id: Option<Uuid>,
        project_id: Uuid,
        content: Option<ContentDto>,
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
