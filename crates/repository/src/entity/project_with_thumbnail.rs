use app_core::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto, project_dto::ProjectDto,
    project_with_thumbnail_dto::ProjectWithThumbnailDto,
};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProjectWithThumbnail {
    pub id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_on: chrono::DateTime<chrono::Utc>,
    pub updated_on: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<serde_json::Value>,
    pub visible: bool,
    pub adult: bool,
    pub thumbnail_content: Option<Value>,
    pub thumbnail_created_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<ProjectWithThumbnail> for ProjectWithThumbnailDto {
    fn from(value: ProjectWithThumbnail) -> Self {
        Self::new(
            value.id,
            value.metadata.and_then(to_metadata),
            value.visible,
            value.adult,
            Some(value.created_on),
            value.updated_on,
            value.thumbnail_content.and_then(to_content),
        )
    }
}

impl From<ProjectWithThumbnail> for ProjectDto {
    fn from(value: ProjectWithThumbnail) -> Self {
        Self::new()
            .id(value.id)
            .metadata(value.metadata.and_then(to_metadata))
            .description(value.description)
            .visible(value.visible)
            .adult(value.adult)
            .created_on(Some(value.created_on))
            .updated_on(value.updated_on)
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
