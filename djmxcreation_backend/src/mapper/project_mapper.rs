use serde_json::{json, Value};
use sqlx::types::Json;

use crate::{
    domain::{metadata::Metadata, project_entity::ProjectEntity},
    view::{content_view::ContentView, project_view::ProjectView, project_payload::ProjectPayload},
};

pub fn to_view(contents: &Vec<ContentView>, project: &ProjectEntity) -> ProjectView {
    ProjectView::new(
        project.id().map(|id| *id),
        project
            .metadata()
            .map(|metadata| &metadata.0)
            .map(|metadata| to_metadata(metadata)),
        project
            .description()
            .map(|description| &description.0)
            .map(|description| description.clone()),
        project.visible(),
        contents.to_vec(),
    )
}

fn to_metadata(value: &Value) -> Metadata {
    serde_json::from_value(value.clone()).unwrap()
}

pub fn to_entity(project: &ProjectPayload) -> ProjectEntity {
    let metadata_json = project.metadata().map(|meta| Json(json!(*meta)));
    let description_json = project.description().map(|descript| Json(descript.clone()));

    ProjectEntity::new(
        None,
        metadata_json,
        description_json,
        project.visible(),
        None,
        None,
    )
}
