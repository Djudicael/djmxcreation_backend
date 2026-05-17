//! Dev Orchestrator — manages PostgreSQL container, RustFS container, migrations,
//! and backend server for local development.
//!
//! Usage:
//!   cargo run -p dev-server -- start     Start everything (DB + migrations + backend)
//!   cargo run -p dev-server -- stop      Stop everything
//!   cargo run -p dev-server -- status    Show status
//!   cargo run -p dev-server -- db-only   Start DB + migrations only
//!   cargo run -p dev-server -- migrate   Run migrations against existing DB
//!   cargo run -p dev-server -- wasm      Start DB + WASM component via wasmtime serve
//!   cargo run -p dev-server -- build-wasm  Build WASM component only
//!   cargo run -p dev-server -- logs      Tail all service logs

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use rustainers::ImageName;
use rustainers::runner::Runner;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info};

const DB_IMAGE: &str = "docker.io/library/postgres";
const DB_TAG: &str = "16-alpine";
const DB_USER: &str = "postgres";
const DB_PASSWORD: &str = "postgres";
const DB_NAME: &str = "portfolio";
const DB_CONTAINER_NAME: &str = "djmx-dev-postgres";

const RUSTFS_IMAGE: &str = "docker.io/rustfs/rustfs";
const RUSTFS_CONTAINER_NAME: &str = "djmx-dev-rustfs";
const RUSTFS_INTERNAL_PORT: u16 = 9000;
const RUSTFS_ACCESS_KEY: &str = "rustfsadmin";
const RUSTFS_SECRET_KEY: &str = "rustfsadmin";
const RUSTFS_REGION: &str = "us-east-1";

const BACKEND_PKG: &str = "djmxcreation-backend-wasi";

// ─── Main ───────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter("info,dev_server=debug,rustainers=warn")
        .init();

    let mut args = std::env::args();
    let _bin = args.next();
    let cmd = args.next().unwrap_or_else(|| "start".into());

    match cmd.as_str() {
        "start" => cmd_start().await,
        "stop" => cmd_stop().await,
        "status" => cmd_status().await,
        "db-only" => cmd_db_only().await,
        "migrate" => cmd_migrate().await,
        "wasm" => cmd_wasm().await,
        "build-wasm" => cmd_build_wasm().await,
        "logs" => cmd_logs().await,
        _ => {
            eprintln!("Usage: dev-server {{start|stop|status|db-only|migrate|wasm|build-wasm|logs}}");
            std::process::exit(1);
        }
    }
}

// ─── Start Command ──────────────────────────────────────────────────────────

async fn cmd_start() -> Result<()> {
    check_tools(false).await?;

    let backend_port = pick_port(8081);

    // 1. Start DB
    let db_url = start_database().await?;

    // 2. Run migrations
    migration::run_migrations(&db_url).await?;

    // 3. Start RustFS
    let rustfs_endpoint = start_rustfs().await?;

    // 4. Build backend
    build_backend().await?;

    // 5. Start backend
    start_backend(&db_url, &rustfs_endpoint, backend_port).await?;

    info!("═══════════════════════════════════════════════════════════════");
    info!("  Dev environment ready!");
    info!("");
    info!("  Backend API:   http://localhost:{}", backend_port);
    info!("  Health check:  http://localhost:{}/health", backend_port);
    info!("  Database:      {}", db_url);
    info!("  RustFS:        {}", rustfs_endpoint);
    info!("");
    info!("  Press Ctrl-C to stop everything");
    info!("═══════════════════════════════════════════════════════════════");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    shutdown_all().await;

    Ok(())
}

// ─── DB-Only Command ────────────────────────────────────────────────────────

async fn cmd_db_only() -> Result<()> {
    check_tools(false).await?;

    let db_url = start_database().await?;
    migration::run_migrations(&db_url).await?;

    info!("═══════════════════════════════════════════════════════════════");
    info!("  Database ready: {}", db_url);
    info!("  Press Ctrl-C to stop (container auto-drops)");
    info!("═══════════════════════════════════════════════════════════════");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    shutdown_all().await;

    Ok(())
}

// ─── Migrate Command ────────────────────────────────────────────────────────

async fn cmd_migrate() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/portfolio?sslmode=prefer".into());
    info!("Running migrations against {}", database_url);
    migration::run_migrations(&database_url).await?;
    info!("Migrations complete.");
    Ok(())
}

// ─── Stop Command ───────────────────────────────────────────────────────────

async fn cmd_stop() -> Result<()> {
    shutdown_all().await;
    info!("All services stopped.");
    Ok(())
}

// ─── Status Command ─────────────────────────────────────────────────────────

async fn cmd_status() -> Result<()> {
    let backend_port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8081);

    let backend_running = is_port_open(backend_port).await;
    let db_running = check_container_running(DB_CONTAINER_NAME).await;
    let rustfs_running = check_container_running(RUSTFS_CONTAINER_NAME).await;

    println!("=== Dev Environment Status ===");
    println!();
    println!(
        "Backend (port {}):  {}",
        backend_port,
        if backend_running { "Running" } else { "Not running" }
    );
    println!(
        "Database:           {}",
        if db_running { "Running" } else { "Not running" }
    );
    println!(
        "RustFS:             {}",
        if rustfs_running { "Running" } else { "Not running" }
    );
    println!();

    Ok(())
}

// ─── WASM Command ───────────────────────────────────────────────────────────

async fn cmd_wasm() -> Result<()> {
    check_tools(true).await?;

    let backend_port = pick_port(8081);

    // 1. Start DB
    let db_url = start_database().await?;

    // 2. Run migrations
    migration::run_migrations(&db_url).await?;

    // 3. Start RustFS
    let rustfs_endpoint = start_rustfs().await?;

    // 4. Build WASM
    cmd_build_wasm().await?;

    // 5. Start WASM via wasmtime serve
    start_wasmtime(&db_url, &rustfs_endpoint, backend_port).await?;

    info!("═══════════════════════════════════════════════════════════════");
    info!("  WASM dev environment ready!");
    info!("");
    info!("  Backend API:   http://localhost:{}", backend_port);
    info!("  Database:      {}", db_url);
    info!("  RustFS:        {}", rustfs_endpoint);
    info!("");
    info!("  Press Ctrl-C to stop everything");
    info!("═══════════════════════════════════════════════════════════════");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    shutdown_all().await;

    Ok(())
}

async fn cmd_build_wasm() -> Result<()> {
    info!("Building WASM component...");
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            BACKEND_PKG,
            "--target",
            "wasm32-wasip2",
            "--release",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to build WASM component")?
        .wait()
        .await?;

    if !status.success() {
        anyhow::bail!("WASM build failed (exit code: {:?})", status.code());
    }
    info!("WASM component built.");
    Ok(())
}

// ─── Logs Command ───────────────────────────────────────────────────────────

async fn cmd_logs() -> Result<()> {
    let log_dir = std::env::temp_dir().join("djmx-dev-logs");
    if !log_dir.exists() {
        println!("No log directory found at {}", log_dir.display());
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&log_dir)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        println!("\n--- {} ---", name);
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            print!("{}", content);
        }
    }

    Ok(())
}

// ─── Tool Checks ────────────────────────────────────────────────────────────

async fn check_tools(need_wasmtime: bool) -> Result<()> {
    let podman = Command::new("podman")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    if podman.is_err() || !podman.unwrap().success() {
        anyhow::bail!("podman is required but not found. Install: sudo apt-get install -y podman");
    }

    if need_wasmtime {
        let wt = Command::new("wasmtime")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
        if wt.is_err() || !wt.unwrap().success() {
            anyhow::bail!("wasmtime is required. Install: curl https://wasmtime.dev/install.sh -sSf | bash");
        }
    }

    Ok(())
}

// ─── Database ───────────────────────────────────────────────────────────────

async fn start_database() -> Result<String> {
    let exists = check_container_running(DB_CONTAINER_NAME).await;
    if exists {
        info!("Database container already running");
        return Ok(std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| format!("postgresql://{DB_USER}:{DB_PASSWORD}@localhost:5432/{DB_NAME}?sslmode=prefer")));
    }

    info!("Starting PostgreSQL container via rustainers...");

    let runner = Runner::podman().context("Failed to create Podman runner")?;

    let image_name = ImageName::new_with_tag(DB_IMAGE, DB_TAG);
    let mut image = rustainers::images::GenericImage::new(image_name);
    image.add_env_var("POSTGRES_DB", DB_NAME);
    image.add_env_var("POSTGRES_USER", DB_USER);
    image.add_env_var("POSTGRES_PASSWORD", DB_PASSWORD);
    image.add_port_mapping(5432);
    image.set_container_name(DB_CONTAINER_NAME);

    let container = runner
        .start(image)
        .await
        .context("Failed to start PostgreSQL container")?;

    let host_port = container
        .host_port(5432)
        .await
        .context("Failed to get host port from container")?;

    let db_url = format!(
        "postgresql://{DB_USER}:{DB_PASSWORD}@localhost:{host_port}/{DB_NAME}?sslmode=disable"
    );

    info!("Container started on host port {host_port}, waiting for PostgreSQL...");
    wait_for_postgres(&db_url).await?;
    info!("PostgreSQL is ready: {}", db_url);

    Ok(db_url)
}

async fn wait_for_postgres(url: &str) -> Result<()> {
    let config = wasi_pg_client::Config::from_uri(url).context("invalid database URL")?;
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(60);

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("PostgreSQL did not become ready within 60s");
        }
        match wasi_pg_client::Connection::connect(&config).await {
            Ok(mut conn) => match conn.query("SELECT 1").await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    debug!("query failed: {e}, retrying...");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            },
            Err(e) => {
                debug!("connect failed: {e}, retrying...");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

// ─── RustFS ─────────────────────────────────────────────────────────────────

async fn start_rustfs() -> Result<String> {
    let exists = check_container_running(RUSTFS_CONTAINER_NAME).await;
    if exists {
        info!("RustFS container already running");
        return Ok(std::env::var("STORAGE_ENDPOINT")
            .unwrap_or_else(|_| format!("http://127.0.0.1:{}", RUSTFS_INTERNAL_PORT)));
    }

    info!("Starting RustFS container via rustainers...");

    let runner = Runner::podman().context("Failed to create Podman runner")?;

    let mut image = rustainers::images::GenericImage::new(
        ImageName::new(RUSTFS_IMAGE),
    );
    image.add_port_mapping(RUSTFS_INTERNAL_PORT);
    image.set_container_name(RUSTFS_CONTAINER_NAME);

    let container = runner
        .start(image)
        .await
        .context("Failed to start RustFS container")?;

    let host_port = container
        .host_port(RUSTFS_INTERNAL_PORT)
        .await
        .context("Failed to get RustFS host port")?;

    // RustFS takes a few seconds to become ready
    tokio::time::sleep(Duration::from_secs(3)).await;

    let endpoint = format!("http://127.0.0.1:{host_port}");
    info!("RustFS ready at {}", endpoint);
    Ok(endpoint)
}

// ─── Container Helpers ──────────────────────────────────────────────────────

async fn check_container_running(container_name: &str) -> bool {
    let output = Command::new("podman")
        .args(["ps", "--filter", &format!("name={container_name}"), "--format", "{{.Names}}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains(container_name)
        }
        _ => false,
    }
}

// ─── Backend Build & Start ──────────────────────────────────────────────────

async fn build_backend() -> Result<()> {
    info!("Building backend (this may take a while on first run)...");
    let status = Command::new("cargo")
        .args(["build", "-p", BACKEND_PKG, "--release", "--quiet"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to build backend")?
        .wait()
        .await?;

    if !status.success() {
        anyhow::bail!(
            "backend build failed (exit code: {:?})",
            status.code()
        );
    }
    info!("Backend built.");
    Ok(())
}

async fn start_backend(db_url: &str, rustfs_endpoint: &str, port: u16) -> Result<()> {
    info!("Starting backend server on port {port}...");

    let log_dir = std::env::temp_dir().join("djmx-dev-logs");
    tokio::fs::create_dir_all(&log_dir).await.ok();

    let db_url = db_url.to_string();
    let port_str = port.to_string();

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("-p")
        .arg(BACKEND_PKG)
        .arg("--release")
        .arg("--quiet")
        .env("DATABASE_URL", &db_url)
        .env("PORT", &port_str)
        .env("STORAGE_ENDPOINT", rustfs_endpoint)
        .env("STORAGE_ACCESS_KEY", RUSTFS_ACCESS_KEY)
        .env("STORAGE_SECRET_KEY", RUSTFS_SECRET_KEY)
        .env("STORAGE_REGION", RUSTFS_REGION)
        .env("STORAGE_BUCKET", "portfolio")
        .env("CORS_ORIGINS", format!("http://localhost:{port},http://127.0.0.1:{port}"))
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("failed to start backend")?;

    let stderr = child.stderr.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    tokio::spawn(stream_logs("backend", stderr, log_dir.clone()));
    tokio::spawn(stream_logs("backend-out", stdout, log_dir));

    let health_url = format!("http://127.0.0.1:{port}/health");
    wait_for_http(&health_url, 30).await?;

    info!("Backend server ready at http://localhost:{port}");
    Ok(())
}

// ─── WASM Start ─────────────────────────────────────────────────────────────

async fn start_wasmtime(db_url: &str, rustfs_endpoint: &str, port: u16) -> Result<()> {
    let port_str = port.to_string();
    let db_url_env = format!("DATABASE_URL={db_url}");
    let rustfs_env = format!("STORAGE_ENDPOINT={rustfs_endpoint}");

    let wasm_path = format!("target/wasm32-wasip2/release/{}", BACKEND_PKG.replace('-', "_"));
    let wasm_path = format!("{wasm_path}.wasm");

    info!("Starting WASM component via wasmtime serve on port {port}...");

    let mut cmd = Command::new("wasmtime");
    cmd.arg("serve")
        .arg("--wasi")
        .arg("inherit-network")
        .arg("--wasi")
        .arg("inherit-env")
        .arg("--env")
        .arg(format!("PORT={port_str}"))
        .arg("--env")
        .arg(&db_url_env)
        .arg("--env")
        .arg(&rustfs_env)
        .arg("--env")
        .arg(format!("STORAGE_ACCESS_KEY={RUSTFS_ACCESS_KEY}"))
        .arg("--env")
        .arg(format!("STORAGE_SECRET_KEY={RUSTFS_SECRET_KEY}"))
        .arg("--env")
        .arg(format!("STORAGE_REGION={RUSTFS_REGION}"))
        .arg("--env")
        .arg("STORAGE_BUCKET=portfolio")
        .arg("--env")
        .arg("RUST_LOG=info")
        .arg(&wasm_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("failed to start wasmtime serve")?;

    let log_dir = std::env::temp_dir().join("djmx-dev-logs");
    tokio::fs::create_dir_all(&log_dir).await.ok();

    let stderr = child.stderr.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    tokio::spawn(stream_logs("wasm", stderr, log_dir.clone()));
    tokio::spawn(stream_logs("wasm-out", stdout, log_dir));

    let health_url = format!("http://127.0.0.1:{port}/health");
    wait_for_http(&health_url, 30).await?;

    info!("WASM backend ready at http://localhost:{port}");
    Ok(())
}

// ─── Shutdown ───────────────────────────────────────────────────────────────

async fn shutdown_all() {
    // Kill backend process
    kill_process_by_name(BACKEND_PKG).await;

    // Kill wasmtime
    let _ = Command::new("pkill")
        .args(["-f", "wasmtime serve"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    // Remove containers
    let _ = Command::new("podman")
        .args(["rm", "-f", DB_CONTAINER_NAME])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    let _ = Command::new("podman")
        .args(["rm", "-f", RUSTFS_CONTAINER_NAME])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn pick_port(fallback: u16) -> u16 {
    portpicker::pick_unused_port().unwrap_or(fallback)
}

async fn is_port_open(port: u16) -> bool {
    tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .is_ok()
}

async fn kill_process_by_name(name: &str) {
    let _ = Command::new("pkill")
        .args(["-f", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
}

async fn wait_for_http(url: &str, timeout_secs: u64) -> Result<()> {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("HTTP service did not become ready within {}s at {url}", timeout_secs);
        }
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            Ok(resp) => {
                debug!("health check returned {} for {url}, retrying...", resp.status());
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                debug!("health check failed for {url}: {e}, retrying...");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

async fn stream_logs(
    name: &str,
    reader: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    log_dir: std::path::PathBuf,
) {
    let log_path = log_dir.join(format!("{name}.log"));
    let mut lines = BufReader::new(reader).lines();
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
        .await
        .ok();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(ref mut f) = file {
            use tokio::io::AsyncWriteExt;
            let _ = f.write_all(format!("{line}\n").as_bytes()).await;
        }
    }
}
