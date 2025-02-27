use std::{
    future::ready,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    router::{
        about_me_router::AboutMeRouter, contact_router::ContactRouter,
        observability_router::ObservabilityRouter, project_router::ProjectRouter,
    },
    service::service_register::ServiceRegister,
};
// use aide::openapi::OpenApi;
use anyhow::Context;
use app_config::{config::Config, security_config::SecurityConfig};
use axum::{
    error_handling::HandleErrorLayer,
    extract::MatchedPath,
    // headers::{authorization::Basic, Authorization},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    BoxError,
    Json,
    Router,
    // TypedHeader,
};
use hyper::{header::HeaderValue, Method, Request, StatusCode};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
// use migration::init_db_migration;
use once_cell::sync::Lazy;
use repository::config::{
    db::db_client,
    minio::{create_bucket, get_aws_client},
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

static GLOBAL_CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config::new()));

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

// pub async fn auth<B>(
//     // run the `TypedHeader` extractor
//     TypedHeader(auth): TypedHeader<Authorization<Basic>>,
//     // you can also add more extractors here but the last
//     // extractor must implement `FromRequest` which
//     // `Request` does
//     request: Request<B>,
//     next: Next<B>,
// ) -> Result<Response, StatusCode> {
//     if token_is_valid(auth.0) {
//         let response = next.run(request).await;
//         Ok(response)
//     } else {
//         Err(StatusCode::UNAUTHORIZED)
//     }
// }

// fn token_is_valid(token: Basic) -> bool {
//     let SecurityConfig { username, password } = GLOBAL_CONFIG.clone().get_security();

//     username == token.username() && password == token.password()
// }

pub async fn start() -> anyhow::Result<()> {
    // aide::gen::on_error(|error| {
    //     println!("{error}");
    // });

    // aide::gen::extract_schemas(true);

    // let mut api = OpenApi::default();

    let config = GLOBAL_CONFIG.clone();

    // init_db_migration(&config.database)
    //     .await
    //     .expect("Failed to migrate database");

    let client = db_client(&config.database).await?;

    let storage = config.clone().get_storage();

    let storage_client = get_aws_client(storage).expect("Failed to create object Storage client");

    create_bucket("portfolio", storage_client.clone()).await?;

    let service_register = ServiceRegister::new(client, storage_client);

    let recorder_handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(String::from("http_requests_duration_seconds")),
            EXPONENTIAL_SECONDS,
        )
        .context("could not setup buckets for metrics, verify matchers are correct")?
        .install_recorder()
        .context("could not install metrics recorder")?;

    // let my_auth = MyAuth { basic_auth };

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
                .timeout(Duration::from_secs(HTTP_TIMEOUT)),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());
    // .layer(
    //     CorsLayer::new()
    //         .allow_origin(Any) //TODO to modify
    //         // .allow_origin(["http://localhost:3008".parse::<HeaderValue>().unwrap()]) //TODO to modify
    //         .allow_methods([
    //             Method::GET,
    //             Method::POST,
    //             Method::PUT,
    //             Method::DELETE,
    //             Method::OPTIONS,
    //         ]),
    // )
    // .route_layer(middleware::from_fn(track_metrics));
    // .layer(middleware::from_fn(auth));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port)).await?;

    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
