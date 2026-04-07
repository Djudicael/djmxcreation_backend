use app_core::dto::{project_content_dto::ProjectContentDto, project_dto::ProjectDto};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::{project_content::ProjectContent, value_to_content, value_to_metadata};

#[derive(Default, Debug, Clone)]
pub struct Project {
    pub id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub description: Option<Value>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub contents: Vec<Value>,
    pub thumbnail_content: Option<Value>,
}

impl Project {
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
            thumbnail_content: None,
        }
    }

    pub fn id(mut self, id: Option<Uuid>) -> Self {
        self.id = id;
        self
    }

    pub fn metadata(mut self, metadata: Option<Value>) -> Self {
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

    pub fn contents(mut self, contents: Vec<Value>) -> Self {
        self.contents = contents;
        self
    }

    pub fn thumbnail_content(mut self, thumbnail_content: Option<Value>) -> Self {
        self.thumbnail_content = thumbnail_content;
        self
    }

}

impl From<Project> for ProjectDto {
    fn from(val: Project) -> ProjectDto {
        ProjectDto::new()
            .id(val.id)
            .metadata(val.metadata.and_then(value_to_metadata))
            .description(val.description)
            .visible(val.visible)
            .adult(val.adult)
            .created_on(val.created_on)
            .updated_on(val.updated_on)
            .contents(
                val.contents
                    .into_iter()
                    .filter_map(|v| serde_json::from_value::<ProjectContent>(v).ok())
                    .map(ProjectContentDto::from)
                    .collect(),
            )
            .thumbnail(val.thumbnail_content.and_then(value_to_content))
    }
}

