use std::{
    future::ready,
    time::{Duration, Instant},
};

use crate::{
    router::{
        about_me_router::AboutMeRouter, observability_router::ObservabilityRouter,
        project_router::ProjectRouter,
    },
    service::service_register::ServiceRegister,
};
use anyhow::Context;
use app_config::config::Config;
use axum::{
    error_handling::HandleErrorLayer,
    extract::MatchedPath,
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    BoxError, Json, Router,
};
use hyper::{header::HeaderValue, Method, Request, StatusCode};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use repository::config::{db::new_db_pool, minio::get_aws_client};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

const HTTP_TIMEOUT: u64 = 30;
const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            Json(json!({
                "error":
                    format!("request took longer than the configured {HTTP_TIMEOUT} second timeout")
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("unhandled internal error: {err}")
            })),
        )
    }
}

async fn track_metrics<B>(request: Request<B>, next: Next<B>) -> impl IntoResponse {
    let path = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        request.uri().path().to_owned()
    };

    let start = Instant::now();
    let method = request.method().clone();
    let response = next.run(request).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    response
}

//TODO add Migration database
pub async fn start() -> anyhow::Result<()> {
    let config = Config::new();

    let db_pool = new_db_pool(&config.database)
        .await
        .expect("Failed to connect to  client");

    let storage_client =
        get_aws_client(config.storage).expect("Failed to create object Storage client");

    let service_register = ServiceRegister::new(db_pool, storage_client);

    let recorder_handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(String::from("http_requests_duration_seconds")),
            EXPONENTIAL_SECONDS,
        )
        .context("could not setup buckets for metrics, verify matchers are correct")?
        .install_recorder()
        .context("could not install metrics recorder")?;

    let router = Router::new()
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .nest("/api", AboutMeRouter::new_router(service_register.clone()))
        .nest("/api", ObservabilityRouter::new_router())
        .nest("/api", ProjectRouter::new_router(service_register.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(HTTP_TIMEOUT)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap()) //TODO to modify
                .allow_methods([Method::GET]),
        )
        .route_layer(middleware::from_fn(track_metrics));

    axum::Server::bind(&format!("0.0.0.0:{}", &config.port).parse().unwrap())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
