use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{content_dto::ContentDto, metadata_dto::MetadataDto};

#[derive(Default, Debug, Clone)]
pub struct ProjectWithThumbnailDto {
    pub id: Option<Uuid>,
    pub metadata: Option<MetadataDto>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<ContentDto>,
}

impl ProjectWithThumbnailDto {
    pub fn new(
        id: Option<Uuid>,
        metadata: Option<MetadataDto>,
        visible: bool,
        adult: bool,
        created_on: Option<DateTime<Utc>>,
        updated_on: Option<DateTime<Utc>>,
        thumbnail: Option<ContentDto>,
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
