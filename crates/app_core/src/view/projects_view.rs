use serde::{Deserialize, Serialize};

use super::project_with_thumbnail_view::ProjectWithThumbnailView;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsView {
    pub page: i64,
    pub size: i64,
    pub total_pages: i64,
    pub projects: Vec<ProjectWithThumbnailView>,
}

impl ProjectsView {
    pub fn new(
        page: i64,
        size: i64,
        total_pages: i64,
        projects: Vec<ProjectWithThumbnailView>,
    ) -> Self {
        Self {
            page,
            size,
            total_pages,
            projects,
        }
    }
}
