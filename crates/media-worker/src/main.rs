#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3010".into());
    let bind = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{bind}:{port}")).await?;
    axum::serve(listener, djmxcreation_media_worker::configured_router()).await?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("Build the WASI HTTP library component with --lib");
}
