use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

pub fn observability_router() -> Router {
    Router::new().route("/ping", get(ping))
}

async fn ping() -> Json<PingResponse> {
    Json(PingResponse::default())
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
