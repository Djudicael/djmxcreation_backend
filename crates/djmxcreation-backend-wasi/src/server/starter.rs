use std::sync::Arc;

use crate::{
    router::{
        about_me_router::AboutMeRouter, contact_router::ContactRouter,
        observability_router::ObservabilityRouter, project_router::ProjectRouter,
    },
    service::service_register::ServiceRegister,
};
use app_config::config::Config;
use axum::Router;
use repository::config::{
    db::DatabaseConfig,
    storage::get_storage_client,
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

#[cfg(not(target_arch = "wasm32"))]
use std::{future::ready, net::SocketAddr, time::Duration};
#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context;
#[cfg(not(target_arch = "wasm32"))]
use axum::{BoxError, Json, extract::Request, middleware::Next, response::Response, routing::get, error_handling::HandleErrorLayer, extract::MatchedPath};
#[cfg(not(target_arch = "wasm32"))]
use hyper::StatusCode;
#[cfg(not(target_arch = "wasm32"))]
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
#[cfg(not(target_arch = "wasm32"))]
use tower::ServiceBuilder;
#[cfg(not(target_arch = "wasm32"))]
use tracing::info;

#[cfg(not(target_arch = "wasm32"))]
const HTTP_TIMEOUT_SECS: u64 = 30;

#[cfg(not(target_arch = "wasm32"))]
const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[cfg(not(target_arch = "wasm32"))]
static METRICS_HANDLE: std::sync::OnceLock<PrometheusHandle> = std::sync::OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn get_or_init_metrics() -> &'static PrometheusHandle {
    METRICS_HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_requests_duration_seconds".to_string()),
                EXPONENTIAL_SECONDS,
            )
            .expect("invalid prometheus bucket configuration")
            .install_recorder()
            .expect("failed to install prometheus recorder")
    })
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

/// Build the application router (shared between native and WASI).
pub fn build_router() -> Router {
    let config = Config::from_env().expect("failed to load configuration");

    let client_db = DatabaseConfig::new(&config.database);
    let storage_cfg = config.get_storage();
    let bucket_name = storage_cfg.bucket.clone();

    let storage_client =
        get_storage_client(storage_cfg).expect("failed to create storage client");

    let service_register = ServiceRegister::new(Arc::new(client_db), storage_client, bucket_name);

    #[allow(unused_mut)]
    let mut router = Router::new()
        .merge(ObservabilityRouter::new_router())
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
        .layer(TraceLayer::new_for_http())
        .layer(if config.cors_origins.is_empty() {
            CorsLayer::new()
        } else {
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(
                    config.cors_origins.iter().filter_map(|o| o.parse().ok()),
                ))
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        });

    #[cfg(not(target_arch = "wasm32"))]
    {
        let handle = get_or_init_metrics();

        router = router
            .route("/metrics", get(move || ready(handle.render())))
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(handle_timeout_error))
                    .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS)),
            )
            .route_layer(axum::middleware::from_fn(track_metrics));
    }

    #[cfg(target_arch = "wasm32")]
    {
        router = router.route("/metrics", axum::routing::get(|| async { "# no metrics available in wasi build\n" }));
    }

    router
}

/// Run the axum server on a TCP listener with graceful shutdown (native only).
#[cfg(not(target_arch = "wasm32"))]
pub async fn run_tcp(app: Router) -> anyhow::Result<()> {
    use repository::config::storage::ensure_bucket;

    let config = Config::from_env().context("failed to load configuration")?;
    let storage_cfg = config.get_storage();
    let bucket_name = storage_cfg.bucket.clone();
    let storage_client = get_storage_client(storage_cfg).context("storage client")?;
    let _ = ensure_bucket(&bucket_name, &storage_client).await;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;
    info!(address = %addr, "server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received, starting graceful shutdown");
}
