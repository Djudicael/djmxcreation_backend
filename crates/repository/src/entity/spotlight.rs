use app_core::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto, spotlight_dto::SpotlightDto,
};
use serde_json::Value;
use uuid::Uuid;

#[derive(Default, Debug, Clone)]
pub struct Spotlight {
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub adult: bool,
    pub metadata: Option<Value>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub thumbnail: Option<Value>,
}

impl From<Spotlight> for SpotlightDto {
    fn from(value: Spotlight) -> Self {
        Self::new()
            .adult(value.adult)
            .project_id(value.project_id)
            .created_on(value.created_on)
            .id(value.id)
            .metadata(value.metadata.and_then(to_metadata))
            .thumbnail(value.thumbnail.and_then(to_content))
            .build()
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
