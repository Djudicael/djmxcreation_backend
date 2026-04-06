# djmxcreation — Portfolio & CMS Backend

A Rust backend for a personal art-portfolio website with a companion CMS.  
This is a one-person, in-progress project that serves as both a practical portfolio
and a playground for experimenting with Rust, WASM/WASI, and self-hosted infrastructure.

> **Status:** Work-in-progress — functional for core portfolio management,
> incomplete for authentication, observability, and WASM deployment.

---

## Quick start

Run backend and frontend from the repository root.

### 1) Backend

```bash
# Configure environment (first time)
cp .env.template .env

# Start API
cargo run --bin djmxcreation-backend-axum
```

### 2) Frontend

```bash
# Start admin + portfolio dev servers
npm run dev

# Or build both
npm run build
```

Frontend command details and API base URL overrides are documented in [front/README.md](front/README.md).

---

## What this application does

The backend exposes a JSON REST API used by two frontends:

| Frontend              | Purpose                                                            |
| --------------------- | ------------------------------------------------------------------ |
| **Portfolio website** | Public-facing site displaying projects, about-me, and contact info |
| **CMS admin**         | Private panel for creating and managing portfolio content          |

### Core features

| Domain            | Capabilities                                                                                                                                           |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Projects**      | Create / update / delete projects; upload multiple media files per project; set a thumbnail; filter by visibility and adult-content flag; pagination   |
| **Spotlight**     | Pin up to one spotlight entry per project (featured on the homepage)                                                                                   |
| **About me**      | Manage first name, last name, bio (rich JSON), and profile photo                                                                                       |
| **Contact**       | Store and expose contact information (rich JSON)                                                                                                       |
| **Storage**       | All media files are stored in an S3-compatible object store (RustFS); presigned URLs are returned to clients so files are served directly from storage |
| **Observability** | `/ping` health check; `/metrics` Prometheus endpoint                                                                                                   |

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   djmxcreation-backend                  │
│                                                         │
│  ┌──────────────┐   ┌─────────────────┐                 │
│  │  Axum Router │──▶│ Service layer   │                 │
│  │  (HTTP API)  │   │ (business logic)│                 │
│  └──────────────┘   └────────┬────────┘                 │
│                              │                          │
│               ┌──────────────┴─────────────┐            │
│               ▼                            ▼            │
│     ┌──────────────────┐      ┌────────────────────┐    │
│     │  PostgreSQL repo │      │  Storage repo      │    │
│     │  (deadpool-pg)   │      │  (aws-sdk-s3 →     │    │
│     └────────┬─────────┘      │   RustFS)          │    │
│              │                └─────────┬──────────┘    │
└──────────────┼──────────────────────────┼───────────────┘
               ▼                          ▼
        PostgreSQL                     RustFS
         (media                  (S3-compatible
         metadata)               object store)
```

### Workspace crates

| Crate                       | Role                                                    |
| --------------------------- | ------------------------------------------------------- |
| `djmxcreation-backend-axum` | Binary — HTTP server, routing, middleware               |
| `app-service`               | Business logic services                                 |
| `repository`                | PostgreSQL repositories + S3 storage client             |
| `app_core`                  | Shared DTOs, view models, and repository/service traits |
| `app_config`                | Configuration loading from environment variables        |
| `app-error`                 | Unified error type                                      |
| `migration`                 | (legacy) SQL migration runner via sqlx                  |
| `test-util`                 | Test helpers (testcontainers)                           |

A **Layered Architecture** is used so that each crate can be reused independently.
`app_core` is intentionally shared across all layers to avoid duplicating DTOs.

---

## API surface

All routes are prefixed with `/api`.

### Projects — `/api/portfolio`

| Method   | Path                                       | Description                             |
| -------- | ------------------------------------------ | --------------------------------------- |
| `POST`   | `/v1/projects`                             | Create a project                        |
| `GET`    | `/v1/projects`                             | List all projects (full)                |
| `GET`    | `/v2/projects?page=&size=&adult=&visible=` | Paginated project list with filters     |
| `GET`    | `/v1/projects/:id`                         | Get a single project with all its media |
| `PUT`    | `/v1/projects/:id`                         | Update project metadata                 |
| `DELETE` | `/v1/projects/:id`                         | Delete a project and its media          |
| `PATCH`  | `/v1/projects/:id/contents`                | Upload media files (multipart)          |
| `DELETE` | `/v1/projects/:id/contents/:content_id`    | Remove a media file                     |
| `PUT`    | `/v1/projects/:id/thumbnails/:content_id`  | Set the project thumbnail               |
| `GET`    | `/v1/projects/spotlights`                  | List spotlight entries                  |
| `POST`   | `/v1/projects/spotlights`                  | Add a spotlight                         |
| `GET`    | `/v1/projects/spotlights/:id`              | Get a spotlight                         |
| `DELETE` | `/v1/projects/spotlights/:id`              | Remove a spotlight                      |

### About me — `/api/about`

| Method   | Path               | Description                      |
| -------- | ------------------ | -------------------------------- |
| `GET`    | `/v1/me`           | Get profile info                 |
| `PUT`    | `/v1/me/:id`       | Update profile info              |
| `POST`   | `/v1/me/:id/image` | Upload profile photo (multipart) |
| `DELETE` | `/v1/me/:id`       | Delete profile photo             |

### Contact — `/api/contact`

| Method | Path                  | Description                |
| ------ | --------------------- | -------------------------- |
| `GET`  | `/v1/information`     | Get contact information    |
| `PUT`  | `/v1/information/:id` | Update contact information |

### Observability

| Method | Path       | Description                     |
| ------ | ---------- | ------------------------------- |
| `GET`  | `/ping`    | Health check — returns `200 OK` |
| `GET`  | `/metrics` | Prometheus metrics              |

---

## Technology stack

| Layer               | Technology                                            |
| ------------------- | ----------------------------------------------------- |
| Language            | Rust (edition 2024)                                   |
| HTTP framework      | Axum 0.8                                              |
| Database            | PostgreSQL via `tokio-postgres` + `deadpool-postgres` |
| Database migrations | Embedded via `refinery`                               |
| Object storage      | **RustFS** (S3-compatible) via `aws-sdk-s3`           |
| Metrics             | `metrics` + `metrics-exporter-prometheus`             |
| Logging             | `tracing` + `tracing-subscriber`                      |
| WASM target         | WASI/Wasmer (in progress — see below)                 |

### Infrastructure (self-hosted, no cloud)

- **Hypervisor:** [Cloud Hypervisor](https://www.cloudhypervisor.org/)
- **Platform:** Bare-metal / WasmCloud (evaluating)
- **Object storage:** [RustFS](https://github.com/rustfs/rustfs) — an open-source, S3-compatible storage server written in Rust

---

## Configuration

Copy `.env.template` to `.env` and fill in the values.

| Variable             | Required | Default     | Description                     |
| -------------------- | -------- | ----------- | ------------------------------- |
| `PG_HOST`            | ✅        | —           | PostgreSQL host                 |
| `PG_PORT`            | ✅        | —           | PostgreSQL port                 |
| `PG_DB`              | ✅        | —           | Database name                   |
| `PG_USER`            | ✅        | —           | Database user                   |
| `PG_PASSWORD`        | ✅        | —           | Database password               |
| `PG_APP_MAX_CON`     |          | `5`         | Connection pool size            |
| `STORAGE_ENDPOINT`   | ✅        | —           | RustFS / S3 endpoint URL        |
| `STORAGE_ACCESS_KEY` | ✅        | —           | Storage access key              |
| `STORAGE_SECRET_KEY` | ✅        | —           | Storage secret key              |
| `STORAGE_REGION`     |          | `us-east-1` | Storage region                  |
| `STORAGE_BUCKET`     |          | `portfolio` | Bucket name                     |
| `USERNAME_APP`       | ✅        | —           | Basic-auth username (API)       |
| `PASSWORD_APP`       | ✅        | —           | Basic-auth password (API)       |
| `PORT`               |          | `8081`      | HTTP server port                |
| `RUST_LOG`           |          | `info`      | Log filter (tracing-subscriber) |

---

## Running locally

### Prerequisites

- Rust (stable, 1.80+)
- PostgreSQL 12+
- A running [RustFS](https://github.com/rustfs/rustfs) or MinIO instance

### Steps

```bash
# 1. Clone and enter the project
git clone <repo>
cd djmxcreation_backend

# 2. Configure environment
cp .env.template .env
# Edit .env with your values

# 3. Build and run
cargo run --bin djmxcreation-backend-axum
```

Database migrations are applied automatically on startup.

### Frontend (from repo root)

You can run frontend commands directly from the repository root:

```bash
npm run dev
npm run build
npm run build:admin
npm run build:portfolio
npm run test
```

These scripts proxy to the frontend workspace in [front/](front/).

### Frontend API URL behavior

The frontend no longer hardcodes a backend host. By default, requests stay relative (for example `/api/...`).

If needed, override API base URL via:

1. `globalThis.__DJMX_API_BASE_URL__`
2. `<meta name="djmx-api-base-url" content="...">`
3. `BACKEND_API_URL` at build/runtime

---

## WASM / WASI target (in progress)

The goal is to compile the backend to `wasm32-wasi` so it can run on
[Wasmer](https://wasmer.io/) or [WasmCloud](https://wasmcloud.com/).

The workspace root `Cargo.toml` patches `tokio`, `hyper`, and `socket2` with
WASI-ready forks from [second-state](https://github.com/second-state):

```toml
[patch.crates-io]
tokio   = { git = "https://github.com/second-state/wasi_tokio.git",  branch = "v1.40.x" }
socket2 = { git = "https://github.com/second-state/socket2.git",     branch = "v0.5.x"  }
hyper   = { git = "https://github.com/second-state/wasi_hyper.git",  branch = "v0.14.x" }
```

> **Note:** Full WASM compilation is not yet verified. The patches currently cover
> hyper 0.14.x; axum 0.8 uses hyper 1.x which will need a separate patch once
> second-state publishes one.  
> Tracking this as a future milestone.

---

## Security

Currently the API is protected by HTTP Basic Auth (username/password from env).  
The full security model will use:

1. **API gateway** — only the gateway knows the API key; internal traffic is trusted.
2. **Keycloak** — for user-facing authentication (planned, not yet implemented).

---

## Roadmap

- [x] Project CRUD with media upload
- [x] Spotlight management
- [x] About me and contact endpoints
- [x] PostgreSQL migrations (embedded, auto-run on startup)
- [x] Prometheus metrics
- [x] Structured logging with `tracing`
- [x] RustFS / S3-compatible storage with path-style addressing
- [ ] Public endpoint — only visible projects with content (for the portfolio frontend)
- [ ] Thumbnail auto-generation on upload
- [ ] Keycloak / JWT authentication
- [ ] OpenAPI (Swagger) documentation
- [ ] Full test suite using testcontainers
- [ ] WASM/WASI compilation (blocked on hyper 1.x WASI patch)
- [ ] CMS admin frontend improvements
- [ ] Adult-content flag UI in the admin panel

---

## Project layout

```
djmxcreation_backend/
├── crates/
│   ├── djmxcreation-backend-axum/   # HTTP server binary
│   ├── app-service/                 # Business logic
│   ├── repository/                  # DB + storage access
│   ├── app_core/                    # Shared types & traits
│   ├── app_config/                  # Configuration
│   ├── app-error/                   # Error types
│   ├── migration/                   # (legacy) sqlx migrations
│   └── test-util/                   # Test helpers
├── sql/migrations/                  # SQL migration files (V1..Vn)
├── front/admin/                     # CMS admin frontend (native web components)
├── front/portfolio/                 # Public portfolio frontend (native web components)
├── front/shared/                    # Shared frontend runtime/build modules
├── docs/                            # Architecture diagrams
├── .env.template                    # Environment variable template
└── Dockerfile                       # Multi-stage production build
```
