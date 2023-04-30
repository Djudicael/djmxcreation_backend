use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{metadata_dto::MetadataDto, project_content_dto::ProjectContentDto};

#[derive(Default, Debug, Clone)]
pub struct ProjectDto {
    pub id: Option<i32>,
    pub metadata: Option<MetadataDto>,
    pub description: Option<Value>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub contents: Vec<ProjectContentDto>,
    pub thumbnail: Option<ProjectContentDto>,
}

impl ProjectDto {
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
            thumbnail: None,
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

    pub fn created_on(mut self, created_on: Option<DateTime<Utc>>) -> Self {
        self.created_on = created_on;
        self
    }

    pub fn updated_on(mut self, updated_on: Option<DateTime<Utc>>) -> Self {
        self.updated_on = updated_on;
        self
    }

    pub fn contents(mut self, contents: Vec<ProjectContentDto>) -> Self {
        self.contents = contents;
        self
    }

    pub fn thumbnail(mut self, thumbnail: Option<ProjectContentDto>) -> Self {
        self.thumbnail = thumbnail;
        self
    }

    pub fn build(self) -> Self {
        ProjectDto {
            id: self.id,
            metadata: self.metadata,
            description: self.description,
            visible: self.visible,
            adult: self.adult,
            created_on: self.created_on,
            updated_on: self.updated_on,
            contents: self.contents,
            thumbnail: self.thumbnail,
        }
    }
}
