# Refactor map: what to change, and what to leave alone

This file records how the current workspace compares to the target in
`SKILL.md`, and **which gaps are worth closing**. Be pragmatic: the codebase
works and is tested. Refactor only where it unlocks a goal, fixes a real
risk, or costs almost nothing. Everything else is changed only when you are
already editing that file for another reason, or left as is.

Update this file as items land. Delete it when tier 1 and tier 2 are done.

## Already compliant (do not touch)

- Layering: `app_core` (ports, DTOs, views) → `repository` / `app-service` →
  API, with `DynI…` trait objects and a single `ServiceRegister`.
- Repository fakes with probes behind `test-fakes`; service tests use them.
- Podman-backed tests through `test-util` (`shared_postgres`,
  `shared_rustfs`, one container per process, `sslmode=disable`).
- `aws-sdk-s3` + custom connector design, presigned reads, the direct
  multipart upload flow, provisioning-only bucket/CORS changes.
- Parameterised SQL, SQLSTATE error mapping, name-based row mapping.
- `{"errors":{"message":[…]}}` error shape, input validation in
  `validation.rs`, body limits.
- OIDC resource-server auth (`auth.rs`): discovery, JWKS cache, algorithm
  checks, role check.
- `build_router_with_auth(config, verifier)` already gives tests an explicit
  config entry point (it plays the role of `build_router_with`).
- `frontend-wasi`: generated asset table, SPA fallback, 404 rules, tests.
- Frontends: Custom Elements + lit-html + esbuild, shared router/api config,
  node tests, browser workflows through the stack.
- `dev-server`: isolation per checkout, labels, locking, PID-reuse check,
  phases, logs, cleanup, smoke and browser tests; already serves components
  with the P3 Wasmtime flags.
- Migration runner: numeric ordering, duplicate detection, `_migrations`
  tracking.

## Tier 1 — required (blocks the latest `wasi-pg-client`)

`wasi-pg-client` 0.2.1+ only works on the P3 executor, so the P2 (`wstd`)
code must go. The scope is small and well bounded:

| Where | Change |
|---|---|
| new `crates/app-http` | Add the crate from `wasi-backend-api/references/app-http.md`. |
| `djmxcreation-backend-wasi/src/lib.rs`, `frontend-wasi/src/lib.rs`, `media-worker/src/lib.rs` | Replace `#[wstd_axum::http_server]` with the P3 `Guest` export (one block each). Native `main.rs` files stay. |
| `repository/src/config/wasi_http_client.rs` | Rewrite on `app_http::send` (≈220 → ≈90 lines; see `wasi-object-storage`). |
| `repository/src/config/storage.rs` | `WasiSleep` → `app_http::sleep`. |
| `djmxcreation-backend-wasi/src/auth.rs` | Replace the `wstd`/`reqwest` fetch pair with one `app_http` fetch. |
| `repository/src/optimization_repository.rs` (worker trigger) | Same: one `app_http` call instead of `wstd`/`reqwest`. |
| Manifests, `.cargo/config.toml` | Remove `wstd`, `wstd-axum`, `wasi` 0.14, `tokio_unstable`; add `wasip3`, `futures-concurrency`; bump `wasi-pg-client` and drop the `=0.2.0` pin. |
| `docs/adr/` | Short ADR: P3 executor, `app-http`, native and WASI as equal runtimes. Note that ADR 0001's tokio/home patches no longer exist. |

Before starting, confirm that the deployment host (Wasm Cloud Platform)
runs WASI 0.3 components. Without that, tier 1 would break the WASI
deployment, and only the native form would still deploy.

Done when: `cargo tree -i wstd --target wasm32-wasip2` is empty,
`cargo tree -i wasip3 --target wasm32-wasip2` shows one version, and
`dev-server smoke` passes in `--mode wasi` and `--mode native`.

## Tier 2 — worth doing (cheap, real benefit)

| Item | Why | Size |
|---|---|---|
| `app_config`: `dotenv` → `dotenvy`, native only | `dotenv` is unmaintained (RUSTSEC advisory); `.env` loading means nothing in a component | 2 lines |
| `djmxcreation-backend-wasi/tests/e2e_tests.rs`: drop `env::set_var`, call `build_router_with_auth(config, …)` | `set_var` races between parallel tests (flaky failures) | 1 test file |
| `migration`: `pg_advisory_lock` + each file and its `_migrations` row in one transaction | today a failing file can be half-applied and two runners can race | ≈20 lines |
| Media env reads (`app_core::optimization`, `optimization_repository`) → `app_config` | core reads env, so tests cannot set these per test; do it together with tier 1 since the same files change | small |
| Delete dead code: `crates/home-patch`, `test-util/src/minio.rs`, `front/*/app/http/httpInspiration.js`, the unused `testcontainers` dev-dependency, unused error variants `IO`, `BucketCreation`, `PublicBucketAcl` | nothing references them; they mislead readers and agents | deletions |

## Tier 3 — only when you are already editing that code

| Gap | Why it can wait |
|---|---|
| Routers use `Extension(service)` instead of `State` (≈45 sites) | Works; the only cost is that a missing layer fails at runtime, and the route tests catch that. Convert a router when you change it. |
| Error enum: `app-error` as a separate crate, variant names (`EntityNotFound`, `StorageUpload`, …), no `http_status()` | Purely structural; merging touches ≈32 files for no behaviour change. Add new variants in the target style; keep the crate. |
| `auth.rs` and server `main.rs` files read env directly instead of `app_config` | Correct and isolated; move when touching them. |
| Frontend folder layout (`api/`, `http/`, `controller/`, `lib/`), `BaseHttp` class, no `BaseComponent` | 76 working files covered by browser workflows. New pages follow the skill; old ones move only when rewritten. |
| `GET /health/ready`, API security headers | Features, not refactors. Add when the platform needs a readiness probe or a security review asks for them. |
| Hashed assets + immutable caching, CSP header in the frontend server | Performance and defence-in-depth; `no-store` is safe today. |

## Tier 4 — optional (cosmetic, wide ripple)

| Item | Cost | Recommendation |
|---|---|---|
| Rename `djmxcreation-backend-wasi` → `app-api`, `djmxcreation-frontend-wasi` → `frontend-server`, `djmxcreation-media-worker` → `media-worker` | Artifact names change: `deploy/wcp`, CI, release workflow, `dev-server`, README, two skills, and the platform's deployed app definitions | Do it as one pure-move commit when convenient, ideally right after tier 1 (the same manifests and `dev-server` lists are already open). Skip it if it would disturb a running deployment. |
| `dev-server destroy`, `infra`, `runtime-test` | New features | Add when someone needs them; `smoke --mode wasi` covers P3 validation meanwhile. |

## Rules for agents

- Never mix a rename or move with a behaviour change in one commit.
- Do not start a tier 3 or tier 4 item unless the user asks, or you are
  already changing that code for another reason.
- Keep the workspace green between items: run the `wasi-testing` checklist.
