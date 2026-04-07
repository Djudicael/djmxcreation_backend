use chrono::{DateTime, Utc};
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
    pub created_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<ContentView>,
}

impl SpotlightView {
    pub fn new(
        id: Option<Uuid>,
        project_id: Uuid,
        adult: bool,
        metadata: Option<MetadataDto>,
        created_on: Option<DateTime<Utc>>,
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

