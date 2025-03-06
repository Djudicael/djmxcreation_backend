use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;
use uuid::Uuid;

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
        project_id: Uuid,
        content: &ContentDto,
    ) -> Result<ProjectContentDto, Error>;

    async fn add_project_thumbnail(
        &self,
        project_id: Uuid,
        thumbnail: &ContentDto,
    ) -> Result<ProjectContentDto, Error>;

    async fn get_project_by_id(&self, id: Uuid) -> Result<Option<ProjectDto>, Error>;

    async fn get_projects(&self) -> Result<Vec<ProjectDto>, Error>;

    async fn get_projects_with_filter(
        &self,
        page: i64,
        size: i64,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsDto, Error>;

    async fn update_project_entity(
        &self,
        project_id: Uuid,
        project: &ProjectDto,
    ) -> Result<(), Error>;

    async fn get_projects_contents(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error>;

    async fn get_projects_content_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error>;

    async fn delete_project_content_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error>;
    async fn delete_project_by_id(&self, project_id: Uuid) -> Result<(), Error>;
    async fn get_projects_content_thumbnail(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectContentDto>, Error>;

    async fn delete_thumbnail_by_id(&self, project_id: Uuid, id: Uuid) -> Result<(), Error>;
    async fn get_thumbnail_by_id(
        &self,
        project_id: Uuid,
        id: Uuid,
    ) -> Result<Option<ProjectContentDto>, Error>;
}
