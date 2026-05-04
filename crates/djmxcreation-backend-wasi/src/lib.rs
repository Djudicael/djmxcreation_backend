pub mod error;
pub mod router;
pub mod server;
pub mod service;
pub mod validation;

use axum::Router;
use server::starter::build_router;

#[wstd_axum::http_server]
fn main() -> Router {
    build_router()
}
