use std::sync::Arc;

use app_error::Error;
use async_trait::async_trait;

use crate::{
    dto::{metadata_dto::MetadataDto, project_dto::ProjectDto},
    view::{content_view::ContentView, project_view::ProjectView, projects_view::ProjectsView},
};

pub type DynIProjectService = Arc<dyn IProjectService + Send + Sync>;

#[async_trait]
pub trait IProjectService {
    async fn create_project(&self, metadata: &MetadataDto) -> Result<ProjectView, Error>;
    async fn add_project(
        &self,
        id: i32,
        file_name: String,
        file: &[u8],
    ) -> Result<ContentView, Error>;
    async fn update_project(&self, id: i32, project: &ProjectDto) -> Result<(), Error>;
    async fn find_project(&self, id: i32) -> Result<ProjectView, Error>;
    async fn delete_project(&self, id: i32) -> Result<(), Error>;
    async fn delete_project_content(&self, project_id: i32, content_id: i32) -> Result<(), Error>;
    async fn get_portfolio_projects(&self) -> Result<Vec<ProjectView>, Error>;
    async fn get_projects_with_filter(
        &self,
        page: i32,
        size: i32,
        is_adult: Option<bool>,
        is_visible: bool,
    ) -> Result<ProjectsView, Error>;
}
