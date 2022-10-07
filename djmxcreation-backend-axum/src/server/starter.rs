use axum::Router;
pub async fn start() -> anyhow::Result<()> {
    let router = Router::new();

    axum::Server::bind(&format!("0.0.0.0:{}", "8081").parse().unwrap())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
