---
name: wasi-testing
description: Test and verify DJMXCreation's servers in both runtimes (native binaries and WASI 0.3 P3 components) and its lit-html frontends. Use when writing unit tests with repository fakes and probes, repository tests against Podman PostgreSQL/RustFS via rustainers, HTTP router and auth tests, media-worker pipeline tests, P3 runtime checks, node --test or Playwright tests, when running the verification checklist before finishing a change, or when editing CI workflows and Dependabot.
---

# Testing and verification

## Environment

- Run Rust and npm commands in **WSL**, not Windows PowerShell. From
  PowerShell, use `wsl.exe --cd <repository path in WSL> bash -lc '<command>'`
  (convert `X:\…` to `/mnt/x/…`).
- Load nvm before Node commands when nvm is installed: `source ~/.nvm/nvm.sh`.
- Respect `rust-toolchain.toml`.
- Integration tests use **Podman** through `rustainers`. If Podman is
  unavailable, report it as an environment limitation and keep it separate
  from code failures.

## Test pyramid

| Level | Where | Tools | What |
|---|---|---|---|
| Domain unit | `app_core` `#[cfg(test)]` | std | pure logic, `http_status`, conversions, media limits |
| Service unit | `app-service/tests/*.rs` | fakes from `repository` (`test-fakes`) | use-case behaviour and side effects |
| Repository | `repository/tests/*.rs` | `test-util` shared Postgres/RustFS | real SQL, migrations, S3 calls |
| HTTP | `app-api/tests/*.rs` (+ `tests/support/`) | `axum-test` or `tower::ServiceExt::oneshot` + `app_router_with(config)` | routing, status codes, auth, validation, uploads |
| Worker | `media-worker/tests/*.rs` | `test-util` + `axum-test` | claim → resize → store pipeline |
| Frontend server | `frontend-server` `#[cfg(test)]` | `axum-test` | SPA fallback, 404 rules, headers |
| Frontend unit | `front/**/x.test.js` | `node --test` | router, http, api config, media helpers, direct upload |
| P3 runtime | `dev-server runtime-test` | Wasmtime + `http-client-smoke` | P3 HTTP server/client, PostgreSQL sockets, deadlines inside a real component |
| End to end | `scripts/*-workflows.mjs` + `scripts/stack-browser.mjs` | Playwright via `dev-server test --browser` | real flows, PKCE login, uploads, deep links, mobile |

New browser workflows go into `scripts/<area>-workflows.mjs` and are wired
into `stack-browser.mjs`. They clean up the data they create and restore
the profile, contact, and Home settings they change.

## Fakes and probes (service tests)

Fakes live in `repository/src/fake_<feature>_repository.rs` behind
`#[cfg(feature = "test-fakes")]`. A constructor returns `(Dyn…, Probe)`:

```rust
pub fn create_storage_repository(url: &str) -> (DynIStorageRepository, StorageRepositoryProbe)
```

- The fake holds canned responses in an `Arc<State>` and records calls in
  `Mutex<Vec<…>>`.
- The probe exposes the recorded calls (`probe.url_requests()`), so tests
  assert **side effects** (what was uploaded or deleted) as well as return
  values.
- Service tests can run without tokio:
  `futures::executor::block_on(service.method())`.
- Prefer fakes to mocking frameworks.

## Shared containers (`test-util`)

- `shared_postgres() -> (Arc<DatabaseConfig>, String)` and
  `shared_rustfs() -> String` use `tokio::sync::OnceCell`, so there is **one
  container per test process**. They start once, wait for readiness, and run
  migrations once (`migration::run_migrations`).
- Append `sslmode=disable` to the container URL.
- For full isolation in HTTP tests, migrate a template database once, then
  `CREATE DATABASE test_<n> TEMPLATE …` per test app. This avoids
  `TRUNCATE` and locks.
- Tests that share a database create their own rows with fresh UUIDs and
  never assume an empty table.
- Remove stale containers from earlier runs only by exact name or label.

## HTTP tests

- Build the app with `app_router_with(config)`. **Never call
  `std::env::set_var`** in tests; they run in parallel.
- Auth: generate a test RSA/Ed25519 key, serve discovery + JWKS from a local
  stub (or inject the verifier), and sign tokens in the test. Fixtures live
  in `app-api/tests/support/`. Cover: no token (401), wrong audience (401),
  non-admin (403), admin (2xx), JWKS unavailable (503).
- Assert on status, the response shape (`{"errors":{"message":[…]}}`), and
  the absence of internal details (no SQL, no stack traces, no URLs with
  credentials).

## P3 runtime test

Native tests cannot prove the P3 path. After any change to `app-http`,
a component entry point, the S3 connector, or `wasi-pg-client`/`wasip3`
versions, run:

```bash
cargo build --locked -p app-http --example http-client-smoke --target wasm32-wasip2 --release
cargo run --locked -p dev-server -- runtime-test --skip-build
```

Until `runtime-test` exists, run `dev-server smoke` in WASI mode instead.

## Verification checklist (run before reporting done)

```bash
# frontend (when front/ changed, and before any frontend-server build)
source ~/.nvm/nvm.sh
npm --prefix front test
npm --prefix front run build

# rust: quick
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo check --target wasm32-wasip2 -p app-api -p frontend-server -p media-worker --lib
cargo tree -i wasip3 --target wasm32-wasip2 -e normal     # exactly one version

# rust: full (Podman)
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target wasm32-wasip2 -p app-api -p frontend-server -p media-worker -p app-http --lib -- -D warnings
cargo test --workspace
cargo build --locked --release -p app-api -p frontend-server -p media-worker      # native binaries
cargo build --locked --profile wasi-release --target wasm32-wasip2 \
  -p app-api -p frontend-server -p media-worker --lib                            # WASI components

# runtime (when P3, servers, or the stack changed): both modes
cargo run --locked -p dev-server -- smoke --mode wasi
cargo run --locked -p dev-server -- smoke --mode native

# hygiene
git diff --check
git status --short
```

Until the renames land, use the current package names:
`djmxcreation-backend-wasi` (→ `app-api`), `djmxcreation-frontend-wasi`
(→ `frontend-server`), and `djmxcreation-media-worker` (→ `media-worker`).
`app-http` exists only after the P3 migration.
Report the exact commands you ran, which passed, and anything not run and
why. Do not weaken assertions to make a test pass.

## CI (GitHub Actions)

`.github/workflows/ci.yml`:

- **frontend**: `actions/setup-node` (Node 22, npm cache keyed on every
  lockfile: `front/`, `front/admin/`, `front/portfolio/`) → `npm ci` per
  lockfile → `npm audit --audit-level=high` per lockfile → `npm test` →
  `npm run build`.
- **rust**: build the frontends first (embedded assets), install the pinned
  toolchain with `wasm32-wasip2`, `Swatinem/rust-cache`, install Podman, then
  fmt, clippy (native and wasm32), the wasm32 builds, and
  `cargo test --workspace`.
- **runtime**: install a pinned Wasmtime with P3 support (verify its
  checksum), then run `dev-server runtime-test --skip-build` against a
  Postgres service. Update the version and checksum together.
- **security**: `cargo deny check advisories bans licenses sources`.
- `permissions: contents: read`, `concurrency` with `cancel-in-progress`,
  and pinned action major versions.

`.github/workflows/release.yml` builds the `.wasm` artifacts and
`front/<app>/dist` on tags and uploads them.

Dependabot covers GitHub Actions, Cargo, and every independent npm lockfile
directory. Upgrades follow `maintain-djmxcreation`.
