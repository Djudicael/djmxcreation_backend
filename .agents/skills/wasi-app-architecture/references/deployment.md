# Deployment model

## Artifacts

Each server (API, frontend server, media worker) can be deployed in either
form, built from the same crate:

- a **WASI 0.3 (P3) component** exporting the async `wasi:http/handler`, run
  by a WASI host (Wasmtime or the Wasm Cloud Platform);
- a **native binary** (tokio), run as a process or container.

Both read the same environment variables and behave the same. Pick the form
per environment; mixing them (for example a native API with a WASI frontend
server) is fine.

```bash
npm --prefix front ci && npm --prefix front/admin ci && npm --prefix front/portfolio ci
npm --prefix front run build                      # frontend-server embeds front/*/dist

# native binaries
cargo build --locked --release -p app-api -p frontend-server -p media-worker
# → target/release/app-api, frontend-server, media-worker
#   (they listen on BIND_ADDRESS:PORT)

# WASI components
cargo build --locked --profile wasi-release --target wasm32-wasip2 \
  -p app-api -p frontend-server -p media-worker --lib
# → target/wasm32-wasip2/wasi-release/app_api.wasm
# → target/wasm32-wasip2/wasi-release/frontend_server.wasm   (FRONTEND_APP=admin|portfolio)
# → target/wasm32-wasip2/wasi-release/media_worker.wasm
```

Run one component locally (Wasmtime with P3 support; `dev-server` checks the
minimum version):

```bash
wasmtime serve --addr 127.0.0.1:8081 \
  -W component-model-async=y -S p3=y -S cli=y -S inherit-network=y \
  -S inherit-env=y -S tcp=y -S allow-ip-name-lookup=y \
  target/wasm32-wasip2/wasi-release/app_api.wasm
```

`-S tcp=y` and `-S allow-ip-name-lookup=y` are required for PostgreSQL;
`-S inherit-network=y` lets the component reach the database and storage.

## Host requirement (WASI form)

P3 components need a host that implements WASI 0.3 for the exact `wasip3`
snapshot in `Cargo.lock` (component-model async, `wasi:http` 0.3, P3
sockets and clocks). Before switching a deployed component from P2 to P3,
confirm that the target platform's runtime supports it. Keep the Wasmtime
version used by `dev-server`, CI, and the platform in step.

## Wasm Cloud Platform manifest

Application manifests live in `deploy/wcp/` as `*.example.toml` files.
Example (media worker):

```toml
[app]
name = "media-worker"
version = "v1"
namespace = "portfolio"
wasm_artifact = "./target/wasm32-wasip2/wasi-release/media_worker.wasm"
wasm_bind_port = 8080

[fuel]
quota = 10_000_000_000
memory_pages = 8192
max_instances = 1
idle_timeout_secs = 60

[placement]
policy = "every_node"

[policy.network]
allow_inbound = true
allow_outbound_tcp = true
allow_dns = true
allowed_cidrs = ["10.0.0.0/8"]      # explicit DB / storage ranges only

[env]
MEDIA_OPTIMIZATION_ENABLED = "true"
STORAGE_ENDPOINT = "https://storage.example.com"
STORAGE_BUCKET = "portfolio"
STORAGE_REGION = "us-east-1"

[secrets]
DATABASE_URL = { ref = "portfolio-database-url" }
STORAGE_ACCESS_KEY = { ref = "portfolio-storage-access-key" }
STORAGE_SECRET_KEY = { ref = "portfolio-storage-secret-key" }
MEDIA_PROCESSING_TOKEN = { ref = "portfolio-media-processing-token" }
```

Rules:

- Secrets are **always** `[secrets]` references, never `[env]` literals.
  Commit `*.example.toml`, never a real manifest with values.
- Use explicit, narrow `allowed_cidrs` for PostgreSQL and object storage.
- Service-to-service calls use the platform's internal routing where
  available. Do not hard-code IPs.
- Apply database migrations **before** deploying a server version that
  needs them. Servers never run migrations at startup, in either form.
- Production object storage (bucket creation, CORS, lifecycle rules) is
  provisioning work. A request handler never changes bucket policy.
- When an artifact name changes, update `deploy/wcp/`, the CI and release
  workflows, `dev-server`, and the README in the same change.

## Topology

One public origin behind a reverse proxy or gateway:

- `/api/*`, `/ping`, `/health*` → `app-api`
- everything else → `frontend-server` (`FRONTEND_APP=portfolio` on the
  public site, `FRONTEND_APP=admin` on the admin host)
- `media-worker` is internal only; it is never exposed publicly.

Same origin keeps the CSP `connect-src 'self'` working and avoids CORS for
the browser apps. Media is served from object storage through presigned URLs,
so the storage origin is added to the frontend CSP (`img-src`, `media-src`,
`connect-src` for direct uploads).

## Configuration across environments

The same `.wasm` or binary runs everywhere. Everything environment-specific
comes from environment variables and secrets (`.env.template` lists them);
native binaries never depend on a `.env` file in production. The
frontends use relative `/api/...` URLs by default, so one build works behind
any reverse proxy (see `wasi-frontend`).

## Infrastructure for local development

`deploy/db`, `deploy/rustfs` (compose files) can run PostgreSQL and RustFS
under Podman for manual work, but the supported path is `dev-server` (see
`wasi-dev-stack`). Tests start their own containers through `rustainers`;
they never depend on compose stacks being up.
