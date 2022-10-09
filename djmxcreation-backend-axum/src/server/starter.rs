use crate::router::{about_me_router, project_router};
use axum::Router;
pub async fn start() -> anyhow::Result<()> {
    let router = Router::new()
        .nest("/api", about_me_router::about_me_router())
        .nest("/api", project_router::project_router());

    axum::Server::bind(&format!("0.0.0.0:{}", "8081").parse().unwrap())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
