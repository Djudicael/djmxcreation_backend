use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::dto::{
    content_dto::ContentDto, metadata_dto::MetadataDto, project_content_dto::ProjectContentDto,
    project_dto::ProjectDto, projects_dto::ProjectsDto,
};

pub type DynIProjectRepository = Arc<dyn IProjectRepository + Send + Sync>;

#[async_trait]
pub trait IProjectRepository {
    async fn create(&self, metadata: &MetadataDto) -> Result<ProjectDto, Error>;

    async fn add_project_content(
        &self,
        project_id: i32,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error>;

    async fn add_project_thumbnail(
        &self,
        project_id: i32,
        thumbnail: &ContentDto,
    ) -> Result<(), Error>;

    async fn get_project_by_id(&self, id: i32) -> Result<ProjectDto, Error>;

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error>;

    async fn get_projects_with_filter(
        &self,
        page: i32,
        size: i32,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error>;

    async fn update_project_entity(
        &self,
        project_id: i32,
        project: &ProjectDto,
    ) -> Result<(), Error>;

    async fn get_projects_contents(&self, project_id: i32)
        -> Result<Vec<ProjectContentDto>, Error>;

    async fn get_projects_content_by_id(
        &self,
        project_id: i32,
        id: i32,
    ) -> Result<ProjectContentDto, Error>;

    async fn delete_project_content_by_id(&self, project_id: i32, id: i32) -> Result<(), Error>;
    async fn delete_project_by_id(&self, project_id: i32) -> Result<(), Error>;
    async fn get_projects_content_thumbnail(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectContentDto>, Error>;
}
