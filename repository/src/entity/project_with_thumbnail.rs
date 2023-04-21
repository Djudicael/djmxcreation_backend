use app_core::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto,
    project_with_thumbnail_dto::ProjectWithThumbnailDto,
};
use serde_json::Value;
use sqlx::types::Json;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct ProjectWithThumbnail {
    pub id: Option<i32>,
    pub metadata: Option<Json<Value>>,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<serde_json::Value>,
    pub visible: bool,
    pub adult: bool,
    pub thumbnail_content: Option<Json<Value>>,
    pub thumbnail_created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<ProjectWithThumbnail> for ProjectWithThumbnailDto {
    fn from(value: ProjectWithThumbnail) -> Self {
        Self::new(
            value.id,
            value
                .metadata
                .map(|metadata_json| metadata_json.0)
                .and_then(to_metadata),
            value.visible,
            value.adult,
            Some(value.created_on),
            value.updated_on,
            value
                .thumbnail_content
                .map(|content_json| content_json.0)
                .and_then(to_content),
        )
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
