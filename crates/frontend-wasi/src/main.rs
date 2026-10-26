#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = std::env::var("FRONTEND_APP")?;
    let port = std::env::var("PORT")?;
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    axum::serve(listener, djmxcreation_frontend_wasi::app_router(&app)).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
