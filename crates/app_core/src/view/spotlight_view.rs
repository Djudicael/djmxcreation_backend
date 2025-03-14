use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::metadata_dto::MetadataDto;

use super::content_view::ContentView;
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotlightView {
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub adult: bool,
    pub metadata: Option<MetadataDto>,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub thumbnail: Option<ContentView>,
}

impl SpotlightView {
    pub fn new(
        id: Option<Uuid>,
        project_id: Uuid,
        adult: bool,
        metadata: Option<MetadataDto>,
        created_on: Option<chrono::DateTime<chrono::Utc>>,
        thumbnail: Option<ContentView>,
    ) -> Self {
        Self {
            id,
            project_id,
            adult,
            metadata,
            created_on,
            thumbnail,
        }
    }
}

impl From<SpotlightView> for crate::dto::spotlight_dto::SpotlightDto {
    fn from(value: SpotlightView) -> Self {
        Self::new()
            .adult(value.adult)
            .project_id(value.project_id)
            .created_on(value.created_on)
            .id(value.id)
            .metadata(value.metadata)
            .thumbnail(None)
            .build()
    }
}
