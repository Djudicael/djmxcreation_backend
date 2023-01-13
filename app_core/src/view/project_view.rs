use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dto::{metadata_dto::MetadataDto, project_dto::ProjectDto};

use super::content_view::ContentView;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectView {
    pub id: Option<i32>,
    pub metadata: Option<MetadataDto>,
    pub description: Option<Value>,
    pub visible: bool,
    // created_on: Option<chrono::DateTime<chrono::Utc>>,
    // updated_on: Option<chrono::DateTime<chrono::Utc>>,
    pub contents: Vec<ContentView>,
}

impl ProjectView {
    pub fn new(
        id: Option<i32>,
        metadata: Option<MetadataDto>,
        description: Option<Value>,
        visible: bool,
        contents: Vec<ContentView>,
    ) -> Self {
        Self {
            id,
            metadata,
            description,
            visible,
            contents,
        }
    }
}

impl From<ProjectDto> for ProjectView {
    fn from(dto: ProjectDto) -> Self {
        let contents: Vec<ContentView> = vec![];
        Self::new(
            dto.id,
            dto.metadata.map(to_metadata),
            dto.description,
            dto.visible,
            contents,
        )
    }
}

fn to_metadata(value: Value) -> MetadataDto {
    serde_json::from_value(value.clone()).unwrap()
}
