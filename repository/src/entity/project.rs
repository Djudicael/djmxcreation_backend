use app_core::dto::{metadata_dto::MetadataDto, project_dto::ProjectDto};
use serde_json::Value;
use sqlx::types::{chrono, Json};

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct Project {
    pub id: Option<i32>,
    pub metadata: Option<Json<Value>>,
    pub description: Option<Json<Value>>,
    pub visible: bool,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_on: Option<chrono::DateTime<chrono::Utc>>,
}

impl Project {
    pub fn new(
        id: Option<i32>,
        metadata: Option<Json<Value>>,
        description: Option<Json<Value>>,
        visible: bool,
        created_on: Option<chrono::DateTime<chrono::Utc>>,
        updated_on: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            metadata,
            description,
            visible,
            created_on,
            updated_on,
        }
    }
}

impl From<Project> for ProjectDto {
    fn from(val: Project) -> ProjectDto {
        ProjectDto::new(
            val.id,
            val.metadata
                .map(|metadata_json| metadata_json.0)
                .and_then(to_metadata),
            val.description.map(|description_json| description_json.0),
            val.visible,
            val.created_on,
            val.updated_on,
        )
    }
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}
