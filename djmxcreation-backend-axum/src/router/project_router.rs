use app_core::{
    dto::{metadata_dto::MetadataDto, project_dto::ProjectDto},
    project::project_service::DynIProjectService,
    view::{content_view::ContentView, project_payload::ProjectPayload, project_view::ProjectView},
};

use axum::{
    extract::{Multipart, Path},
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::axum_error::ApiResult, service::service_register::ServiceRegister};

pub struct ProjectRouter;

#[derive(Deserialize)]
pub struct DeleteContentParams {
    id: i32,
    content_id: i32,
}

impl ProjectRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/v1/projects", post(ProjectRouter::create_project))
            .route("/v1/projects", get(ProjectRouter::get_projects))
            .route(
                "/v1/projects/:id/contents",
                patch(ProjectRouter::add_project),
            )
            .route("/v1/projects/:id", put(ProjectRouter::update_project))
            .route("/v1/projects/:id", get(ProjectRouter::find_project))
            .route("/v1/projects/:id", delete(ProjectRouter::delete_project))
            .route(
                "/v1/projects/:id/contents/:content_id",
                delete(ProjectRouter::delete_content_project),
            )
            .layer(Extension(service_register.project_service))
    }

    pub async fn get_projects(
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<Vec<ProjectView>>> {
        let projects = project_service.get_portfolio_projects().await?;
        Ok(Json(projects))
    }

    pub async fn create_project(
        Extension(project_service): Extension<DynIProjectService>,
        Json(body): Json<MetadataDto>,
    ) -> ApiResult<Json<ProjectView>> {
        let new_project = project_service.create_project(&body).await?;
        Ok(Json(new_project))
    }

    pub async fn add_project(
        Extension(project_service): Extension<DynIProjectService>,
        Path(id): Path<i32>,
        mut form: Multipart,
    ) -> ApiResult<Json<Vec<ContentView>>> {
        let mut contents: Vec<ContentView> = vec![];

        while let Some(field) = form.next_field().await? {
            let uudi_v4 = Uuid::new_v4().to_string();
            // let content_type = field.get_or_insert("");
            // dbg!(content_type);
            let file_name = if let Some(file_name) = field.file_name() {
                format!("{}-{}", uudi_v4, file_name.to_owned())
            } else {
                uudi_v4
            };

            let content_view = project_service
                .add_project(id, file_name, &field.bytes().await?)
                .await?;
            contents.push(content_view);
        }

        Ok(Json(contents))
    }

    pub async fn update_project(
        Extension(project_service): Extension<DynIProjectService>,
        Path(id): Path<i32>,
        Json(project): Json<ProjectPayload>,
    ) -> ApiResult<()> {
        project_service
            .update_project(id, &ProjectDto::from(project))
            .await?;
        Ok(())
    }

    pub async fn find_project(
        Path(id): Path<i32>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<Json<ProjectView>> {
        let project = project_service.find_project(id).await?;
        Ok(Json(project))
    }

    pub async fn delete_project(
        Path(id): Path<i32>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<()> {
        project_service.delete_project(id).await?;
        Ok(())
    }

    // alternative to the struct:
    //  Path((id, content_id)): Path<(i32, i32)
    pub async fn delete_content_project(
        Path(DeleteContentParams { id, content_id }): Path<DeleteContentParams>,
        Extension(project_service): Extension<DynIProjectService>,
    ) -> ApiResult<()> {
        project_service
            .delete_project_content(id, content_id)
            .await?;
        Ok(())
    }
}
