//! Local DJMXCreation stack CLI, modeled after OpenID Connect WASI Hub's dev tools.
mod hub;
mod proxy;
mod stack;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let options = stack::Options::parse(&root)?;
    if options.command == "migrate" {
        let _ = dotenvy::dotenv();
        return migration::run_migrations(&std::env::var("DATABASE_URL")?).await;
    }
    stack::execute(root, options).await
}
