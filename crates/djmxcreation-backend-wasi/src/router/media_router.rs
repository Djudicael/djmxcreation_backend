use crate::{
    error::error::ApiResult, service::service_register::ServiceRegister,
    validation::validate_file_name,
};
use app_core::media::{DynMediaRepository, MediaDetails, MediaOrder, UploadRequest, UploadSession};
use axum::{
    Extension, Json, Router,
    extract::Path,
    routing::{post, put},
};
use serde::Serialize;
use uuid::Uuid;

pub fn router(services: ServiceRegister) -> Router {
    Router::new()
        .route("/v1/projects/{project}/media-order", put(reorder))
        .route("/v1/projects/{project}/uploads", post(begin))
        .route(
            "/v1/projects/{project}/uploads/{session}/parts/{part}",
            post(part_url),
        )
        .route(
            "/v1/projects/{project}/uploads/{session}/complete",
            post(complete),
        )
        .route(
            "/v1/projects/{project}/uploads/{session}/abort",
            post(abort),
        )
        .route("/v1/projects/{project}/media/{content}", put(edit))
        .layer(Extension(services.media_repository))
}

async fn reorder(
    Extension(repo): Extension<DynMediaRepository>,
    Path(project): Path<Uuid>,
    Json(order): Json<MediaOrder>,
) -> ApiResult<()> {
    Ok(repo.reorder(project, order).await?)
}

async fn begin(
    Extension(repo): Extension<DynMediaRepository>,
    Path(project): Path<Uuid>,
    Json(request): Json<UploadRequest>,
) -> ApiResult<Json<UploadSession>> {
    validate_file_name(&request.file_name)?;
    Ok(Json(repo.begin(project, request).await?))
}

#[derive(Serialize)]
struct Url {
    url: String,
}
async fn part_url(
    Extension(repo): Extension<DynMediaRepository>,
    Path((project, session, part)): Path<(Uuid, Uuid, i32)>,
) -> ApiResult<Json<Url>> {
    Ok(Json(Url {
        url: repo.part_url(project, session, part).await?,
    }))
}

#[derive(Serialize)]
struct Uploaded {
    id: Uuid,
}
async fn complete(
    Extension(repo): Extension<DynMediaRepository>,
    Path((project, session)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Uploaded>> {
    Ok(Json(Uploaded {
        id: repo.complete(project, session).await?,
    }))
}
async fn abort(
    Extension(repo): Extension<DynMediaRepository>,
    Path((project, session)): Path<(Uuid, Uuid)>,
) -> ApiResult<()> {
    Ok(repo.abort(project, session).await?)
}
async fn edit(
    Extension(repo): Extension<DynMediaRepository>,
    Path((project, content)): Path<(Uuid, Uuid)>,
    Json(details): Json<MediaDetails>,
) -> ApiResult<()> {
    Ok(repo.edit(project, content, details).await?)
}
