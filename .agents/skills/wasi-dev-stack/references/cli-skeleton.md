# `dev-server` CLI skeleton

The current crate (`crates/dev-server`) already implements most of this.
Use this skeleton when you restructure it, and apply the items marked
**TODO** (not implemented yet).

## Crate

```toml
[package]
name = "dev-server"
version.workspace = true
edition.workspace = true
publish = false

[[bin]]
name = "dev-server"
path = "src/main.rs"

[dependencies]
app_config = { workspace = true }
repository = { workspace = true }     # reuse storage provisioning
migration = { workspace = true }      # reuse run_migrations
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "process", "signal", "time", "io-util"] }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
dotenvy = { workspace = true }
reqwest = { workspace = true, features = ["form"] }   # smoke checks, Hub admin API, proxy
axum = { workspace = true, features = ["tokio", "http1"] }   # proxy
uuid = { workspace = true }
sha2 = { workspace = true }
base64 = { workspace = true }
```

Argument parsing is a small hand-written `Options::parse`. Switch to `clap`
only when the command surface outgrows a single `match`.

## Modules

```text
src/
├── main.rs      # resolve repo root, parse options, dispatch (migrate handled here)
├── stack.rs     # Options, Runtime, Credentials, state dir, Runner, execute()
├── podman.rs    # TODO (optional split): container(), volume(), list_by_label(), remove_by_label()
├── proxy.rs     # same-origin reverse proxy (axum + reqwest)
└── hub.rs       # OIDC realm/client/role/user provisioning + HTTP smoke checks
```

## `main.rs`

```rust
//! Local DJMXCreation stack CLI.
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
```

## Persisted types

```rust
/// Generated once per checkout; deleted only by `destroy`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,            // admin test user (default admin@portfolio.local)
    pub reader_email: String,     // ordinary test user
    pub password: String,         // shared test-user password
    database_password: String,
    storage_secret: String,
    encryption_key: String,       // Hub encryption key
    pairwise_salt: String,        // Hub pairwise subject salt
    #[serde(default)]
    media_processing_token: String,
}

/// Written atomically on every phase change; read by status/stop/test.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Runtime {
    pid: u32,
    start_time: String,          // /proc/<pid>/stat field 22, guards against PID reuse
    pub phase: String,           // starting | <step text> | ready | stopping | stopped | failed
    pub mode: String,            // wasi | native
    pub urls: BTreeMap<String, String>,
    ports: BTreeMap<String, u16>,
    #[serde(default)]
    error: Option<String>,
}
```

Keep these helpers small and tested: `state_dir(root) -> (PathBuf, tag)`,
`save()` (tmp + rename + 0600), `load()`, `process_start(pid)`,
`alive(&Runtime)`, `private_file()`, `credentials(dir)` (create if missing),
`normalize_path()` (`X:\path` → `/mnt/x/path`), `strings()`, `shell_quote()`,
`termination()`.

## Runner

```rust
struct Worker { label: String, child: Child }

struct Runner {
    root: PathBuf,
    dir: PathBuf,              // state dir
    tag: String,
    state: Runtime,
    children: Vec<Worker>,     // start order; cleanup walks it in reverse
    containers: Vec<String>,   // started this run
    proxies: Vec<JoinHandle<()>>,
}

impl Runner {
    fn phase(&mut self, phase: &str) -> Result<()>;                         // update + save runtime.json
    fn spawn(&mut self, label: &str, args: &[String], cwd: &Path,
             env: &BTreeMap<String, String>) -> Result<usize>;              // log to <label>.log, process_group(0)
    async fn run(&mut self, label: &str, args: &[String], cwd: &Path,
                 env: &BTreeMap<String, String>) -> Result<()>;             // spawn + wait; error names the log
    async fn npm(&mut self, label: &str, args: &[&str]) -> Result<()>;      // bash -lc with nvm
    async fn wait(&mut self, url: &str, worker: Option<usize>) -> Result<()>;     // poll URL; fail if child exited
    async fn build(&mut self, skip: bool) -> Result<()>;
    async fn container(&mut self, suffix: &str, image: &str, port: u16,
                       mount: &str, extra: &[String]) -> Result<u16>;       // returns mapped host port
    fn application(&mut self, label: &str, package: &str, port: u16,
                   env: &BTreeMap<String, String>) -> Result<usize>;        // wasmtime serve or native bin
    async fn prepare(&mut self, skip: bool) -> Result<()>;                  // whole start sequence
    async fn cleanup(&mut self);
}
```

### `container()` with a labelled volume

```rust
async fn container(&mut self, suffix: &str, image: &str, port: u16, mount: &str, extra: &[String]) -> Result<u16> {
    let name = format!("djmx-stack-{}-{suffix}", self.tag);
    let label = format!("dev.djmx.stack={}", self.tag);

    // Refuse to replace a container we do not own.
    let inspect = Command::new("podman").args(["container", "inspect", &name]).output().await?;
    if inspect.status.success() {
        let info: Value = serde_json::from_slice(&inspect.stdout)?;
        ensure!(
            info[0]["Config"]["Labels"]["dev.djmx.stack"].as_str() == Some(&self.tag),
            "refusing to replace unowned container {name}"
        );
        Self::output(&strings(&["podman", "rm", "-f", &name]), None).await?;
    }

    // TODO: labelled volume so `destroy` can find it. Creating an existing volume fails; ignore that.
    let volume = format!("djmx-stack-{}-{suffix}", self.tag);
    let _ = Self::output(&strings(&["podman", "volume", "create", "--label", &label, &volume]), None).await;

    self.containers.push(name.clone());
    let mut args = strings(&[
        "podman", "run", "-d", "--name", &name, "--label", &label,
        "-p", &format!("127.0.0.1::{port}"),
        "-v", &format!("{volume}:{mount}"),
    ]);
    args.extend_from_slice(extra);
    args.push(image.into());
    self.run(&format!("container-{suffix}"), &args, &self.root.clone(), &BTreeMap::new()).await?;

    let mapped = Self::output(&strings(&["podman", "port", &name, &port.to_string()]), None).await?;
    Ok(mapped.lines().next().context("container port")?
        .rsplit_once(':').context("mapped port")?.1.parse()?)
}
```

### `application()` in WASI mode

```rust
let args = vec![
    self.wasmtime_binary(), "serve".into(),
    "--addr".into(), format!("127.0.0.1:{port}"),
    "-W".into(), "component-model-async=y".into(),
    "-S".into(), "p3=y".into(),
    "-S".into(), "cli=y".into(),
    "-S".into(), "inherit-env=y".into(),
    "-S".into(), "inherit-network=y".into(),
    "-S".into(), "tcp=y".into(),
    "-S".into(), "allow-ip-name-lookup=y".into(),
    artifact.display().to_string(),
];
```

### `cleanup()`

```rust
async fn cleanup(&mut self) {
    for proxy in self.proxies.drain(..) { proxy.abort(); let _ = proxy.await; }
    for worker in self.children.iter_mut().rev() {
        if let Some(pid) = worker.child.id() {
            let _ = Command::new("kill").args(["-TERM", "--", &format!("-{pid}")]).status().await;
            if tokio::time::timeout(Duration::from_secs(10), worker.child.wait()).await.is_err() {
                let _ = Command::new("kill").args(["-KILL", "--", &format!("-{pid}")]).status().await;
                let _ = worker.child.wait().await;
            }
        }
    }
    for name in self.containers.iter().rev() {
        let _ = Self::output(&strings(&["podman", "rm", "-f", name]), None).await;
    }
}
```

## Supervisor (`serve`)

```rust
async fn serve(mut runner: Runner, skip: bool, one_shot: bool) -> Result<()> {
    runner.state.pid = std::process::id();
    runner.state.start_time = process_start(runner.state.pid).context("process identity")?;
    runner.phase("preparing")?;
    let result = tokio::select! { r = runner.prepare(skip) => r, _ = termination() => Ok(()) };
    let result = if result.is_ok() && runner.state.phase == "ready" && !one_shot {
        loop {
            tokio::select! {
                _ = termination() => break Ok(()),
                _ = sleep(Duration::from_secs(1)) => {
                    let exited = runner.children.iter_mut().find_map(|w| match w.child.try_wait() {
                        Ok(Some(_)) => Some(w.label.clone()),
                        _ => None,
                    });
                    if let Some(label) = exited {
                        break Err(anyhow!("{label} exited; inspect its log"));
                    }
                }
            }
        }
    } else {
        result
    };
    runner.phase("stopping")?;
    runner.cleanup().await;
    match &result {
        Err(e) => { runner.state.error = Some(format!("{e:#}")); runner.phase("failed")?; }
        Ok(()) => runner.phase("stopped")?,
    }
    result
}
```

`smoke` = `serve(runner, skip, true)` after `prepare()` has run the checks.

## `execute()` dispatch

```rust
pub async fn execute(root: PathBuf, options: Options) -> Result<()> {
    ensure!(cfg!(target_os = "linux"), "Run this CLI in WSL/Linux");
    let (dir, tag) = state_dir(&root)?;
    let lock = File::create(dir.join("control.lock"))?;
    lock.lock()?;
    let runtime = dir.join("runtime.json");
    let previous = load::<Runtime>(&runtime).ok();

    match options.command.as_str() {
        "start" | "smoke" | "infra" => { /* ensure !alive(previous); compute ports (+offset);
                                            save Runtime{phase:"starting"}; --detach → re-exec `_serve`
                                            with process_group(0), stdout/stderr → launcher.log;
                                            else drop(lock) and serve(...) */ }
        "_serve"  => { /* drop(lock); serve(Runner::new(..., previous?), skip, false) */ }
        "stop"    => stop(&runtime, previous).await,
        "destroy" => destroy(&dir, &tag, &runtime, previous, options.yes).await,   // TODO
        "status"  => { /* human or --json */ }
        "logs"    => { /* launcher.log or <service>.log */ }
        "test"    => { /* require alive && phase == "ready"; hub::smoke(); --browser → Playwright */ }
        "runtime-test" => { /* TODO: build + run the app-http smoke component */ }
        _ => bail!("Usage: cargo run -p dev-server -- {{start|stop|destroy|status|logs|test|smoke|runtime-test|infra|migrate}} \
                    [--mode wasi|native] [--detach] [--skip-build] [--port-offset N] [--with-media-worker] \
                    [--hub-dir PATH] [--browser] [--json] [--yes]"),
    }
}
```

Parse aliases in `Options::parse`: `up` → `start`, `down` → `stop`,
`db-only` → `infra`, `wasm` → `start --mode wasi`, `native` →
`start --mode native`.

## `destroy()` (TODO)

```rust
async fn destroy(dir: &Path, tag: &str, runtime: &Path, previous: Option<Runtime>, yes: bool) -> Result<()> {
    let label = format!("label=dev.djmx.stack={tag}");
    let containers = list(&["podman", "ps", "-a", "--filter", &label, "-q"]).await?;
    let mut volumes = list(&["podman", "volume", "ls", "--filter", &label, "-q"]).await?;
    // Legacy unlabelled volumes: exact names only, never a wildcard.
    for suffix in ["db", "rustfs"] {
        let name = format!("djmx-stack-{tag}-{suffix}");
        if !volumes.contains(&name) && podman_ok(&["podman", "volume", "exists", &name]).await {
            volumes.push(name);
        }
    }

    println!("Containers: {containers:?}\nVolumes: {volumes:?}\nState dir: {}", dir.display());
    if !yes {
        bail!("Dry run only. Re-run with --yes to permanently delete the resources above");
    }

    if let Some(state) = previous.filter(alive) {
        stop_running(runtime, &state).await?;           // same code path as `stop`
    }
    for id in &containers { podman(&["podman", "rm", "-f", id]).await?; }
    for v in &volumes { podman(&["podman", "volume", "rm", "-f", v]).await?; }

    // Delete the state dir only if it really is ours.
    let base = state_base()?;
    ensure!(
        dir.starts_with(&base) && dir.file_name() == Some(format!("djmx-stack-{tag}").as_ref()),
        "refusing to delete unexpected path {}", dir.display()
    );
    fs::remove_dir_all(dir)?;
    println!("Environment destroyed");
    Ok(())
}
```

Check the real volume suffixes in `container()` calls before writing the
legacy list. The control lock file lives inside the state dir; removing it
while locked is fine on Linux (the lock is released when the process exits).

## Proxy (`proxy.rs`)

- `backend_path(path, hub)` decides the upstream: on app origins `/api/`,
  `/ping`, and `/health*` → API, everything else → frontend server; on the
  Hub origin, the Hub's own API/console split.
- Strip hop-by-hop headers in both directions; forward method, body, query,
  and cookies; preserve status codes.
- Bind to `127.0.0.1:<public port>`; keep the task handle in
  `runner.proxies`.

## Unit tests to keep

```rust
#[test] fn process_identity_rejects_reused_pid() { /* Runtime with wrong start_time → !alive */ }
#[test] fn windows_paths_are_normalized_for_wsl() { /* X:\work\repo → /mnt/x/work/repo */ }
#[tokio::test] async fn cleanup_stops_owned_process_group_and_leaves_unrelated_process_running() { … }
#[tokio::test] async fn proxy_routes_protocol_and_api_calls_preserving_cookies_and_status() { … }
#[test] fn destroy_refuses_paths_outside_state_base() { … }   // TODO
```

## README (`crates/<crate>/README.md`)

Document the prerequisites (WSL/Linux, Podman, minimum Wasmtime with the
P3 snapshot, Node, a Hub checkout), default URLs and ports,
`--port-offset`, what runs in each mode, the state dir contents
(`runtime.json`, `credentials.json`, `*.log`), and the difference between
`stop` (keeps data) and `destroy --yes` (deletes data). Use relative or
placeholder paths only.
