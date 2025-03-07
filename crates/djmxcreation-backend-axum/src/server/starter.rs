use std::{
    future::ready,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    router::{
        about_me_router::AboutMeRouter, contact_router::ContactRouter,
        project_router::ProjectRouter,
    },
    service::service_register::ServiceRegister,
};

use anyhow::Context;
use app_config::config::Config;
use axum::body::Body;
use axum::{
    BoxError, Json, Router,
    error_handling::HandleErrorLayer,
    extract::MatchedPath,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use hyper::{Method, Request, StatusCode};
use metrics_exporter_prometheus::PrometheusHandle;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};

use hyper::header;
use metrics::counter;
use metrics::histogram;
use once_cell::sync::Lazy;
use repository::config::{
    db::DatabasePool,
    minio::{create_bucket, get_aws_client},
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// constants for configuration
const METRICS_PATH: &str = "/metrics";
const HEALTH_PATH: &str = "/health";

static GLOBAL_CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::new()));

const HTTP_TIMEOUT: u64 = 30;
const EXPONENTIAL_SECONDS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

pub fn init_logger() {
    let filter = EnvFilter::new("info");
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();
}

async fn push_log(level: &str, message: &str) {
    let client = Client::new();
    let payload = json!({
        "level": level,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Err(e) = client
        .post("http://fluent-bit:9880/logs") //to modify
        .json(&payload)
        .send()
        .await
    {
        error!("Failed to send log: {}", e);
    }
}

async fn track_metrics(req: Request<Body>, next: Next) -> Response {
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    let method = req.method().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let log_level = if status >= 500 {
        "error"
    } else if status >= 400 {
        "warn"
    } else {
        "info"
    };

    let message = format!(
        "Request [{} {}] - {} - {:.3}s",
        method, path, status, latency
    );

    push_log(log_level, &message).await;

    let _ = histogram!(
        "http_requests_duration_seconds",
        "latency"=>latency.to_string(),
        "path" => path.clone(),
        "method" => method.to_string(),
        "status" => status.clone()
    );

    let counter = counter!(
        "http_requests_total",
        "path" => path,
        "method" => method.to_string(),
        "status" => status
    );

    counter.increment(1);

    response
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, Json<serde_json::Value>) {
    let error_message = if err.is::<tower::timeout::error::Elapsed>() {
        format!("Request Timeout: {} seconds", HTTP_TIMEOUT)
    } else {
        format!("Unhandled Error: {}", err)
    };

    push_log("error", &error_message).await;

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": error_message })),
    )
}

pub async fn start() -> anyhow::Result<()> {
    let config = GLOBAL_CONFIG.clone();

    // Initialize database
    let client_db = DatabasePool::new(&config.database, None)
        .await
        .context("Failed to initialize database pool")?;

    // Initialize storage
    let storage = config.clone().get_storage();
    let storage_client =
        get_aws_client(storage).context("Failed to create object Storage client")?;
    create_bucket("portfolio", storage_client.clone())
        .await
        .context("Failed to create bucket")?;

    // Initialize services
    let service_register = ServiceRegister::new(Arc::new(client_db), storage_client);

    // Setup metrics
    let recorder_handle = setup_metrics()?;

    // Create router with all middlewares
    let app = create_router(service_register, recorder_handle)?;

    // Start server
    let addr = format!("0.0.0.0:{}", &config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    println!("Server listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn create_router(
    service_register: ServiceRegister,
    metrics_handler: PrometheusHandle,
) -> anyhow::Result<Router> {
    let cors = CorsLayer::new()
        // .allow_origin(
        //     config
        //         .allowed_origins
        //         .iter()
        //         .map(|o| o.parse::<HeaderValue>().unwrap())
        //         .collect::<Vec<_>>(),
        // )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .max_age(Duration::from_secs(3600));

    let app = Router::new()
        .route(HEALTH_PATH, get(health_check))
        .route(METRICS_PATH, get(move || ready(metrics_handler.render())))
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
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(HTTP_TIMEOUT))
                .layer(middleware::from_fn(track_metrics))
                .layer(cors),
        );

    Ok(app)
}

fn setup_metrics() -> anyhow::Result<PrometheusHandle> {
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(String::from("http_requests_duration_seconds")),
            EXPONENTIAL_SECONDS,
        )
        .context("Failed to setup metrics buckets")?
        .install_recorder()
        .context("Failed to install metrics recorder")
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
