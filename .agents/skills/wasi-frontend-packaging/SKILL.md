---
name: wasi-frontend-packaging
description: Package DJMXCreation's built admin and portfolio frontends (front/<app>/dist) into the frontend-server crate, which serves the embedded static assets with SPA fallback as a native binary or a WASI 0.3 (P3 async) component. Use when changing the frontend server crate (currently djmxcreation-frontend-wasi), its build.rs asset table, the FRONTEND_APP selection, content types, cache or security headers, or the build order between npm and cargo.
---

# Packaging the frontends (`frontend-server`)

Static assets are **compiled into the server** with `include_bytes!`. It
needs no filesystem at runtime, so the same crate is a tiny, immutable,
self-contained web server whether it runs as a native binary or as a WASI
component. The crate is `crates/frontend-wasi` (package
`djmxcreation-frontend-wasi`) today; the target is `crates/frontend-server`. One crate embeds both apps; `FRONTEND_APP`
(`admin` or `portfolio`) selects which one an instance serves. The frontends
call the API over relative `/api/...` URLs, which the gateway routes.

## Crate

```toml
[package]
name = "frontend-server"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]   # cdylib = WASI component, rlib = native bin and tests

[[bin]]
name = "frontend-server"
path = "src/main.rs"

[dependencies]
axum = { workspace = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
app-http = { workspace = true }   # handle() for the P3 entry point
wasip3 = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "net", "signal", "macros"] }
axum = { workspace = true, features = ["tokio", "http1"] }

[dev-dependencies]
axum-test = { workspace = true }
```

Do **not** add it to `default-members`. It fails to build until the npm build
has produced `front/admin/dist` and `front/portfolio/dist`.

## `build.rs`: generate the asset table

- For each app in `["admin", "portfolio"]`, walk `../../front/<app>/dist`
  recursively in **sorted** order, so the output is deterministic.
- Fail with a clear message if `index.html`, `js/index.js`, or the CSS bundle
  is missing: "missing <app>/<file>: run `npm --prefix front run build`
  before building the frontend server".
- Reject symlinks and skip `*.map` files.
- Emit `cargo:rerun-if-changed=<dist dir>` for each app.
- Write `OUT_DIR/assets.rs` with one static per app:
  `static ADMIN: &[(&str, &[u8])] = &[("js/index.js", include_bytes!("…")), …];`

When you add an app, add it to the build script list, the `app_router`
match, the tests, and the dev CLI.

## `src/lib.rs`: router and entry point

```rust
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
type Assets = &'static [(&'static str, &'static [u8])];

pub fn app_router(app: &str) -> Router {
    let assets = match app {
        "admin" => ADMIN,
        "portfolio" => PORTFOLIO,
        _ => panic!("FRONTEND_APP must be admin or portfolio"),
    };
    Router::new()
        .route("/", get(index))
        .route("/{*path}", get(serve))
        .with_state(assets)
}

#[cfg(target_arch = "wasm32")]
wasip3::http::service::export!(HttpServer);

#[cfg(target_arch = "wasm32")]
struct HttpServer;

#[cfg(target_arch = "wasm32")]
impl wasip3::exports::http::handler::Guest for HttpServer {
    async fn handle(
        request: wasip3::http::types::Request,
    ) -> Result<wasip3::http::types::Response, wasip3::http::types::ErrorCode> {
        let app = std::env::var("FRONTEND_APP").unwrap_or_default();
        app_http::handle(app_router(&app), request).await
    }
}
```

The native `src/main.rs` binds `BIND_ADDRESS:PORT` and serves the same
`app_router(&FRONTEND_APP)`; it is a full deployment option, not only a
development aid.

## Serving rules (`serve_path`)

1. Return **404** for any path containing `.` or `..` segments.
2. Return **404** for `api` and `api/*`, `ping`, and `health*`. A missing API
   route must not return the HTML shell, because that hides errors from the
   frontend.
3. Return the exact asset when it exists.
4. Return `index.html` (SPA fallback) when the last path segment has **no
   extension**, so deep links such as `/projects/42` work after a reload.
5. Return **404** for missing files that have an extension
   (`/ressource/missing.svg`).
6. Only `GET` routes exist (axum returns 405 for other methods).

Keep rule 2 in sync with the gateway/proxy route split (see
`wasi-app-architecture` → deployment) and with the frontend router.

## Response headers

- `content-type` comes from a small extension match: `html`, `js` →
  `text/javascript; charset=utf-8`, `css`, `svg`, `png`, `jpg`/`jpeg`,
  `webp`, `gif`, `ico`, `woff`, `woff2`, `json`, `mp4`, otherwise
  `application/octet-stream`.
- `x-content-type-options: nosniff` on every response.
- `cache-control: no-store` for `index.html`. Hashed assets
  (`*.<hash>.css`, `*.<hash>.js`) use `public, max-age=31536000, immutable`;
  other assets use `no-cache`.
- `content-security-policy` on HTML responses, mirroring the `<meta>` CSP in
  `front/<app>/index.html` (including the storage and OIDC origins the app
  needs). Change both together.
- `referrer-policy: no-referrer` and `x-frame-options: DENY`.

## Build order (CI and local)

```bash
npm --prefix front ci && npm --prefix front/admin ci && npm --prefix front/portfolio ci
npm --prefix front run build
cargo build --locked --profile wasi-release --target wasm32-wasip2 -p frontend-server --lib
```

CI and `dev-server` always build the frontends first; keep it that way.

## Tests

`axum_test::TestServer::new(app_router(app))` for each app, asserting that:

- `/` and a deep link (`/projects/example`) return 200 HTML;
- `/js/index.js` returns 200 with `text/javascript; charset=utf-8`;
- a missing `.svg` returns 404;
- `/api/portfolio/v1/projects` returns 404;
- `POST /` returns 405;
- an asset whose name contains an encoded space resolves
  (`/ressource/icon/pen%20tool.svg` in admin);
- cache and security headers are present.

## When not to embed

Media (videos, project images) lives in object storage and is served through
presigned URLs. Embed only the application shell and small UI assets
(icons, fonts, the logo).
