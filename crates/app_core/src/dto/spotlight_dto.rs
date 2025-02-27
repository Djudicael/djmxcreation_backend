use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{content_dto::ContentDto, metadata_dto::MetadataDto};

#[derive(Default, Debug, Clone)]
pub struct SpotlightDto {
    pub id: Option<Uuid>,
    pub project_id: Uuid,
    pub adult: bool,
    pub metadata: Option<MetadataDto>,
    pub created_on: Option<DateTime<Utc>>,
    pub thumbnail: Option<ContentDto>,
}

impl SpotlightDto {
    pub fn new() -> Self {
        Self {
            id: None,
            project_id: Uuid::new_v4(),
            adult: false,
            metadata: None,
            created_on: None,
            thumbnail: None,
        }
    }

    pub fn id(mut self, id: Option<Uuid>) -> Self {
        self.id = id;
        self
    }

    pub fn project_id(mut self, project_id: Uuid) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn adult(mut self, adult: bool) -> Self {
        self.adult = adult;
        self
    }

    pub fn metadata(mut self, metadata: Option<MetadataDto>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn created_on(mut self, created_on: Option<DateTime<Utc>>) -> Self {
        self.created_on = created_on;
        self
    }

    pub fn thumbnail(mut self, thumbnail: Option<ContentDto>) -> Self {
        self.thumbnail = thumbnail;
        self
    }

    pub fn build(self) -> Self {
        SpotlightDto {
            id: self.id,
            project_id: self.project_id,
            adult: self.adult,
            metadata: self.metadata,
            created_on: self.created_on,
            thumbnail: self.thumbnail,
        }
    }
}
