---
name: wasi-app-architecture
description: Target architecture of DJMXCreation — one Rust codebase that runs either as native binaries or as WASI 0.3 (P3 async) components on the wasm32-wasip2 target, with PostgreSQL through the latest wasi-pg-client, RustFS/S3 through aws-sdk-s3, a media worker, and two lit-html frontends (admin, portfolio). Use when adding a crate or feature, deciding where code belongs, choosing a library, refactoring the workspace toward this layout, or reviewing a change for structure. Start here; it routes to the specialised wasi-* skills.
---

# DJMXCreation architecture

This skill is the entry point for every structural decision in this repository.
It describes the **target** architecture. New code follows it fully. Existing
code is changed pragmatically: [the refactor map](references/refactor-map.md)
lists what already complies, what must change (the P3 migration), what is
cheap and worth doing, and what to leave alone unless you are already
editing that file. Do not refactor working code only to match this skill.

Goal: code that is **maintainable and readable by a person**. Choose the boring,
explicit option. Prefer small, flat modules. Avoid clever generics.

## Non-negotiable rules

1. **Two runtimes, one codebase.** Every server crate builds as a native
   binary (tokio) **and** as a WASI 0.3 (P3) component (`wasm32-wasip2`).
   Both are valid ways to run the application. Business code does not know
   which one it runs on.
2. **Crate names describe the role, never the runtime.** No `wasi`, `wasm`,
   or `native` in crate names (`app-api`, `frontend-server`, `media-worker`,
   `app-http`). Runtime differences live inside crates, behind `cfg`.
3. **Library choices are fixed** (table below). Do not add an alternative
   (`sqlx`, `tokio-postgres`, `wstd`, `wstd-axum`, `wasi` 0.14 bindings,
   `reqwest` on wasm, `rust-s3`, Lit, React, …) without an ADR in `docs/adr/`.
4. **Dependencies point inward:** `api → service → core ← repository`.
   `app_core` has no I/O and no framework dependencies.
5. **Versions live in the root `[workspace.dependencies]`.** Crates write
   `dep = { workspace = true }` and add only features.
6. **Platform differences use `cfg`, never runtime checks:**
   `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(target_arch = "wasm32"))]`
   and target-specific dependency tables. Use this one predicate everywhere;
   do not mix in `target_os = "wasi"`. Keep `cfg` blocks in a few adapter
   places (`app-http`, entry points, the S3 client builder), not spread
   through handlers and services.
7. **One async runtime per process.** Native uses tokio. A WASI component uses
   the component-model P3 executor from `wasip3`: HTTP serving, outbound HTTP,
   clocks, and `wasi-pg-client` sockets all share it. Outbound HTTP, sleeps,
   and timeouts always go through `app-http`, which picks the right
   implementation per target. Never block on a future or start a second
   runtime.
8. **Stateless request handling.** No filesystem, process spawning,
   background threads, or "work after the response" in server crates, even
   on native, so that both runtimes behave the same. Durable or background
   work belongs to the media worker (triggered over HTTP) or to the database.
9. **Pure-Rust crypto only** (`getrandom` 0.4, `sha2`, `jsonwebtoken` with
   `rust_crypto`, `rustls` on native). Never link native TLS into a
   component; outbound HTTPS from a component goes through the host's
   `wasi:http`.
10. **Never return secrets or internal errors to clients.** Log details with
    `tracing`; return a stable, generic message.

## Fixed library choices

| Concern | WASI component (wasm32-wasip2, P3) | Native binary |
|---|---|---|
| HTTP server | `axum` 0.8 (`default-features = false`) + `wasip3::exports::http::handler::Guest`, bridged by `app_http::handle` | `axum` + `tokio` (`server::starter::run_tcp`) |
| Async runtime | component-model P3 executor (`wasip3`, `http-compat` feature) | `tokio` |
| Outbound HTTP, sleep, timeout | `app_http::{send, read_body, sleep, timeout}` → `wasi:http` + P3 clocks | same API → `reqwest` (rustls) + tokio |
| PostgreSQL | `wasi-pg-client`, **latest release** (0.2.1+, P3 async sockets) | `wasi-pg-client` + `tokio-transport` |
| Object storage | `aws-sdk-s3` (`default-features = false`) + `WasiHttpClient` connector over `app-http` | `aws-sdk-s3` default features |
| Image processing | `image`, `fast_image_resize`, `jpeg-encoder` (no default features, pure Rust) | same |
| Errors | `app_core::Error` (`thiserror`), `anyhow` only in binaries and tooling | same |
| Serialization | `serde`, `serde_json` | same |
| IDs / time | `uuid` (v4, serde), `chrono` (serde) | same |
| Logging / metrics | `tracing` + `tracing-subscriber` (env-filter) | same, plus Prometheus `/metrics` |
| Async traits | `async-trait` | same |
| Auth (resource server) | `jsonwebtoken` (`rust_crypto`), JWKS fetched with `app_http` | same |
| Test containers | — | `rustainers` (Podman) |
| HTTP tests | — | `axum-test` or `tower::ServiceExt::oneshot` |
| Frontend | native Custom Elements + **`lit-html`** (not Lit), `esbuild` | same build |
| Frontend e2e | — | Playwright |

## Workspace shape (target)

```text
djmxcreation_backend/
├── Cargo.toml                 # workspace, [workspace.dependencies], wasi-release profile
├── rust-toolchain.toml        # pinned channel + wasm32-wasip2 target
├── .cargo/config.toml         # wasm32-wasip2 runner with P3 flags
├── AGENTS.md                  # points agents to these skills
├── crates/
│   ├── app_core/              # domain: DTOs, views, ports (traits), Error, pure logic
│   ├── app_config/            # typed config loaded from env
│   ├── repository/            # Postgres + S3 adapters implementing core traits; fakes behind `test-fakes`
│   ├── app-service/           # use cases implementing core service traits
│   ├── app-api/               # API server: routers, middleware, auth, composition root
│   ├── app-http/              # runtime adapters: handle() (wasm), send(), read_body(), sleep(), timeout()
│   ├── frontend-server/       # serves the embedded admin and portfolio builds
│   ├── media-worker/          # optimises images, triggered over HTTP
│   ├── migration/             # native migration runner (lib + bin)
│   ├── dev-server/            # native CLI: start / stop / destroy the local stack
│   └── test-util/             # native test harness: shared Podman containers
├── sql/migrations/            # V<n>__<UPPER_SNAKE>.sql
├── front/                     # admin + portfolio + shared (see wasi-frontend)
├── deploy/                    # platform manifests, compose files for infra
└── docs/adr/                  # architecture decision records
```

The existing names `app_core`, `app_config`, `repository`, `app-service`,
`migration`, `dev-server`, and `test-util` stay. New crates use kebab-case
`app-<role>` or `<role>` names (see the naming table in the workspace
layout).

Read [workspace layout](references/workspace-layout.md) for each crate's
responsibilities, allowed dependencies, and module layout. Read
[toolchain and Cargo](references/cargo-and-toolchain.md) before editing a
manifest, `rust-toolchain.toml`, or `.cargo/config.toml`.

## Which skill to use

| Task | Skill |
|---|---|
| HTTP routes, handlers, middleware, auth, config, composition root, entry points, `app-http` | `wasi-backend-api` |
| Tables, queries, transactions, migrations | `wasi-postgres` |
| Buckets, uploads, presigned URLs, RustFS, the S3 connector | `wasi-object-storage` |
| Admin or portfolio pages, components, routing, API calls in the browser | `wasi-frontend` |
| Serving the built frontends from `frontend-server` | `wasi-frontend-packaging` |
| Tests, fakes, Podman harness, verification commands, CI | `wasi-testing` |
| The CLI that starts, stops, or destroys the full local environment | `wasi-dev-stack` |
| Running the local stack (operational) | `run-djmx-stack` |
| Dependency and toolchain upgrades | `maintain-djmxcreation` |

## Adding a feature end to end

Work from the inside out. Each step compiles before the next one:

1. **core**: add the DTO, the view (if an API shape changes), and the
   `I<Name>Repository` / `I<Name>Service` traits with their `DynI…` aliases.
2. **migration**: add `sql/migrations/V<next>__<WHAT>.sql`.
3. **repository**: add the entity and the Postgres/S3 implementation, then add
   or update the fake (behind `test-fakes`).
4. **service**: implement the use case against the traits only. Unit-test it
   with fakes.
5. **api**: add a router module, register it in the composition root, and
   map errors.
6. **front**: add a service function, then a page/component, then the route.
7. **verify** with the `wasi-testing` checklist: native tests **and** the
   wasm32 builds.

## Deployment model

Each server (API, frontend server with one instance per `FRONTEND_APP`, and
media worker) is deployed either as a WASI P3 component (`cdylib`) or as a
native binary (`bin`) from the same crate. Deployment, networking, and
secrets belong to the platform. Read [deployment](references/deployment.md)
before writing a manifest or adding configuration that differs between
environments.

## Review checklist

- Does `app_core` still compile without I/O crates (no `wasi-pg-client`,
  `aws-sdk-s3`, `axum`, `wasip3`)?
- Is every new dependency declared in `[workspace.dependencies]`, and is it
  wasm-safe or gated to native?
- Did the change add `cfg(target_arch …)` outside the adapter places? Move it
  into `app-http` or the entry point.
- Does `cargo tree -i wasip3 --target wasm32-wasip2` show exactly **one**
  `wasip3` version (the one `wasi-pg-client` uses)?
- Do both `cargo build -p app-api` and
  `cargo build --target wasm32-wasip2 -p app-api --lib` succeed?
- Are SQL statements parameterised (`$1`, `$2`)? Are user inputs validated at
  the router boundary?
- Are error responses generic, with details logged?
- Do new frontend files follow the lit-html component pattern with no new
  framework?
