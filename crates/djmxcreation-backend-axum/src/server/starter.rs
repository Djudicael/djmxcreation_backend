use std::{future::ready, sync::Arc, time::Duration};

use crate::{
    router::{
        about_me_router::AboutMeRouter, contact_router::ContactRouter,
        observability_router::ObservabilityRouter, project_router::ProjectRouter,
    },
    service::service_register::ServiceRegister,
};
use anyhow::Context;
use app_config::config::Config;
use axum::{
    BoxError, Json, Router,
    error_handling::HandleErrorLayer,
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use hyper::StatusCode;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use repository::config::{
    db::DatabasePool,
    storage::{ensure_bucket, get_storage_client},
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

const HTTP_TIMEOUT_SECS: u64 = 30;

/// Exponential histogram buckets for HTTP request duration metrics (seconds).
const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "error": format!("request took longer than the {HTTP_TIMEOUT_SECS}s timeout")
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("unhandled internal error: {err}") })),
        )
    }
}

/// Record HTTP request duration for Prometheus metrics.
async fn track_metrics(request: Request, next: Next) -> Response {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());
    let method = request.method().clone();

    let response = next.run(request).await;

    let _ = metrics::histogram!(
        "http_requests_duration_seconds",
        "method" => method.to_string(),
        "path"   => path,
        "status" => response.status().as_u16().to_string(),
    );

    response
}

pub async fn start() -> anyhow::Result<()> {
    // ── Configuration ────────────────────────────────────────────────────────
    let config = Config::from_env().context("failed to load configuration")?;

    // ── Database ─────────────────────────────────────────────────────────────
    let client_db = DatabasePool::new(&config.database, None)
        .await
        .context("failed to initialise database pool")?;
    info!("database pool ready");

    // ── Object storage ────────────────────────────────────────────────────────
    let storage_cfg = config.get_storage();
    let bucket_name = storage_cfg.bucket.clone();

    let storage_client =
        get_storage_client(storage_cfg).context("failed to create storage client")?;
    ensure_bucket(&bucket_name, &storage_client)
        .await
        .context("failed to ensure storage bucket exists")?;
    info!(bucket = %bucket_name, "storage ready");

    // ── Prometheus metrics ────────────────────────────────────────────────────
    let recorder_handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .context("invalid prometheus bucket configuration")?
        .install_recorder()
        .context("failed to install prometheus recorder")?;

    // ── Services ──────────────────────────────────────────────────────────────
    let service_register = ServiceRegister::new(Arc::new(client_db), storage_client, bucket_name);

    // ── Router ────────────────────────────────────────────────────────────────
    let router = Router::new()
        .nest("/", ObservabilityRouter::new_router())
        .nest(
            "/api/about",
            AboutMeRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/portfolio",
            ProjectRouter::new_router(service_register.clone()),
        )
        .nest(
            "/api/contact",
            ContactRouter::new_router(service_register.clone()),
        )
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS)),
        )
        .layer(if config.cors_origins.is_empty() {
            CorsLayer::new()
        } else {
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(
                    config.cors_origins.iter().filter_map(|o| o.parse().ok()),
                ))
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        })
        .route_layer(middleware::from_fn(track_metrics));

    // ── Server ────────────────────────────────────────────────────────────────
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;
    info!(address = %addr, "server listening");

    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
