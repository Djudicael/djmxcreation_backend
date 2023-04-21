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
    pub adult: bool,
    created_on: Option<chrono::DateTime<chrono::Utc>>,
    updated_on: Option<chrono::DateTime<chrono::Utc>>,
    pub contents: Vec<ContentView>,
}

impl ProjectView {
    pub fn new() -> Self {
        Self {
            id: None,
            metadata: None,
            description: None,
            visible: false,
            adult: false,
            created_on: None,
            updated_on: None,
            contents: vec![],
        }
    }

    pub fn id(mut self, id: Option<i32>) -> Self {
        self.id = id;
        self
    }

    pub fn metadata(mut self, metadata: Option<MetadataDto>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn description(mut self, description: Option<Value>) -> Self {
        self.description = description;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn adult(mut self, adult: bool) -> Self {
        self.adult = adult;
        self
    }

    pub fn created_on(mut self, created_on: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        self.created_on = created_on;
        self
    }

    pub fn updated_on(mut self, updated_on: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        self.updated_on = updated_on;
        self
    }

    pub fn contents(mut self, contents: Vec<ContentView>) -> Self {
        self.contents = contents;
        self
    }

    pub fn build(self) -> ProjectView {
        ProjectView {
            id: self.id,
            metadata: self.metadata,
            description: self.description,
            visible: self.visible,
            adult: self.adult,
            created_on: self.created_on,
            updated_on: self.updated_on,
            contents: self.contents,
        }
    }
}

impl From<ProjectDto> for ProjectView {
    fn from(dto: ProjectDto) -> Self {
        let contents: Vec<ContentView> = vec![];

        Self::new()
            .id(dto.id)
            .metadata(dto.metadata)
            .description(dto.description)
            .visible(dto.visible)
            .adult(dto.adult)
            .created_on(dto.created_on)
            .updated_on(dto.updated_on)
            .contents(contents)
            .build()
    }
}
