use app_core::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
    project_dto::ProjectDto,
};
use serde_json::Value;
use sqlx::types::{chrono, Json};

use super::project_content::ProjectContent;

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct Project {
    pub id: Option<i32>,
    pub metadata: Option<Json<Value>>,
    pub description: Option<Json<Value>>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_on: Option<chrono::DateTime<chrono::Utc>>,
    pub contents: Vec<Json<Value>>,
    pub thumbnail_content: Option<Json<Value>>,
}

#[derive(sqlx::FromRow, Default, Debug, Clone)]
pub struct ProjectCreated {
    pub id: Option<i32>,
    pub metadata: Option<Json<Value>>,
    pub description: Option<Json<Value>>,
    pub visible: bool,
    pub adult: bool,
    pub created_on: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_on: Option<chrono::DateTime<chrono::Utc>>,
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

    pub fn id(mut self, id: Option<i32>) -> Self {
        self.id = id;
        self
    }

    pub fn metadata(mut self, metadata: Option<Json<Value>>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn description(mut self, description: Option<Json<Value>>) -> Self {
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

    pub fn contents(mut self, contents: Vec<Json<Value>>) -> Self {
        self.contents = contents;
        self
    }

    pub fn thumbnail_content(mut self, thumbnail_content: Option<Json<Value>>) -> Self {
        self.thumbnail_content = thumbnail_content;
        self
    }

    pub fn build(self) -> Project {
        Project {
            id: self.id,
            metadata: self.metadata,
            description: self.description,
            visible: self.visible,
            adult: self.adult,
            created_on: self.created_on,
            updated_on: self.updated_on,
            contents: self.contents,
            thumbnail_content: self.thumbnail_content,
        }
    }
}

impl From<Project> for ProjectDto {
    fn from(val: Project) -> ProjectDto {
        ProjectDto::new()
            .id(val.id)
            .metadata(
                val.metadata
                    .map(|metadata_json| metadata_json.0)
                    .and_then(to_metadata),
            )
            .description(val.description.map(|description_json| description_json.0))
            .visible(val.visible)
            .adult(val.adult)
            .created_on(val.created_on)
            .updated_on(val.updated_on)
            .contents(
                val.contents
                    .into_iter()
                    .map(|content_json| content_json.0)
                    .flat_map(to_content)
                    .map(ProjectContentDto::from)
                    .collect(),
            )
            .thumbnail(
                val.thumbnail_content
                    .map(|thumbnail_json| thumbnail_json.0)
                    .and_then(to_thumbnail),
            )
            .build()
    }
}
impl From<ProjectCreated> for ProjectDto {
    fn from(val: ProjectCreated) -> ProjectDto {
        ProjectDto::new()
            .id(val.id)
            .metadata(
                val.metadata
                    .map(|metadata_json| metadata_json.0)
                    .and_then(to_metadata),
            )
            .description(val.description.map(|description_json| description_json.0))
            .visible(val.visible)
            .adult(val.adult)
            .created_on(val.created_on)
            .updated_on(val.updated_on)
            .contents(vec![])
            .thumbnail(None)
            .build()
    }
}

fn to_content(value: Value) -> Option<ProjectContent> {
    serde_json::from_value(value).ok()
}
fn to_thumbnail(value: Value) -> Option<ContentDto> {
    serde_json::from_value(value).ok()
}

fn to_metadata(value: Value) -> Option<MetadataDto> {
    serde_json::from_value(value).ok()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_project_build() {
//         let project = Project::new()
//             .id(Some(1))
//             .metadata(Some(Json(Value::String(String::from("metadata")))))
//             .description(Some(Json(Value::String(String::from("description")))))
//             .visible(true)
//             .adult(false)
//             .created_on(Some(chrono::Utc::now()))
//             .updated_on(Some(chrono::Utc::now()))
//             .contents(vec![Json(Value::String(String::from("content")))])
//             .thumbnail_content(Some(Json(Value::String(String::from("thumbnail"))))))
//             .build();

//         assert_eq!(project.id, Some(1));
//         assert_eq!(project.metadata.unwrap().0, String::from("metadata"));
//         assert_eq!(project.description.unwrap().0, String::from("description"));
//         assert_eq!(project.visible, true);
//         assert_eq!(project.adult, false);
//         assert!(project.created_on.is_some());
//         assert!(project.updated_on.is_some());
//         assert_eq!(project.contents.len(), 1);
//         assert_eq!(project.thumbnail_content.unwrap().0, String::from("thumbnail"));
//     }
// }
