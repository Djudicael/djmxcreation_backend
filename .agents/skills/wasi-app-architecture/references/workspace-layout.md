# Workspace layout and crate responsibilities

Crate names describe the role, never the runtime: no `wasi`, `wasm`, or
`native` in a crate name, because every server crate runs both natively and
as a WASI component. The existing names stay (`app_core`, `app_config`,
`repository`, `app-service`, `migration`, `dev-server`, `test-util`); new
crates use kebab-case (`app-api`, `app-http`, `frontend-server`,
`media-worker`). Use `edition = "2024"` and inherit `version`, `edition`,
`rust-version`, and `license` from `[workspace.package]`.

## Dependency direction

```text
        ┌───────────┐   ┌──────────────┐
        │  app-api  │   │ media-worker │   (servers; composition roots)
        └──┬─────┬──┘   └──────┬───────┘
           │     │             │
  ┌────────▼──┐ ┌▼─────────────▼─┐
  │app-service│ │   repository   │
  └────────┬──┘ └┬───────────────┘
           │     │
        ┌──▼─────▼──┐   ┌────────────┐
        │ app_core  │   │ app_config │
        └───────────┘   └────────────┘

  app-http ← app-api, frontend-server, media-worker,
             repository (S3 connector on wasm, outbound calls)
```

- `app_core` depends on no other workspace crate.
- `app-service` depends on `app_core` only (dev-dependencies may include
  `repository` with `features = ["test-fakes"]` and `test-util`).
- `repository` depends on `app_core`, `app_config`, and `app-http`.
- `app-api` and `media-worker` are the only crates that know every
  layer. They wire concrete repositories into services.
- `frontend-server` depends on no domain crate.

If you need an arrow that is not shown here, the code is in the wrong crate.

## `app_core` — domain

Pure Rust: no network, database, or HTTP framework. Allowed dependencies:
`serde`, `serde_json`, `uuid`, `chrono`, `thiserror`, `async-trait`,
`tracing`, and pure utility crates.

```text
src/
├── lib.rs              # pub mod list + `pub use error::Error`
├── error.rs            # the application Error enum (thiserror)
├── dto/                # <thing>_dto.rs  — internal transfer objects
├── view/               # <thing>_view.rs — public JSON shapes (serde, camelCase)
├── mapper/             # dto → view helpers that need more than From
├── <feature>/          # one folder per bounded feature: project, about_me, contact,
│   ├── mod.rs          #   home, spotlight, media, storage, optimization
│   ├── <feature>_repository.rs   # trait I<Feature>Repository + DynI alias
│   └── <feature>_service.rs      # trait I<Feature>Service + DynI alias
└── traits/             # cross-cutting ports: Clock, … (only when logic depends on them)
```

Trait conventions:

```rust
pub type DynIProjectRepository = Arc<dyn IProjectRepository + Send + Sync>;

#[async_trait]
pub trait IProjectRepository {
    async fn get_project_by_id(&self, id: Uuid) -> Result<Option<ProjectDto>, Error>;
}
```

- A query for an entity that may not exist returns `Result<Option<T>, Error>`.
  The service decides whether absence becomes `Error::NotFound`.
- Abstract time and randomness behind a trait (`Clock` with `SystemClock`
  and a fixed test clock) whenever logic depends on them.
- DTOs and views are plain structs. Derive `Default` and use a builder-style
  chain (`ProjectDto::new().id(..).visible(..)`) only when a struct has many
  optional fields. Otherwise use a struct literal.
- Views use `#[serde(rename_all = "camelCase")]` because the frontend is
  JavaScript. Map at the boundary with `impl From<Dto> for View`.
- Use `#[serde(skip_serializing)]` on every secret-bearing field.
- Configuration-dependent decisions (for example "is media optimisation
  enabled", "is this internal token valid") take their inputs as arguments.
  Core never reads environment variables.

### Error type

A single `thiserror` enum in `app_core::error`. Variants describe **what
happened**, not which HTTP status to return:

```rust
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    #[error("not found: {0}")]          NotFound(String),
    #[error("invalid input: {0}")]      InvalidInput(String),
    #[error("conflict: {0}")]           Conflict(String),
    #[error("forbidden")]               Forbidden,
    #[error("database error")]          Database,
    #[error("storage error: {0}")]      Storage(&'static str),   // "upload", "delete", "presign", …
    #[error("internal error")]          Internal,
}

impl Error {
    pub fn http_status(&self) -> u16 {
        match self {
            Error::NotFound(_) => 404,
            Error::InvalidInput(_) => 400,
            Error::Conflict(_) => 409,
            Error::Forbidden => 403,
            Error::Database | Error::Storage(_) | Error::Internal => 500,
        }
    }
}
```

The API crate's mapping is then a single function. Do not depend on `axum`
here, and do not wrap `std::io::Error` (components have no filesystem).

## `app_config` — configuration

- `Config::from_env() -> Result<Config, ConfigError>` reads environment
  variables once. Load `.env` with `dotenvy` **on native only**: `.env` files
  do not exist in a WASI deployment.
- Group fields into sub-structs: `DatabaseConfiguration`,
  `StorageConfiguration`, `ServerConfiguration`, `AuthConfiguration`,
  `MediaConfiguration`.
- Required values cause `ConfigError::Missing(&'static str)`. Optional
  values have documented defaults.
- Provide `Config::new(...)` or a builder so tests construct configuration
  **without** `std::env::set_var`, which is racy in parallel tests.
- Document every variable in `.env.template`.

## `repository` — adapters

```text
src/
├── lib.rs
├── config/
│   ├── db.rs               # DatabaseConfig { url } + connect()
│   ├── storage.rs          # StorageClient + get_storage_client() + ensure_bucket() + configure_media_cors()
│   └── wasi_http_client.rs # #[cfg(target_arch = "wasm32")] AWS connector over app-http
├── entity/                 # row-shaped structs + From<Entity> for Dto
├── error.rs                # PgError / SDK errors → app_core::Error
├── <feature>_repository.rs # impl I<Feature>Repository for <Feature>Repository
└── fake_<feature>_repository.rs   # #[cfg(feature = "test-fakes")]
```

- One repository struct per aggregate. It holds `Arc<DatabaseConfig>` and/or
  `StorageClient`, never request data.
- Postgres details are in `wasi-postgres`; S3 details are in
  `wasi-object-storage`.
- Fakes expose a *probe* that records calls so service tests can assert
  side effects (see `wasi-testing`).

## `app-service` — use cases

- `pub struct ProjectService { project_repository: DynIProjectRepository, storage_repository: DynIStorageRepository, bucket: String }`
- `impl IProjectService for ProjectService` lives here. The trait lives in core.
- Optional collaborators use `with_<thing>(mut self, …) -> Self`.
- Services are async, take `&self`, and never touch HTTP types.
- Concurrency: use `futures::future::join_all` to fan out independent
  awaits. Do not spawn tasks.

## `app-api` — the API server

```toml
[lib]
crate-type = ["cdylib", "rlib"]   # cdylib = WASI component, rlib = native bin and tests
[[bin]]
name = "app-api"
path = "src/main.rs"
```

```text
src/
├── lib.rs           # pub mods + app_router() + app_router_with(config) + P3 Guest export (wasm32)
├── main.rs          # #[tokio::main] native entry (cfg not wasm32)
├── server/starter.rs            # build_router(), build_router_with(config), run_tcp()
├── service/service_register.rs  # composition root
├── router/<feature>_router.rs
├── middleware/      # auth, cors, security_headers, metrics (native)
├── error.rs         # ApiError + IntoResponse
├── auth.rs          # bearer-token verification against the OIDC provider
└── validation.rs    # boundary validation helpers
```

Details are in `wasi-backend-api`.

## `app-http` — runtime adapters

The same API on both targets: `send(request)`, `read_body(response,
max_bytes)`, `sleep(duration)`, and `timeout(duration, on_timeout, work)`.
On wasm they use `wasi:http` and the P3 clocks; on native they use `reqwest`
and tokio. `handle(router, request)` (the P3 entry-point bridge) exists on
wasm only. Every outbound call, sleep, and timeout in the workspace goes
through this crate, so callers need no `cfg`. Full source is in
`wasi-backend-api` → [the adapter reference](../../wasi-backend-api/references/app-http.md).

## Additional servers

- **Frontend server** (`frontend-server`): one crate embeds both `admin`
  and `portfolio`; `FRONTEND_APP` selects which one an instance serves. See
  `wasi-frontend-packaging`.
- **Media worker** (`media-worker`): a separate server (`cdylib` + `bin`)
  with its own entry points. The platform (or the API's maintenance route) triggers
  `POST /internal/media/process-due` with the `X-Media-Worker-Key` shared
  token. It claims one job, resizes one variant, stores it, and returns. It
  reuses `app_core`, `app_config`, and `repository`.
- **Migration** (`migration`): native lib + bin. See `wasi-postgres`.
- **Dev environment CLI** (`dev-server`): native CLI that starts the complete
  local environment (Podman infra, migrations, seed data, builds, components
  under Wasmtime or native binaries, the OIDC provider, a same-origin proxy,
  and smoke tests), then stops it (keeping data) or destroys it (wiping data).
  See `wasi-dev-stack`.

## Naming quick reference

| Item | Convention | Example |
|---|---|---|
| Crate | role only, never the runtime; new crates kebab-case | `app-api`, `frontend-server` |
| Artifact | crate name with `_` | `app_api.wasm`, `app-api` binary |
| Module file | snake_case, suffixed by role | `project_repository.rs`, `project_router.rs` |
| Trait | `I` + PascalCase + role | `IProjectRepository` |
| Trait object alias | `DynI…` | `DynIProjectRepository` |
| Concrete adapter | PascalCase + role | `ProjectRepository` |
| Router type | `<Feature>Router` with `new_router(...)` | `ProjectRouter::new_router(register)` |
| SQL migration | `V<n>__<UPPER_SNAKE>.sql` | `V3__CREATE_TABLE_PROJECT.sql` |
| Env var | UPPER_SNAKE, grouped by prefix | `STORAGE_ENDPOINT`, `OIDC_ISSUER` |
| Custom element | `c-<name>` component, `p-<name>` page | `c-drag-drop`, `p-projects` |
