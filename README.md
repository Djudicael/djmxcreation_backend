# DJMXCreation — Portfolio and CMS

A Rust API, public portfolio, and administration interface for managing a personal
art portfolio. The backend and frontend asset server support native execution and
WASI `wasm32-wasip2` HTTP components.

Deployments connect to PostgreSQL, an S3-compatible object store such as RustFS,
and an OpenID Connect identity provider.
[OpenID Connect WASI Hub](https://github.com/Djudicael/OpenID-Connect-WASI-Hub)
is the tested identity provider; the admin uses its Keycloak-compatible endpoints
through `keycloak-js`.

## Deployment

The API and frontends can be deployed independently of the identity provider.
Configure the provider through its public URL and realm issuer. Its source code,
database, and private signing keys are not required to deploy DJMXCreation.

### Required services

| Service | Deployment responsibility |
| --- | --- |
| PostgreSQL | Provision a database and application credentials; apply the versioned migrations before starting the API. |
| S3-compatible storage | Provision a bucket and credentials. The API must reach the endpoint, and browsers must reach the returned presigned media URLs. |
| OpenID Connect provider | Expose Keycloak-compatible realm endpoints for the browser adapter; configure a public PKCE client, exact callback/logout URLs, API audience, and administrator role as described in [Security](#security). |
| HTTP hosting | Host the API and both frontend applications, configure public HTTPS addresses, and supply application settings/secrets through the deployment environment. |

### API settings

Use [.env.template](.env.template) as a configuration reference. Supply real
values through the environment or your hosting platform's secret configuration.
The following addresses and placeholders are illustrative:

```dotenv
DATABASE_URL=postgresql://app:replace-with-password@db.example.com:5432/portfolio
STORAGE_ENDPOINT=https://storage.example.com
STORAGE_ACCESS_KEY=replace-with-access-key
STORAGE_SECRET_KEY=replace-with-secret-key
STORAGE_REGION=us-east-1
STORAGE_BUCKET=portfolio

OIDC_ISSUER=https://identity.example.com/realms/portfolio
OIDC_AUDIENCE=portfolio-api
OIDC_ADMIN_CLIENT_ID=portfolio
OIDC_ADMIN_ROLE=admin
CORS_ORIGINS=https://admin.example.com,https://portfolio.example.com
```

Run the native migration tool with `DATABASE_URL` in its environment and
`sql/migrations/` available relative to its working directory. From this source
checkout:

```bash
cargo run --locked -p migration
```

The API does not run migrations on startup. Migration execution belongs in the
deployment process before application instances begin serving requests.

### Frontend settings

Set the frontend build environment before building assets or the frontend WASI
component:

```dotenv
OIDC_HUB_URL=https://identity.example.com
OIDC_REALM=portfolio
OIDC_CLIENT_ID=portfolio
BACKEND_API_URL=https://api.example.com
```

Register `https://admin.example.com/` as the exact callback and post-logout URL.
Configure the provider's allowed origins for credentialed browser code exchange
and refresh requests. See [Security](#security) for claims and role configuration.

Alternatively, use relative API paths when your gateway routes each frontend's
`/api/*` requests to the API. Frontend OIDC settings are embedded at build time;
rebuild assets and the frontend component when these values change.
[Frontend configuration](front/README.md) describes environment precedence and
runtime API URL overrides.

### Application hosting

Build artifacts and commands are listed in [Builds and tests](#builds-and-tests).
Choose either:

- **Native hosting:** run the API binary under your service supervisor. Serve the
  built admin and portfolio assets with an HTTP server configured to return
  `index.html` for application routes such as `/projects/:id`.
- **WASI hosting:** deploy the API and frontend components on a compatible WASI
  HTTP host. Run two frontend instances with `FRONTEND_APP=admin` and
  `FRONTEND_APP=portfolio`. The API host needs outbound HTTP/network access to the
  issuer/JWKS, storage, and PostgreSQL.

For example, with Wasmtime 50 and the API release artifact, after supplying the
API environment:

```bash
wasmtime serve -W component-model-async=y -S cli=y -S p3=y -S inherit-env=y -S inherit-network=y -S tcp=y -S allow-ip-name-lookup=y --addr 127.0.0.1:8081 djmxcreation-backend-wasip2.wasm
```

This example listens behind an HTTPS gateway; adapt host permissions, routing,
persistence, and supervision to your platform. The repository provides application
artifacts and a development launcher, rather than a turnkey production deployment.

## Admin workflows

The admin interface is in English and uses native Web Components, `lit-html`,
Quill, and `keycloak-js`.

| Area | Workflow |
| --- | --- |
| **Dashboard** | Open Home, project, profile, and contact management; navigation also works on mobile. |
| **Home** | Save an optional short introduction (500 characters maximum). Search projects by title/client and select a cover card to feature a published, non-adult project. Use **Remove selection** to retain the abstract visual. |
| **Projects** | Choose **New project**, enter a title, and create a private draft. Edit the metadata, rich-text description, visibility, and adult-content flag; choose **Save project**. |
| **Media and cover** | Upload images/videos, preview files, select **Set as cover**, and delete media. Media actions preserve text edits in the form. |
| **Publication** | Enable **Publish on portfolio** and save to publish; disable it and save to hide the project. |
| **About me** | Save names and biography; upload/remove the profile photo separately. The public About page displays the saved profile. |
| **Contacts** | Add/edit/remove email, phone, and website/social-link rows and edit the introduction. Choose **Save contacts** to apply changes, including removals. Saved links appear on the public Contact page. |
| **Session** | Sign in through the Hub; use **Sign out** to end the session. Direct links and reloads preserve the requested path, query, and fragment. |

Actions show progress, success, and error feedback. Project/media/photo deletion
asks for confirmation. Admin uploads allow **32 MiB per file**. The project
multipart endpoint limits requests to **50 MiB**; the general API body limit is
**64 MiB**. The admin sends project files in separate requests.

Home settings require migration **V8** and are independent of the About biography.
If the featured project becomes private/adult or loses its image cover, Home hides
it; deleting the project clears its selection.

Contact rows are stored in `description.contacts` alongside the existing Quill
`ops` JSON. Description-only records remain supported; no new migration is needed.
Spotlight management is available through the API; the admin has no spotlight
editor yet. Cover selection is manual; automatic thumbnail generation remains
future work. See [frontend workflows and configuration](front/README.md).

## Architecture

The API exposes an Axum router backed by business services and PostgreSQL/S3
repositories. Identity management is provided by a separately hosted OIDC service.
The frontend component embeds both applications' built assets, selected per
instance with `FRONTEND_APP`. Node/npm are build tools and are not required by the
WASI application servers.

```mermaid
flowchart LR
    Admin["Admin browser"] -->|PKCE| Hub["OpenID Connect WASI Hub"]
    Admin -->|Bearer access token| API["Axum API"]
    Portfolio["Portfolio browser"] -->|Public reads| API
    API --> Services["Business services"]
    Services --> Repositories
    Repositories --> PostgreSQL
    Repositories --> RustFS["RustFS / S3"]
```

Media downloads use presigned URLs directly from storage.

| Crate | Role |
| --- | --- |
| `djmxcreation-backend-wasi` | Axum API: native binary and WASI HTTP component |
| `djmxcreation-frontend-wasi` (`crates/frontend-wasi`) | Native/WASI server embedding both built frontends |
| `dev-server` | Rust stack CLI, local proxies, Hub provisioning, and HTTP checks |
| `app-service` | Business logic |
| `repository` | PostgreSQL via `wasi-pg-client`; S3 via `aws-sdk-s3` |
| `app_core` | DTOs, views, and service/repository traits |
| `app_config` | Environment configuration |
| `app-error` | Shared errors |
| `migration` | Native SQL migration runner using `wasi-pg-client` |
| `test-util` | Podman PostgreSQL/RustFS helpers using `rustainers` |

PostgreSQL connections use `wasi-pg-client`. Shared Rust dependency versions and
the WASI build profile are defined in [Cargo.toml](Cargo.toml). Runtime and
dependency compatibility notes for the development stack are documented in the
[CLI guide](crates/dev-server/README.md).

## API surface

Business routes are prefixed with `/api`. All writes and the full administrative
project list require an administrator access token; public reads are described in
[Security](#security).

### Projects — `/api/portfolio`

| Method   | Path                                       | Description                             |
| -------- | ------------------------------------------ | --------------------------------------- |
| `POST`   | `/v1/projects`                             | Create a private draft                        |
| `GET`    | `/v1/projects`                             | List all projects (full)                |
| `GET`    | `/v2/projects?page=&size=&adult=&visible=` | Paginated project list with filters     |
| `GET`    | `/v1/projects/:id`                         | Get a single project with all its media |
| `PUT`    | `/v1/projects/:id`                         | Update metadata, description, and flags                 |
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
| `GET`  | `/metrics` | Native Prometheus metrics; WASI placeholder |

---

## Configuration reference

The following settings apply to the deployed API. The migration runner records
applied SQL filenames in `_migrations` and applies files in version order.

| Variable | Default / requirement | Description |
| --- | --- | --- |
| `DATABASE_URL` | Set explicitly (recommended) | PostgreSQL connection URI |
| `PG_HOST`, `PG_PORT`, `PG_DB`, `PG_USER`, `PG_PASSWORD` | Legacy fallback if `DATABASE_URL` is absent | Connection URI components; defaults are in `app_config` |
| `STORAGE_ENDPOINT` | Required | RustFS/S3 URL; returned media URLs must be browser-reachable |
| `STORAGE_ACCESS_KEY`, `STORAGE_SECRET_KEY` | Required | Storage credentials |
| `STORAGE_REGION` | `us-east-1` | Storage region |
| `STORAGE_BUCKET` | `portfolio` | Media bucket |
| `OIDC_ISSUER` | Required | Exact Hub realm issuer |
| `OIDC_AUDIENCE` | Required | Dedicated API audience, e.g. `portfolio-api` |
| `OIDC_ADMIN_CLIENT_ID` | `portfolio` | Client whose role claims may grant administration |
| `OIDC_ADMIN_ROLE` | `admin` | Required realm or configured client role |
| `CORS_ORIGINS` | Empty | Comma-separated allowed browser origins |
| `PORT` | `8081` | Native API port; the WASI host owns its socket |
| `BIND_ADDRESS` | `0.0.0.0` | Native bind address; the launcher uses loopback |
| `RUST_LOG` | `info` | Log filter |

API URL overrides are checked in this order: `globalThis.__DJMX_API_BASE_URL__`,
the `djmx-api-base-url` HTML meta tag, then `BACKEND_API_URL`. When using relative
URLs, configure your gateway to proxy API requests. See [front/README.md](front/README.md)
for build environment precedence.

## Builds and tests

Building from source requires Rust 1.97.1 and the `wasm32-wasip2` target from
[rust-toolchain.toml](rust-toolchain.toml), plus Node.js/npm (CI uses Node.js 22).
Run contributor build/test commands on Linux or in WSL on Windows.

Build frontend assets **before** the frontend component or the entire Rust
workspace. The component embeds `front/admin/dist` and `front/portfolio/dist`.

```bash
# Load nvm when installed.
if [ -f "$HOME/.nvm/nvm.sh" ]; then . "$HOME/.nvm/nvm.sh"; fi
npm --prefix front ci
npm --prefix front/admin ci
npm --prefix front/portfolio ci
npm run build

# Native API and complete workspace.
cargo build --locked -p djmxcreation-backend-wasi --bin djmxcreation-backend-wasi
cargo build --locked --workspace

# WASI Preview 2 components.
rustup target add wasm32-wasip2
cargo build --locked --profile wasi-release --target wasm32-wasip2 -p djmxcreation-backend-wasi -p djmxcreation-frontend-wasi --lib
```

| Artifact | Path |
| --- | --- |
| Native API, debug | `target/debug/djmxcreation-backend-wasi` |
| Native API, release (build with `--release`) | `target/release/djmxcreation-backend-wasi` |
| WASI API | `target/wasm32-wasip2/wasi-release/djmxcreation_backend_wasi.wasm` |
| WASI frontend | `target/wasm32-wasip2/wasi-release/djmxcreation_frontend_wasi.wasm` |

Use the `wasi-release` profile declared in `Cargo.toml` for WASI builds. The API
exports a wstd/Axum HTTP entry point and uses WASI HTTP for S3 requests.

Run repository checks on Linux/WSL with Podman available for integration tests:

```bash
npm test
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

After starting the [local development stack](#local-development), its HTTP and
browser tests can be run separately:

```bash
cargo run -p dev-server -- test
cargo run -p dev-server -- test --browser
```

HTTP checks cover real tokens/JWKS, public reads, 401/403 responses, project CRUD,
S3 upload/download, and refresh. Browser checks cover PKCE, direct links/reloads,
creation validation/error recovery, publication, media imports over 2 MiB, cover
replacement, deletion, profile/photo edits, public contact links, mobile navigation,
and sign-out. They remove test projects and restore the original profile/contact
content.

CI builds frontend assets before Rust checks and validates both WASI components.
Tagged releases package the native API, both WASI components, and frontend assets:
see [CI](.github/workflows/ci.yml) and [release automation](.github/workflows/release.yml).

```bash
cargo audit
npm --prefix front audit
npm --prefix front/admin audit
npm --prefix front/portfolio audit
```

## Local development

The optional Rust `dev-server` CLI builds and launches an isolated development
stack containing the Hub, API, both frontends, PostgreSQL, and RustFS. This workflow
builds a Hub checkout and provisions test users; it is independent of deployment
against an externally hosted identity provider.

See the [local stack guide](crates/dev-server/README.md) for prerequisites, source
checkout configuration, generated development credentials, URLs, and lifecycle
commands. For frontend-only development, see [front/README.md](front/README.md).

```bash
cargo run -p dev-server -- start --mode wasi --detach
cargo run -p dev-server -- status
cargo run -p dev-server -- test --browser
cargo run -p dev-server -- stop
```

## Security

[OpenID Connect WASI Hub](https://github.com/Djudicael/OpenID-Connect-WASI-Hub)
is the identity provider. The admin frontend keeps `keycloak-js` as its browser
adapter. The API is an independent OIDC resource server: it retrieves the realm's
discovery document and JWKS, then verifies RS256 or EdDSA signatures, issuer,
audience, expiry, subject, issued-at time, and optional not-before time. It requires
an access-token `scope` claim and rejects sender-constrained tokens without proof
support. Authentication cannot be disabled by omitting configuration.

All API writes, `GET /api/portfolio/v1/projects`, and `GET /api/home/v1/settings` require the configured admin
role, either in `realm_access.roles` or in
`resource_access[OIDC_ADMIN_CLIENT_ID].roles`. Profile/contact reads, visible
project reads, spotlights, `/ping`, and `/metrics` remain public. Anonymous
pagination must use `visible=true`; requests for hidden project details return
404. A malformed or invalid supplied bearer token returns 401, a valid token
without the admin role returns 403 on administrative operations, and an
unavailable discovery/JWKS endpoint returns 503 when keys must be refreshed.

### Configure the Hub and the applications

Configure the deployed identity provider and applications as follows. The API uses
HTTP protocol endpoints; it does not require the provider source tree, database,
or private signing keys.

1. In the Hub console, create the `portfolio` realm and a public client named
   `portfolio`. Enable Authorization Code, Refresh Token, and required S256 PKCE.
2. Register the exact admin callback and post-logout URLs, for example
   `https://admin.example.com/`. The admin always uses this root callback and restores
   its requested route, query, and fragment after sign-in, including after a reload.
   The Hub does not interpret `/*` as a redirect wildcard. Allow the
   admin origin in the Hub's `OIDC_CORS_ORIGINS` (and client web origins where
   configured). The Hub must allow credentialed requests for the browser
   adapter's code exchange and refresh. Use HTTPS in deployment.
3. Allow `openid`, `profile`, and `roles`. Assign a default client scope with an
   audience mapper adding `portfolio-api` to **access tokens**. Keep this API
   audience out of ID tokens. Include the relevant realm/client role claims in
   access tokens and grant the configured `admin` role to portfolio administrators.
4. Configure the API environment using `.env.template`: `OIDC_ISSUER` must be the
   exact realm issuer, e.g. `https://identity.example.com/realms/portfolio`, and
   `OIDC_AUDIENCE` must be `portfolio-api`. Include the public browser origins
   in `CORS_ORIGINS` when they call the API across origins.
5. Configure the frontend build environment using `front/.env.template` as a
   reference. Set `OIDC_HUB_URL` to the provider's public base URL,
   `OIDC_REALM=portfolio`, `OIDC_CLIENT_ID=portfolio`, and the API URL.
   Restart/rebuild the frontend after changing these build variables.
6. Verify login, token refresh, and logout against the running Hub. Anonymous
   project creation must return 401; an ordinary user's access token must return
   403; an administrator's access token must permit project creation. Tokens from
   another realm, for another audience, with a modified signature, or expired
   must be rejected. Confirm the public portfolio still loads anonymously.

The [frontend build plugin](front/build/keycloak-hub-plugin.js) adapts Keycloak JS
26's refresh-token metadata parsing to the Hub's opaque refresh tokens. It preserves
the original token and access/ID-token parsing and nonce checks. Builds fail if the
upstream code changes, requiring a compatibility review during upgrades.

The backend must reach the public issuer and its same-origin JWKS URL. Inside
containers, `localhost` refers to the container; use a reachable HTTPS Hub hostname.
Native discovery/JWKS requests do not follow redirects. Keys are cached for five
minutes; unknown key IDs trigger at most one refresh per thirty seconds, so allow
that interval when rotating keys and retain old signing keys until issued tokens
expire. WASI runtimes that instantiate a new component for each request may fetch
keys for each authenticated request. Outbound WASI HTTP access to the issuer is
required. Offline JWT validation does not check session revocation on each request;
revoked sessions stop token renewal, while issued access tokens remain valid until
expiry. This integration accepts bearer access tokens, not API keys or Basic Auth.

Run `cargo audit` and the three frontend npm audits when updating dependencies;
review remaining advisories rather than treating a successful build as a security
audit. See [Builds and tests](#builds-and-tests) for commands.

## Remaining work

- Automatic thumbnail generation.
- Admin UI for spotlight management.
- OpenAPI documentation.
- Turnkey deployment integrations for specific hosting platforms.

## Project layout

```text
djmxcreation_backend/
├── crates/
│   ├── djmxcreation-backend-wasi/  # Native API + WASI HTTP component
│   ├── frontend-wasi/             # Embedded admin/portfolio asset server
│   ├── dev-server/                # Rust stack CLI and proxies
│   ├── app-service/               # Business logic
│   ├── repository/                # PostgreSQL + S3 access
│   ├── app_core/                  # Shared DTOs, views, and traits
│   ├── app_config/                # Environment configuration
│   ├── app-error/                 # Shared errors
│   ├── migration/                 # Native SQL migration runner
│   └── test-util/                 # Podman integration-test helpers
├── sql/migrations/                # Versioned SQL files
├── front/admin/                   # English CMS admin
├── front/portfolio/               # Public portfolio
├── front/shared/                  # Shared browser runtime and tests
├── front/build/                   # Shared esbuild configuration
├── scripts/                       # Browser workflow tests
├── .agents/skills/                # Repository agent skills
├── .github/workflows/             # CI and release automation
├── .env.template                  # Standalone API configuration example
└── rust-toolchain.toml            # Rust and WASI target pin
```
