use serde_json::Value;

use crate::{
    domain::{metadata::Metadata, project_entity::ProjectEntity},
    view::{content_view::ContentView, project_view::ProjectView},
};

pub fn to_view(content: &Vec<ContentView>, project: &ProjectEntity) -> ProjectView {
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
        content.to_vec(),
    )
}

fn to_metadata(value: &Value) -> Metadata {
    serde_json::from_value(value.clone()).unwrap()
}
