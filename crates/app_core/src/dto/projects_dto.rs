use super::project_with_thumbnail_dto::ProjectWithThumbnailDto;

#[derive(Default, Debug, Clone)]
pub struct ProjectsDto {
    pub page: i64,
    pub size: i64,
    pub total_pages: i64,
    pub projects: Vec<ProjectWithThumbnailDto>,
}

impl ProjectsDto {
    pub fn new(
        page: i64,
        size: i64,
        total_pages: i64,
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
