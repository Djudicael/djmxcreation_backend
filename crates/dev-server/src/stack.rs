use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    task::JoinHandle,
    time::{Instant, sleep},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub reader_email: String,
    pub password: String,
    database_password: String,
    storage_secret: String,
    encryption_key: String,
    pairwise_salt: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Runtime {
    pid: u32,
    start_time: String,
    pub phase: String,
    pub mode: String,
    pub hub_dir: PathBuf,
    pub urls: BTreeMap<String, String>,
    ports: BTreeMap<String, u16>,
    #[serde(default)]
    error: Option<String>,
}

pub struct Options {
    pub command: String,
    mode: String,
    hub: PathBuf,
    detach: bool,
    skip: bool,
    offset: u16,
    json: bool,
    browser: bool,
}

impl Options {
    pub fn parse(root: &Path) -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let mut options = Self {
            command: args.next().unwrap_or_else(|| "start".into()),
            mode: "wasi".into(),
            hub: root
                .parent()
                .context("repository parent")?
                .join("openid_connect_wasi"),
            detach: false,
            skip: false,
            offset: 0,
            json: false,
            browser: false,
        };
        if let Ok(hub) = std::env::var("OIDC_HUB_DIR") {
            options.hub = normalize_path(&hub);
        }
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mode" => options.mode = args.next().context("--mode requires wasi or native")?,
                "--hub-dir" => {
                    options.hub = normalize_path(&args.next().context("--hub-dir requires a path")?)
                }
                "--detach" => options.detach = true,
                "--skip-build" => options.skip = true,
                "--port-offset" => {
                    options.offset = args
                        .next()
                        .context("--port-offset requires a number")?
                        .parse()?
                }
                "--json" => options.json = true,
                "--browser" => options.browser = true,
                _ => bail!("Unknown option {arg}"),
            }
        }
        if options.command == "wasm" {
            options.command = "start".into();
            options.mode = "wasi".into();
        }
        if options.command == "native" {
            options.command = "start".into();
            options.mode = "native".into();
        }
        ensure!(
            matches!(options.mode.as_str(), "wasi" | "native"),
            "--mode must be wasi or native"
        );
        Ok(options)
    }
}

fn normalize_path(path: &str) -> PathBuf {
    if path.len() > 2 && path.as_bytes()[1] == b':' {
        PathBuf::from(format!(
            "/mnt/{}/{}",
            path[..1].to_lowercase(),
            path[3..].replace('\\', "/")
        ))
    } else {
        PathBuf::from(path)
    }
}

fn state_dir(root: &Path) -> Result<(PathBuf, String)> {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    root.hash(&mut hash);
    let tag = format!("{:016x}", hash.finish());
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".local/state")
        });
    let path = base.join(format!("djmx-stack-{tag}"));
    fs::create_dir_all(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    }
    Ok((path, tag))
}

fn save<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, serde_json::to_vec_pretty(data)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(temp, path)?;
    Ok(())
}

fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn process_start(pid: u32) -> Option<String> {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()?
        .rsplit_once(')')?
        .1
        .split_whitespace()
        .nth(19)
        .map(String::from)
}
fn alive(state: &Runtime) -> bool {
    !matches!(state.phase.as_str(), "stopped" | "failed")
        && !state.start_time.is_empty()
        && process_start(state.pid).as_ref() == Some(&state.start_time)
}

fn private_file(path: &Path) -> Result<File> {
    let file = File::create(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}
pub fn spare_port() -> Result<u16> {
    Ok(std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port())
}
fn random() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn credentials(dir: &Path) -> Result<Credentials> {
    let path = dir.join("credentials.json");
    if path.exists() {
        return load(&path);
    }
    let credentials = Credentials {
        email: "admin@portfolio.local".into(),
        reader_email: "reader@portfolio.local".into(),
        password: format!("Dev-{}-9aA!", random()),
        database_password: random(),
        storage_secret: random(),
        encryption_key: random(),
        pairwise_salt: random(),
    };
    save(&path, &credentials)?;
    Ok(credentials)
}

struct Worker {
    label: String,
    child: Child,
}
struct Runner {
    root: PathBuf,
    dir: PathBuf,
    tag: String,
    state: Runtime,
    children: Vec<Worker>,
    containers: Vec<String>,
    proxies: Vec<JoinHandle<()>>,
    client: reqwest::Client,
}

impl Runner {
    fn wasmtime_binary(&self) -> String {
        if let Ok(path) = std::env::var("WASMTIME_BIN") {
            return path;
        }
        let portable = self.dir.join("runtime/wasmtime");
        if portable.is_file() {
            portable.to_string_lossy().into()
        } else {
            "wasmtime".into()
        }
    }
    fn new(root: PathBuf, dir: PathBuf, tag: String, state: Runtime) -> Result<Self> {
        Ok(Self {
            root,
            dir,
            tag,
            state,
            children: vec![],
            containers: vec![],
            proxies: vec![],
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(45))
                .build()?,
        })
    }
    fn phase(&mut self, phase: &str) -> Result<()> {
        self.state.phase = phase.into();
        save(&self.dir.join("runtime.json"), &self.state)?;
        println!("{phase}");
        Ok(())
    }
    fn spawn(
        &mut self,
        label: &str,
        command: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
    ) -> Result<usize> {
        let log = private_file(&self.dir.join(format!("{label}.log")))?;
        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .current_dir(cwd)
            .env_remove("RUSTUP_TOOLCHAIN")
            .envs(env)
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .kill_on_drop(true);
        #[cfg(unix)]
        cmd.process_group(0);
        let child = cmd.spawn().with_context(|| format!("start {label}"))?;
        self.children.push(Worker {
            label: label.into(),
            child,
        });
        Ok(self.children.len() - 1)
    }
    async fn run(
        &mut self,
        label: &str,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
    ) -> Result<()> {
        println!(
            "{label}; log: {}",
            self.dir.join(format!("{label}.log")).display()
        );
        let index = self.spawn(label, args, cwd, env)?;
        let status = self.children[index].child.wait().await?;
        self.children.remove(index);
        ensure!(
            status.success(),
            "{label} failed; inspect {}",
            self.dir.join(format!("{label}.log")).display()
        );
        Ok(())
    }
    async fn npm(
        &mut self,
        label: &str,
        args: &[&str],
        cwd: &Path,
        env: &BTreeMap<String, String>,
    ) -> Result<()> {
        // Arguments are fixed launcher commands; shell syntax is used only to load nvm.
        ensure!(
            args.iter().all(|s| s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "/-:.".contains(c))),
            "invalid npm command"
        );
        let script = format!(
            "if [ -f \"$HOME/.nvm/nvm.sh\" ]; then . \"$HOME/.nvm/nvm.sh\"; fi; exec npm {}",
            args.join(" ")
        );
        self.run(label, &["bash".into(), "-lc".into(), script], cwd, env)
            .await
    }
    async fn output(args: &[String], input: Option<&str>) -> Result<String> {
        let mut command = Command::new(&args[0]);
        command
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if input.is_some() {
            command.stdin(Stdio::piped());
        } else {
            command.stdin(Stdio::null());
        }
        let mut child = command.spawn()?;
        if let Some(input) = input {
            child
                .stdin
                .take()
                .context("command stdin")?
                .write_all(input.as_bytes())
                .await?;
        }
        let output =
            tokio::time::timeout(Duration::from_secs(120), child.wait_with_output()).await??;
        ensure!(
            output.status.success(),
            "{} {} failed: {}",
            args[0],
            args.get(1).map(String::as_str).unwrap_or(""),
            String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(300)
                .collect::<String>()
        );
        Ok(String::from_utf8(output.stdout)?.trim().into())
    }
    async fn wait(&mut self, url: &str, worker: Option<usize>) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(120);
        while Instant::now() < deadline {
            if let Some(index) = worker {
                ensure!(
                    self.children[index].child.try_wait()?.is_none(),
                    "{} exited; inspect its log",
                    self.children[index].label
                );
            }
            if let Ok(response) = self.client.get(url).send().await
                && response.status().is_success()
            {
                return Ok(());
            }
            sleep(Duration::from_millis(500)).await;
        }
        bail!("readiness timeout: {url}")
    }
    async fn build(&mut self, skip: bool) -> Result<()> {
        let hub = self.state.hub_dir.clone();
        let root = self.root.clone();
        let env = BTreeMap::from([
            ("OIDC_HUB_URL".into(), self.state.urls["hub"].clone()),
            ("OIDC_REALM".into(), "portfolio".into()),
            ("OIDC_CLIENT_ID".into(), "portfolio".into()),
            ("BACKEND_API_URL".into(), "".into()),
            ("NODE_ENV".into(), "production".into()),
        ]);
        let config = self.dir.join("build-config.json");
        if skip {
            ensure!(
                config.exists() && load::<BTreeMap<String, String>>(&config)? == env,
                "embedded URLs changed; restart without --skip-build"
            );
            return Ok(());
        }
        self.phase("Building frontends and application servers")?;
        for app in ["front", "front/admin", "front/portfolio"] {
            self.npm(
                &format!("npm-ci-{}", app.replace('/', "-")),
                &["--prefix", app, "ci"],
                &root,
                &BTreeMap::new(),
            )
            .await?;
        }
        self.npm("front-build", &["run", "build"], &root, &env)
            .await?;
        self.npm(
            "hub-npm-ci",
            &["--prefix", "front/admin", "ci"],
            &hub,
            &BTreeMap::new(),
        )
        .await?;
        self.npm(
            "hub-front-build",
            &["--prefix", "front/admin", "run", "build"],
            &hub,
            &BTreeMap::new(),
        )
        .await?;
        self.run(
            "hub-build-seed",
            &strings(&["cargo", "build", "--locked", "-p", "oidc-wasm-dev"]),
            &hub,
            &BTreeMap::new(),
        )
        .await?;
        if self.state.mode == "wasi" {
            self.run(
                "build-wasi",
                &strings(&[
                    "cargo",
                    "build",
                    "--locked",
                    "--profile",
                    "wasi-release",
                    "--target",
                    "wasm32-wasip2",
                    "-p",
                    "djmxcreation-backend-wasi",
                    "-p",
                    "djmxcreation-frontend-wasi",
                    "--lib",
                ]),
                &root,
                &BTreeMap::new(),
            )
            .await?;
            self.run(
                "hub-build-wasi",
                &strings(&[
                    "cargo",
                    "build",
                    "--locked",
                    "--release",
                    "--target",
                    "wasm32-wasip2",
                    "-p",
                    "openid-connect-wasi",
                    "-p",
                    "oidc-admin-wasi",
                    "--lib",
                ]),
                &hub,
                &BTreeMap::new(),
            )
            .await?;
        } else {
            self.run(
                "build-native",
                &strings(&[
                    "cargo",
                    "build",
                    "--locked",
                    "-p",
                    "djmxcreation-backend-wasi",
                    "-p",
                    "djmxcreation-frontend-wasi",
                ]),
                &root,
                &BTreeMap::new(),
            )
            .await?;
            self.run(
                "hub-build-native",
                &strings(&[
                    "cargo",
                    "build",
                    "--locked",
                    "-p",
                    "openid-connect-wasi",
                    "-p",
                    "oidc-admin-wasi",
                ]),
                &hub,
                &BTreeMap::new(),
            )
            .await?;
        }
        save(&config, &env)?;
        Ok(())
    }
    async fn container(
        &mut self,
        suffix: &str,
        image: &str,
        port: u16,
        mount: &str,
        extra: &[String],
    ) -> Result<u16> {
        let name = format!("djmx-stack-{}-{suffix}", self.tag);
        let inspect = Command::new("podman")
            .args(["container", "inspect", &name])
            .output()
            .await?;
        if inspect.status.success() {
            let info: Value = serde_json::from_slice(&inspect.stdout)?;
            ensure!(
                info[0]["Config"]["Labels"]["dev.djmx.stack"].as_str() == Some(&self.tag),
                "refusing to replace unowned container {name}"
            );
            Self::output(&strings(&["podman", "rm", "-f", &name]), None).await?;
        }
        self.containers.push(name.clone());
        let mut args = strings(&[
            "podman",
            "run",
            "-d",
            "--name",
            &name,
            "--label",
            &format!("dev.djmx.stack={}", self.tag),
            "-p",
            &format!("127.0.0.1::{port}"),
            "-v",
            &format!("djmx-stack-{}-{suffix}:{mount}", self.tag),
        ]);
        args.extend_from_slice(extra);
        args.push(image.into());
        self.run(
            &format!("container-{suffix}"),
            &args,
            &self.root.clone(),
            &BTreeMap::new(),
        )
        .await?;
        let mapped = Self::output(
            &strings(&["podman", "port", &name, &port.to_string()]),
            None,
        )
        .await?;
        Ok(mapped
            .lines()
            .next()
            .context("container port")?
            .rsplit_once(':')
            .context("mapped port")?
            .1
            .parse()?)
    }
    async fn psql(&self, database: &str, sql: &str) -> Result<String> {
        Self::output(
            &strings(&[
                "podman",
                "exec",
                "-i",
                &self.containers[0],
                "psql",
                "-U",
                "postgres",
                "-d",
                database,
                "-v",
                "ON_ERROR_STOP=1",
                "-At",
            ]),
            Some(sql),
        )
        .await
    }
    fn application(
        &mut self,
        label: &str,
        package: &str,
        port: u16,
        env: &BTreeMap<String, String>,
    ) -> Result<usize> {
        let hub = label.starts_with("hub-");
        let project = if hub {
            self.state.hub_dir.clone()
        } else {
            self.root.clone()
        };
        let (artifact, args) = if self.state.mode == "wasi" {
            let profile = if hub { "release" } else { "wasi-release" };
            let artifact = project.join(format!(
                "target/wasm32-wasip2/{profile}/{}.wasm",
                package.replace('-', "_")
            ));
            let mut args = strings(&[
                &self.wasmtime_binary(),
                "serve",
                "--addr",
                &format!("127.0.0.1:{port}"),
                "-W",
                "component-model-async=y",
                "-S",
                "cli=y",
                "-S",
                "p3=y",
                "-S",
                "inherit-env=y",
                "-S",
                "inherit-network=y",
                "-S",
                "tcp=y",
                "-S",
                "allow-ip-name-lookup=y",
                "--max-instance-reuse-count",
                "1000",
            ]);
            args.push(artifact.to_string_lossy().into());
            (artifact, args)
        } else {
            let artifact = project.join("target/debug").join(package);
            (artifact.clone(), vec![artifact.to_string_lossy().into()])
        };
        ensure!(
            artifact.is_file(),
            "missing {}; restart without --skip-build",
            artifact.display()
        );
        self.spawn(label, &args, &project, env)
    }
    async fn prepare(&mut self, skip: bool) -> Result<()> {
        let root = self.root.clone();
        let hub = self.state.hub_dir.clone();
        if self.state.mode == "wasi" {
            let version = Self::output(&[self.wasmtime_binary(), "--version".into()], None).await?;
            let major = version
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim_start_matches('v')
                .split('.')
                .next()
                .unwrap_or("")
                .parse::<u32>()
                .unwrap_or(0);
            ensure!(
                major >= 50,
                "The current Hub requires a newer WASI 0.3 runtime; use Wasmtime 50+ via WASMTIME_BIN or the stack's runtime/wasmtime portable binary (found: {version})"
            );
        }
        ensure!(
            hub.join("crates/oidc-wasm-dev/Cargo.toml").is_file(),
            "Hub checkout missing: {}",
            hub.display()
        );
        Self::output(&strings(&["podman", "info", "--format", "json"]), None).await?;
        for port in self.state.ports.values() {
            std::net::TcpListener::bind(("127.0.0.1", *port))
                .with_context(|| format!("port {port} is in use"))?;
        }
        self.build(skip).await?;
        let credentials = credentials(&self.dir)?;
        self.phase("Starting isolated PostgreSQL and RustFS")?;
        let db = self
            .container(
                "db",
                "docker.io/library/postgres:17",
                5432,
                "/var/lib/postgresql/data",
                &strings(&[
                    "-e",
                    &format!("POSTGRES_PASSWORD={}", credentials.database_password),
                ]),
            )
            .await?;
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            if Self::output(
                &strings(&[
                    "podman",
                    "exec",
                    &self.containers[0],
                    "pg_isready",
                    "-U",
                    "postgres",
                ]),
                None,
            )
            .await
            .is_ok()
            {
                break;
            }
            ensure!(Instant::now() < deadline, "PostgreSQL readiness failed");
            sleep(Duration::from_millis(500)).await;
        }
        for db in ["hub", "portfolio"] {
            if self
                .psql(
                    "postgres",
                    &format!("SELECT 1 FROM pg_database WHERE datname='{db}'"),
                )
                .await?
                != "1"
            {
                self.psql("postgres", &format!("CREATE DATABASE {db}"))
                    .await?;
            }
        }
        let storage = self
            .container(
                "rustfs",
                "docker.io/rustfs/rustfs:latest",
                9000,
                "/data",
                &strings(&[
                    "-e",
                    "RUSTFS_ACCESS_KEY=djmxlocal",
                    "-e",
                    &format!("RUSTFS_SECRET_KEY={}", credentials.storage_secret),
                ]),
            )
            .await?;
        let storage_url = format!("http://127.0.0.1:{storage}");
        self.wait(&format!("{storage_url}/health"), None).await?;
        let storage_config = app_config::storage_configuration::StorageConfiguration::new(
            storage_url.clone(),
            "djmxlocal".into(),
            credentials.storage_secret.clone(),
            "us-east-1".into(),
            "portfolio".into(),
        );
        let storage_client = repository::config::storage::get_storage_client(storage_config)?;
        if storage_client
            .inner
            .head_bucket()
            .bucket("portfolio")
            .send()
            .await
            .is_err()
        {
            storage_client
                .inner
                .create_bucket()
                .bucket("portfolio")
                .send()
                .await
                .context("create private RustFS bucket")?;
        }
        repository::config::storage::configure_media_cors(
            "portfolio",
            &storage_client,
            vec![
                self.state.urls["admin"].clone(),
                self.state.urls["portfolio"].clone(),
            ],
        )
        .await
        .context("configure direct media upload CORS")?;
        let db_base = format!(
            "postgresql://postgres:{}@127.0.0.1:{db}",
            credentials.database_password
        );
        self.phase("Applying migrations and seeding Hub")?;
        migration::run_migrations(&format!("{db_base}/portfolio?sslmode=disable")).await?;
        let seed_env = BTreeMap::from([
            (
                "OIDC_DATABASE_URL".into(),
                format!("{db_base}/hub?sslmode=disable"),
            ),
            (
                "OIDC_PROXY_PORT".into(),
                self.state.ports["hub"].to_string(),
            ),
            ("DEFAULT_EMAIL".into(), credentials.email.clone()),
            ("DEFAULT_PASSWORD".into(), credentials.password.clone()),
            ("RUST_LOG".into(), "info".into()),
        ]);
        self.run(
            "hub-seed",
            &[
                hub.join("target/debug/oidc-wasm-dev")
                    .to_string_lossy()
                    .into(),
                "seed".into(),
            ],
            &hub,
            &seed_env,
        )
        .await?;
        // The Hub's first-run role migration predates its seed user; grant only
        // our isolated master-realm bootstrap account current admin authority.
        self.psql("hub","INSERT INTO roles(id,realm_id,name,permissions) SELECT gen_random_uuid(),id,'djmx-stack-bootstrap','[\"admin\"]'::jsonb FROM realms WHERE name='master' AND NOT EXISTS(SELECT 1 FROM roles WHERE name='djmx-stack-bootstrap'); INSERT INTO user_roles(user_id,role_id) SELECT u.id,r.id FROM users u JOIN roles r ON r.realm_id=u.realm_id WHERE r.name='djmx-stack-bootstrap' AND u.username='admin' ON CONFLICT DO NOTHING;").await?;
        for (name, algorithm) in [("rsa", "RSA"), ("ed25519", "Ed25519")] {
            let key = self.dir.join(format!("{name}.pem"));
            if !key.exists() {
                let mut args = strings(&["openssl", "genpkey", "-algorithm", algorithm]);
                if algorithm == "RSA" {
                    args.extend(strings(&["-pkeyopt", "rsa_keygen_bits:2048"]));
                }
                args.extend(["-out".into(), key.to_string_lossy().into()]);
                self.run(&format!("generate-{name}"), &args, &root, &BTreeMap::new())
                    .await?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(key, fs::Permissions::from_mode(0o600))?;
                }
            }
        }
        // Hub expects PKCS#1 RSA PEM; OpenSSL genpkey emits PKCS#8.
        let rsa_path = self.dir.join("rsa.pem");
        if fs::read_to_string(&rsa_path)?.contains("BEGIN PRIVATE KEY") {
            let converted = self.dir.join("rsa-pkcs1.pem");
            self.run(
                "convert-rsa-pkcs1",
                &[
                    "openssl".into(),
                    "rsa".into(),
                    "-traditional".into(),
                    "-in".into(),
                    rsa_path.to_string_lossy().into(),
                    "-out".into(),
                    converted.to_string_lossy().into(),
                ],
                &root,
                &BTreeMap::new(),
            )
            .await?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&converted, fs::Permissions::from_mode(0o600))?;
            }
            fs::rename(converted, &rsa_path)?;
        }
        let hb = spare_port()?;
        let hf = spare_port()?;
        let mut hub_env = seed_env;
        hub_env.extend(BTreeMap::from([
            ("OIDC_ISSUER".into(), self.state.urls["hub"].clone()),
            ("OIDC_SERVER_BIND_ADDRESS".into(), "127.0.0.1".into()),
            ("OIDC_SERVER_PORT".into(), hb.to_string()),
            ("OIDC_ADMIN_SERVER_BIND_ADDRESS".into(), "127.0.0.1".into()),
            ("OIDC_ADMIN_SERVER_PORT".into(), hf.to_string()),
            (
                "OIDC_ENCRYPTION_KEY".into(),
                credentials.encryption_key.clone(),
            ),
            (
                "OIDC_PAIRWISE_SALT".into(),
                credentials.pairwise_salt.clone(),
            ),
            (
                "OIDC_SIGNING_KEY".into(),
                fs::read_to_string(self.dir.join("rsa.pem"))?,
            ),
            (
                "OIDC_ED25519_KEY".into(),
                fs::read_to_string(self.dir.join("ed25519.pem"))?,
            ),
            ("OIDC_SIGNING_KID".into(), "djmx-rsa".into()),
            ("OIDC_ED25519_KID".into(), "djmx-ed25519".into()),
            (
                "OIDC_CORS_ORIGINS".into(),
                format!(
                    "{},{}",
                    self.state.urls["admin"], self.state.urls["portfolio"]
                ),
            ),
            ("OIDC_RATE_LIMIT_MAX".into(), "1000".into()),
            ("RUST_LOG".into(), "warn".into()),
        ]));
        self.phase("Starting Hub WASI/native servers")?;
        let worker = self.application("hub-backend", "openid-connect-wasi", hb, &hub_env)?;
        self.wait(&format!("http://127.0.0.1:{hb}/health"), Some(worker))
            .await?;
        let worker = self.application("hub-frontend", "oidc-admin-wasi", hf, &hub_env)?;
        self.wait(&format!("http://127.0.0.1:{hf}/"), Some(worker))
            .await?;
        self.proxies
            .push(crate::proxy::start(self.state.ports["hub"], hf, hb, true).await?);
        self.phase("Provisioning portfolio realm and PKCE client")?;
        crate::hub::provision(
            &self.client,
            &self.state.urls["hub"],
            &self.state.urls["admin"],
            &credentials,
        )
        .await?;
        let port = self.state.ports["api"];
        let backend_env = BTreeMap::from([
            (
                "DATABASE_URL".into(),
                format!("{db_base}/portfolio?sslmode=disable"),
            ),
            ("STORAGE_ENDPOINT".into(), storage_url),
            ("STORAGE_ACCESS_KEY".into(), "djmxlocal".into()),
            (
                "STORAGE_SECRET_KEY".into(),
                credentials.storage_secret.clone(),
            ),
            ("STORAGE_REGION".into(), "us-east-1".into()),
            ("STORAGE_BUCKET".into(), "portfolio".into()),
            ("PORT".into(), port.to_string()),
            ("BIND_ADDRESS".into(), "127.0.0.1".into()),
            (
                "OIDC_ISSUER".into(),
                format!("{}/realms/portfolio", self.state.urls["hub"]),
            ),
            ("OIDC_AUDIENCE".into(), "portfolio-api".into()),
            ("OIDC_ADMIN_CLIENT_ID".into(), "portfolio".into()),
            ("OIDC_ADMIN_ROLE".into(), "admin".into()),
            (
                "CORS_ORIGINS".into(),
                format!(
                    "{},{}",
                    self.state.urls["admin"], self.state.urls["portfolio"]
                ),
            ),
            ("RUST_LOG".into(), "warn".into()),
        ]);
        self.phase("Starting DJMX API, admin and portfolio")?;
        let worker =
            self.application("backend", "djmxcreation-backend-wasi", port, &backend_env)?;
        self.wait(&format!("{}/ping", self.state.urls["api"]), Some(worker))
            .await?;
        for app in ["admin", "portfolio"] {
            let internal = spare_port()?;
            let env = BTreeMap::from([
                ("FRONTEND_APP".into(), app.into()),
                ("PORT".into(), internal.to_string()),
            ]);
            let worker = self.application(app, "djmxcreation-frontend-wasi", internal, &env)?;
            self.wait(&format!("http://127.0.0.1:{internal}/"), Some(worker))
                .await?;
            self.proxies
                .push(crate::proxy::start(self.state.ports[app], internal, port, false).await?);
        }
        self.phase("Testing real Hub authentication and API")?;
        crate::hub::smoke(&self.state, &credentials).await?;
        self.phase("ready")
    }
    async fn cleanup(&mut self) {
        for proxy in self.proxies.drain(..) {
            proxy.abort();
            let _ = proxy.await;
        }
        for worker in self.children.iter_mut().rev() {
            if let Some(pid) = worker.child.id() {
                let _ = Command::new("kill")
                    .args(["-TERM", "--", &format!("-{pid}")])
                    .status()
                    .await;
                if tokio::time::timeout(Duration::from_secs(10), worker.child.wait())
                    .await
                    .is_err()
                {
                    let _ = Command::new("kill")
                        .args(["-KILL", "--", &format!("-{pid}")])
                        .status()
                        .await;
                    let _ = worker.child.wait().await;
                }
            }
        }
        for name in self.containers.iter().rev() {
            let _ = Self::output(&strings(&["podman", "rm", "-f", name]), None).await;
        }
    }
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).into()).collect()
}

async fn termination() {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler");
        tokio::select! {_=tokio::signal::ctrl_c()=>{},_=term.recv()=>{}}
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn serve(mut runner: Runner, skip: bool, one_shot: bool) -> Result<()> {
    runner.state.pid = std::process::id();
    runner.state.start_time = process_start(runner.state.pid).context("Linux process identity")?;
    runner.phase("preparing")?;
    let result = tokio::select! {result=runner.prepare(skip)=>result,_=termination()=>Ok(())};
    let result = if result.is_ok() && runner.state.phase == "ready" && !one_shot {
        loop {
            tokio::select! {_=termination()=>break Ok(()),_=sleep(Duration::from_secs(1))=>{let exited=runner.children.iter_mut().find_map(|worker|match worker.child.try_wait(){Ok(Some(_))=>Some(worker.label.clone()),_=>None}); if let Some(label)=exited {break Err(anyhow::anyhow!("{label} exited; inspect its log"));}}}
        }
    } else {
        result
    };
    runner.phase("stopping")?;
    runner.cleanup().await;
    if let Err(error) = &result {
        runner.state.error = Some(format!("{error:#}"));
        runner.phase("failed")?;
    } else {
        runner.phase("stopped")?;
    }
    result
}

pub async fn execute(root: PathBuf, options: Options) -> Result<()> {
    ensure!(cfg!(target_os = "linux"), "Run this CLI in WSL/Linux");
    let (dir, tag) = state_dir(&root)?;
    let lock = File::create(dir.join("control.lock"))?;
    lock.lock()?;
    let runtime = dir.join("runtime.json");
    let previous = load::<Runtime>(&runtime).ok();
    match options.command.as_str() {
        "start" | "smoke" => {
            ensure!(
                !previous.as_ref().is_some_and(alive),
                "Stack already running; use status or stop"
            );
            let mut ports: BTreeMap<String, u16> = BTreeMap::new();
            for (name, port) in [
                ("hub", 3000u16),
                ("admin", 3008),
                ("portfolio", 3009),
                ("api", 8081),
            ] {
                ports.insert(
                    name.into(),
                    port.checked_add(options.offset)
                        .context("invalid port offset")?,
                );
            }
            let mut state = Runtime {
                mode: options.mode,
                hub_dir: options.hub.canonicalize().context("Hub checkout path")?,
                urls: ports
                    .iter()
                    .map(|(name, port)| (name.clone(), format!("http://localhost:{port}")))
                    .collect(),
                ports,
                phase: "starting".into(),
                ..Runtime::default()
            };
            save(&runtime, &state)?;
            if options.detach {
                ensure!(
                    options.command == "start",
                    "--detach is for start, not smoke"
                );
                let log = private_file(&dir.join("launcher.log"))?;
                let mut cmd = Command::new(std::env::current_exe()?);
                cmd.arg("_serve");
                if options.skip {
                    cmd.arg("--skip-build");
                }
                cmd.current_dir(&root)
                    .stdin(Stdio::null())
                    .stdout(log.try_clone()?)
                    .stderr(log);
                #[cfg(unix)]
                cmd.process_group(0);
                let mut child = cmd.spawn()?;
                let pid = child.id().context("launcher pid")?;
                state.pid = pid;
                state.start_time = process_start(pid).context("launcher process identity")?;
                save(&runtime, &state)?;
                drop(lock);
                for _ in 0..100 {
                    if load::<Runtime>(&runtime).ok().is_some_and(|s| s.pid == pid) {
                        break;
                    }
                    ensure!(
                        child.try_wait()?.is_none(),
                        "Launcher failed; inspect launcher.log"
                    );
                    sleep(Duration::from_millis(100)).await;
                }
                println!(
                    "Stack starting in background; use cargo run -p dev-server -- status\nLogs: {}",
                    dir.display()
                );
                Ok(())
            } else {
                drop(lock);
                serve(
                    Runner::new(root, dir, tag, state)?,
                    options.skip,
                    options.command == "smoke",
                )
                .await
            }
        }
        "_serve" => {
            let state = previous.context("missing runtime state")?;
            drop(lock);
            serve(Runner::new(root, dir, tag, state)?, options.skip, false).await
        }
        "stop" => {
            if let Some(state) = previous.filter(alive) {
                Runner::output(
                    &strings(&["kill", "-TERM", "--", &state.pid.to_string()]),
                    None,
                )
                .await?;
                for _ in 0..480 {
                    let state = load::<Runtime>(&runtime)?;
                    if !alive(&state) || matches!(state.phase.as_str(), "stopped" | "failed") {
                        println!("Stack stopped; persistent volumes and credentials preserved");
                        return Ok(());
                    }
                    sleep(Duration::from_millis(250)).await;
                }
                bail!("Cleanup still running; inspect launcher.log");
            }
            println!("Stack not running");
            Ok(())
        }
        "status" => {
            if let Some(state) = previous {
                if options.json {
                    let mut value = serde_json::to_value(&state)?;
                    value["running"] = json!(alive(&state));
                    value["state_dir"] = json!(dir);
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    println!(
                        "Mode: {}; phase: {}; running: {}",
                        state.mode,
                        state.phase,
                        alive(&state)
                    );
                    for (name, url) in state.urls {
                        println!("{name}: {url}");
                    }
                }
            } else {
                if options.json {
                    println!(
                        "{}",
                        json!({"running":false,"phase":"not started","state_dir":dir})
                    );
                } else {
                    println!("Stack not started");
                }
            }
            if !options.json {
                println!("Logs and credentials: {}", dir.display());
            }
            Ok(())
        }
        "logs" => {
            println!(
                "{}",
                fs::read_to_string(dir.join("launcher.log"))
                    .unwrap_or_else(|_| "No launcher log".into())
            );
            Ok(())
        }
        "test" => {
            let state = previous.context("Start the stack before testing")?;
            ensure!(
                alive(&state) && state.phase == "ready",
                "Stack must be ready before testing"
            );
            crate::hub::smoke(&state, &credentials(&dir)?).await?;
            if options.browser {
                let mut runner = Runner::new(root.clone(), dir.clone(), tag, state)?;
                let env = BTreeMap::from([
                    ("DJMX_STACK_STATE".into(), runtime.to_string_lossy().into()),
                    (
                        "DJMX_STACK_CREDENTIALS".into(),
                        dir.join("credentials.json").to_string_lossy().into(),
                    ),
                ]);
                let script = format!(
                    "if [ -f \"$HOME/.nvm/nvm.sh\" ]; then . \"$HOME/.nvm/nvm.sh\"; fi; exec node {}",
                    shell_quote(&root.join("scripts/stack-browser.mjs").to_string_lossy())
                );
                runner
                    .run(
                        "browser-tests",
                        &["bash".into(), "-lc".into(), script],
                        &root,
                        &env,
                    )
                    .await?;
                println!("Browser tests passed");
            }
            Ok(())
        }
        _ => bail!(
            "Usage: cargo run -p dev-server -- {{start|wasm|native|stop|status|logs|test|smoke}} [--mode wasi|native] [--detach] [--hub-dir PATH] [--skip-build] [--browser]"
        ),
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn process_identity_rejects_reused_pid() {
        let mut state = Runtime {
            pid: std::process::id(),
            start_time: process_start(std::process::id()).unwrap(),
            ..Runtime::default()
        };
        assert!(alive(&state));
        state.start_time.push('0');
        assert!(!alive(&state));
    }
    #[test]
    fn windows_paths_are_normalized_for_wsl() {
        assert_eq!(
            normalize_path("D:\\dev\\openid_connect_wasi"),
            PathBuf::from("/mnt/d/dev/openid_connect_wasi")
        );
    }

    #[tokio::test]
    async fn cleanup_stops_owned_process_group_and_leaves_unrelated_process_running() {
        let dir = std::env::temp_dir().join(format!("djmx-cli-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let mut runner =
            Runner::new(dir.clone(), dir.clone(), "test".into(), Runtime::default()).unwrap();
        let mut unrelated = Command::new("sleep")
            .arg("300")
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        runner
            .spawn(
                "owned",
                &strings(&["sh", "-c", "sleep 300"]),
                &dir,
                &BTreeMap::new(),
            )
            .unwrap();
        runner.cleanup().await;
        assert!(runner.children[0].child.try_wait().unwrap().is_some());
        assert!(unrelated.try_wait().unwrap().is_none());
        unrelated.kill().await.unwrap();
        unrelated.wait().await.unwrap();
        fs::remove_file(dir.join("owned.log")).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
