use app_core::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto, spotlight_dto::SpotlightDto,
};
use serde_json::Value;
use sqlx::types::Json;

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct Spotlight {
    pub id: Option<i32>,
    pub project_id: i32,
    pub adult: bool,
    pub metadata: Option<Json<Value>>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub thumbnail: Option<Json<Value>>,
}

impl From<Spotlight> for SpotlightDto {
    fn from(value: Spotlight) -> Self {
        Self::new()
            .adult(value.adult)
            .project_id(value.project_id)
            .created_on(value.created_on)
            .id(value.id)
            .metadata(
                value
                    .metadata
                    .map(|metadata_json| metadata_json.0)
                    .and_then(to_metadata),
            )
            .thumbnail(
                value
                    .thumbnail
                    .map(|thumbnail_json| thumbnail_json.0)
                    .and_then(to_content),
            )
            .build()
    }
}

fn to_content(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
