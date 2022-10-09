use app_domain::{
    metadata::Metadata,
    view::{content_view::ContentView, project_payload::ProjectPayload, project_view::ProjectView},
};
use axum::{
    extract::{Multipart, Path},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    controller::project_controller::{
        handler_add_project, handler_create_project, handler_delete_content_project,
        handler_delete_project, handler_find_project, handler_get_projects, handler_update_project,
    },
    error::axum_error::ApiResult,
};

pub fn project_router() -> Router {
    Router::new()
        .route("/v1/projects", post(create_project))
        .route("/v1/projects", get(handler_get_projects))
        .route("/v1/projects/:id/contents", patch(add_project))
        .route("/v1/projects/:id", put(update_project))
        .route("/v1/projects/:id", get(find_project))
        .route("/v1/projects/:id", delete(delete_project))
        .route(
            "/v1/projects/:id/contents/:content_id",
            delete(delete_content_project),
        )
}

pub async fn create_project(Json(body): Json<Metadata>) -> ApiResult<Json<ProjectView>> {
    handler_create_project(body).await
}

pub async fn add_project(
    Path(id): Path<i32>,
    form: Multipart,
) -> ApiResult<Json<Vec<ContentView>>> {
    handler_add_project(id, form).await
}

pub async fn update_project(
    Path(id): Path<i32>,
    Json(project): Json<ProjectPayload>,
) -> ApiResult<()> {
    handler_update_project(id, project).await
}

pub async fn find_project(Path(id): Path<i32>) -> ApiResult<Json<ProjectView>> {
    handler_find_project(id).await
}

pub async fn delete_project(Path(id): Path<i32>) -> ApiResult<()> {
    handler_delete_project(id).await
}

#[derive(Deserialize)]
pub struct DeleteContentParams {
    id: i32,
    content_id: i32,
}

// alternative to the struct:
//  Path((id, content_id)): Path<(i32, i32)
pub async fn delete_content_project(
    Path(DeleteContentParams { id, content_id }): Path<DeleteContentParams>,
) -> ApiResult<()> {
    handler_delete_content_project(id, content_id).await
}
