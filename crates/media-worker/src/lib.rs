pub mod processing;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use repository::optimization_repository::{OptimizationJob, OptimizationRepository, VariantWrite};
use serde_json::{Value, json};
use std::sync::Arc;

pub fn app_router(repo: OptimizationRepository) -> Router {
    Router::new()
        .route("/health", get(|| async { "DJMX media worker / WASI P2" }))
        .route("/internal/media/process-due", post(process_due))
        .with_state(Arc::new(repo))
}

async fn process_due(
    State(repo): State<Arc<OptimizationRepository>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    if !app_core::optimization::internal_authorized(
        headers
            .get("X-Media-Worker-Key")
            .and_then(|h| h.to_str().ok()),
    ) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if !app_core::optimization::enabled() {
        return Ok(Json(json!({"status":"disabled"})));
    }
    let Some(job) = repo
        .claim()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
    else {
        return Ok(Json(json!({"status":"idle"})));
    };
    match process_one(&repo, &job).await {
        Ok(status) => Ok(Json(json!({"status":status,"mediaId":job.id}))),
        Err(_) => {
            let _ = repo
                .fail(
                    &job,
                    "Storage unavailable; processing will be retried",
                    false,
                )
                .await;
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

pub async fn process_one(
    repo: &OptimizationRepository,
    job: &OptimizationJob,
) -> Result<&'static str, app_error::Error> {
    let original = match repo.original(job).await {
        Ok(bytes) => bytes,
        Err(app_error::Error::InvalidInput(_)) => {
            repo.fail(job, "Original exceeds processing limits", true)
                .await?;
            return Ok("skipped");
        }
        Err(error) => return Err(error),
    };
    let Some(&target) = processing::WIDTHS.get(job.step as usize) else {
        repo.fail(job, "Invalid processing step", true).await?;
        return Ok("skipped");
    };
    let variant = match processing::resize(&original, target) {
        Ok(variant) => variant,
        Err(reason) => {
            repo.fail(job, &reason, true).await?;
            return Ok("skipped");
        }
    };
    let width = variant.width as i32;
    let height = variant.height as i32;
    let mut outputs = vec![variant.baseline];
    if let Some(webp) = variant.webp {
        outputs.push(webp);
    }
    for output in outputs {
        if !repo
            .store(
                job,
                VariantWrite {
                    target: target as i32,
                    width,
                    height,
                    extension: output.extension,
                    mime: output.mime,
                    bytes: output.bytes,
                },
            )
            .await?
        {
            return Ok("cancelled");
        }
    }
    let done = variant.width < target || variant.original_width <= target || job.step == 3;
    repo.finish(
        job,
        done,
        variant.original_width as i32,
        variant.original_height as i32,
    )
    .await?;
    Ok(if done { "ready" } else { "pending" })
}

pub fn configured_router() -> Router {
    let config = app_config::config::Config::from_env().expect("media worker configuration");
    let storage = repository::config::storage::get_storage_client(config.storage)
        .expect("storage configuration");
    app_router(OptimizationRepository::new(
        Arc::new(repository::config::db::DatabaseConfig::new(
            &config.database,
        )),
        storage,
    ))
}

#[cfg(target_arch = "wasm32")]
#[wstd_axum::http_server]
fn main() -> axum::Router {
    configured_router()
}
