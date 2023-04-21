use chrono::{DateTime, Utc};
use serde_json::Value;

use super::metadata_dto::MetadataDto;

#[derive(Default, Debug, Clone)]
pub struct ProjectDto {
    pub id: Option<i32>,
    pub metadata: Option<MetadataDto>,
    pub description: Option<Value>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
}

impl ProjectDto {
    pub fn new(
        id: Option<i32>,
        metadata: Option<MetadataDto>,
        description: Option<Value>,
        visible: bool,
        adult: bool,
        created_on: Option<DateTime<Utc>>,
        updated_on: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            metadata,
            description,
            visible,
            adult,
            created_on,
            updated_on,
        }
    }
}
