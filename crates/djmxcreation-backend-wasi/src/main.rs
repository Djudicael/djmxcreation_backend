#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    djmxcreation_backend_wasi::server::starter::run_tcp(
        djmxcreation_backend_wasi::app_router(),
    )
    .await
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("native main should not be called on wasm32; use the WASI P2 entry point instead");
}
