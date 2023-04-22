use super::project_with_thumbnail_view::ProjectWithThumbnailView;

pub struct ProjectsView {
    pub page: i32,
    pub size: i32,
    pub total_pages: i32,
    pub projects: Vec<ProjectWithThumbnailView>,
}

impl ProjectsView {
    pub fn new(
        page: i32,
        size: i32,
        total_pages: i32,
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
