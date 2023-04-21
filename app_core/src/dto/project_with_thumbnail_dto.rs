use chrono::{DateTime, Utc};

use super::{metadata_dto::MetadataDto, content_dto::ContentDto};

pub struct ProjectWithThumbnailDto {
    pub id: Option<i32>,
    pub metadata: Option<MetadataDto>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<ContentDto>,
}

impl ProjectWithThumbnailDto {
    pub fn new(
        id: Option<i32>,
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
