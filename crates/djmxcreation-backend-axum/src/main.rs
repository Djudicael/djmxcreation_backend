use djmxcreation_backend_axum::server::starter::start;
use tracing::error;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialise structured logging.
    // Log level is controlled by the RUST_LOG env variable
    // (e.g. `RUST_LOG=info,repository=debug`).
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .init();

    if let Err(e) = start().await {
        error!(error = ?e, "server failed to start");
        std::process::exit(1);
    }
}
