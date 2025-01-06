use crate::{
    dto::project_dto::ProjectDto,
    view::{content_view::ContentView, project_view::ProjectView},
};

pub fn to_view(contents: &[ContentView], project: &ProjectDto) -> ProjectView {
    let mut project_view: ProjectView = ProjectView::from(project.clone());
    project_view.contents = contents.to_vec();
    project_view
}
