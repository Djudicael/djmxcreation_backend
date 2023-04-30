use super::project_with_thumbnail_dto::ProjectWithThumbnailDto;

#[derive(Default, Debug, Clone)]
pub struct ProjectsDto {
    pub page: i32,
    pub size: i32,
    pub total_pages: i32,
    pub projects: Vec<ProjectWithThumbnailDto>,
}

impl ProjectsDto {
    pub fn new(
        page: i32,
        size: i32,
        total_pages: i32,
        projects: Vec<ProjectWithThumbnailDto>,
    ) -> Self {
        Self {
            page,
            size,
            total_pages,
            projects,
        }
    }
}
