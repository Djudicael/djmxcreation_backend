use app_core::dto::spotlight_dto::SpotlightDto;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::{value_to_content, value_to_metadata};

#[derive(Default, Debug, Clone)]
pub struct Spotlight {
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub adult: bool,
    pub metadata: Option<Value>,
    pub created_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<Value>,
}

impl From<Spotlight> for SpotlightDto {
    fn from(value: Spotlight) -> Self {
        Self::new()
            .adult(value.adult)
            .project_id(value.project_id)
            .created_on(value.created_on)
            .id(value.id)
            .metadata(value.metadata.and_then(value_to_metadata))
            .thumbnail(value.thumbnail.and_then(value_to_content))
    }
}
