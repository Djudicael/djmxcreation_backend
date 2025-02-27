use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::metadata_dto::MetadataDto;

use super::content_view::ContentView;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWithThumbnailView {
    pub id: Option<Uuid>,
    pub metadata: Option<MetadataDto>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<ContentView>,
}

impl ProjectWithThumbnailView {
    pub fn new(
        id: Option<Uuid>,
        metadata: Option<MetadataDto>,
        visible: bool,
        adult: bool,
        created_on: Option<DateTime<Utc>>,
        updated_on: Option<DateTime<Utc>>,
        thumbnail: Option<ContentView>,
    ) -> Self {
        Self {
            id,
            metadata,
            visible,
            adult,
            created_on,
            updated_on,
            thumbnail,
        }
    }
}
