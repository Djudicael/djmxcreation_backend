use crate::{
    app_error::Error,
    domain::metadata::Metadata,
    mapper::project_mapper::to_view,
    repository::project_repository::create,
    view::{content_view::ContentView, project_view::ProjectView},
};

pub async fn create_project(metadata: &Metadata) -> Result<ProjectView, Error> {
    let project = create(metadata).await?;
    let contents: Vec<ContentView> = vec![];
    let projectView = to_view(&contents, &project);
    Ok(projectView)
}
