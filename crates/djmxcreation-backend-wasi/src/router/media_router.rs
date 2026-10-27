use crate::{
    error::error::ApiResult, service::service_register::ServiceRegister,
    validation::validate_file_name,
};
use app_core::media::{DynMediaRepository, MediaDetails, MediaOrder, UploadRequest, UploadSession};
use app_core::optimization::IMediaProcessing;
use axum::{
    Extension, Json, Router,
    extract::Path,
    routing::{post, put},
};
use serde::Serialize;
use uuid::Uuid;

pub fn router(services: ServiceRegister) -> Router {
    Router::new()
        .route("/v1/media/optimization/process", post(process_optimization))
        .route(
            "/v1/media/optimization/enqueue-existing",
            post(enqueue_existing),
        )
        .route("/v1/projects/{project}/media-order", put(reorder))
        .route(
            "/v1/projects/{project}/optimization/enqueue",
            post(enqueue_project),
        )
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
        .layer(Extension(services.optimization))
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
    let id = repo.complete(project, session).await?;
    // A separate request wakes optional processing; upload success never waits for it.
    Ok(Json(Uploaded { id }))
}

async fn process_optimization(
    Extension(repo): Extension<
        std::sync::Arc<repository::optimization_repository::OptimizationRepository>,
    >,
) -> ApiResult<Json<serde_json::Value>> {
    let started = repo.wake().await.unwrap_or(false);
    Ok(Json(
        serde_json::json!({"started":started,"enabled":app_core::optimization::enabled()}),
    ))
}
async fn enqueue_existing(
    Extension(repo): Extension<
        std::sync::Arc<repository::optimization_repository::OptimizationRepository>,
    >,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        serde_json::json!({"queued":repo.enqueue_existing(100).await?}),
    ))
}

pub fn maintenance_router(services: ServiceRegister) -> Router {
    Router::new()
        .route("/internal/media/maintenance", post(maintenance))
        .layer(Extension(services.optimization))
}

async fn enqueue_project(
    Extension(repo): Extension<
        std::sync::Arc<repository::optimization_repository::OptimizationRepository>,
    >,
    Path(project): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        serde_json::json!({"queued":repo.enqueue_project(project).await?}),
    ))
}
async fn maintenance(
    Extension(repo): Extension<
        std::sync::Arc<repository::optimization_repository::OptimizationRepository>,
    >,
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    if !app_core::optimization::internal_authorized(
        headers
            .get("X-Media-Worker-Key")
            .and_then(|value| value.to_str().ok()),
    ) {
        return Err(crate::error::error::Error::Forbidden);
    }
    Ok(Json(serde_json::json!({"cleaned":repo.cleanup(16).await?})))
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
