use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

pub struct ObservabilityRouter;

impl ObservabilityRouter {
    pub fn new_router() -> Router {
        Router::new()
            .route("/ping", get(ping))
            .route("/metrics", get(metrics))
    }
}

async fn ping() -> Json<PingResponse> {
    Json(PingResponse::default())
}

async fn metrics() -> &'static str {
    "# no metrics available in wasi build\n"
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct PingResponse {
    pub message: String,
}

impl PingResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Default for PingResponse {
    fn default() -> Self {
        Self::new(String::from("API is responsive"))
    }
}
