# Cargo, toolchain, and build configuration

## `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.97.1"            # exact release; bump deliberately (maintain-djmxcreation)
components = ["clippy", "rustfmt"]
profile = "minimal"
targets = ["wasm32-wasip2"]
```

Keep `[workspace.package].rust-version` consistent with the channel. Never
silently build with a different toolchain.

The target stays `wasm32-wasip2`: stable Rust has no `wasm32-wasip3` target
yet. P3 support comes from the `wasip3` crate (component-model async ABI) and
from the host running Wasmtime with `component-model-async` and `p3` enabled.

## Root `Cargo.toml` (target)

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
# Crates every developer builds. Tooling and the frontend server (needs
# front/*/dist) are members but not default members.
default-members = [
    "crates/app_core",
    "crates/app_config",
    "crates/repository",
    "crates/app-service",
    "crates/app-api",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.97"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# workspace crates
app_core = { path = "crates/app_core" }
app_config = { path = "crates/app_config" }
repository = { path = "crates/repository" }
app-service = { path = "crates/app-service" }
app-http = { path = "crates/app-http" }
migration = { path = "crates/migration" }
test-util = { path = "crates/test-util" }

# WASI 0.3 (P3 async HTTP, clocks, sockets on the wasm32-wasip2 target).
# Keep this on the same version that wasi-pg-client resolves (see below).
wasip3 = { version = "0.8", features = ["http-compat"] }
futures-concurrency = "7"

# web
axum = { version = "0.8", default-features = false, features = ["json", "matched-path", "original-uri", "query", "tower-log", "tracing", "multipart", "macros"] }
tower = { version = "0.5", features = ["timeout", "util"] }
tower-http = { version = "0.7", features = ["trace", "cors"] }

# data — wasi-pg-client is always the latest release (0.2.1+ uses P3 sockets).
wasi-pg-client = { version = "0.2.1", features = ["uuid", "serde-json", "chrono"] }
aws-sdk-s3 = { version = "1", default-features = false }
aws-smithy-runtime-api = { version = "1", features = ["client"] }
aws-smithy-types = "1"

# media
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
fast_image_resize = { version = "6", default-features = false, features = ["std", "image"] }
jpeg-encoder = "0.7"

# common
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
async-trait = "0.1"
futures = "0.3"
uuid = { version = "1", features = ["v4", "fast-rng", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
url = "2"
getrandom = "0.4"
sha2 = "0.11"
base64 = "0.23"
jsonwebtoken = { version = "11", default-features = false, features = ["rust_crypto"] }

# native runtime (and tooling)
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.13", default-features = false, features = ["rustls", "json"] }
dotenvy = "0.15"
metrics = "0.24"
metrics-exporter-prometheus = { version = "0.18", default-features = false, features = ["http-listener", "tracing"] }

# tests
axum-test = "21"
rustainers = "0.15"

# Rust 1.97's wasm-component-ld crashes at opt-level 3 for crc-fast (pulled in
# by aws-sdk-s3). Keep native release settings unchanged; use O2 for WASI.
[profile.wasi-release]
inherits = "release"
opt-level = 2
```

The versions above are a shape, not a pin list. Before choosing a version,
check the registry (`cargo info <crate>`) and the current lockfile. Never
guess a version. Record each intentional pin (`=x.y.z`) with a comment that
explains why and when it can be lifted.

### One `wasip3` in the graph

`wasi-pg-client` depends on a specific `wasip3` release (0.2.1 →
`wasip3 0.8.0+wasi-0.3.0`). The application must use the **same** `wasip3`
version, otherwise two `wit-bindgen` async runtimes end up in one component
and HTTP and database futures stop sharing an executor. After every change to
`wasip3` or `wasi-pg-client`, run:

```bash
cargo tree -i wasip3 --target wasm32-wasip2 -e normal
```

It must print exactly one version. When `wasi-pg-client` moves to a newer
`wasip3`, bump the workspace entry to match, and check that the Wasmtime used
by `dev-server`, CI, and the platform supports that WASI 0.3 snapshot.

### Forbidden on the wasm path

`wstd`, `wstd-axum`, the `wasi` 0.14 bindings, `tokio`, `reqwest`, and
`--cfg tokio_unstable`. They belong to the old P2 design or to the native
runtime and would block or duplicate the P3 executor. `tokio` and `reqwest`
are fine in native target tables.

## Target-specific dependencies (server crates)

```toml
[lib]
crate-type = ["cdylib", "rlib"]    # cdylib = WASI component, rlib = native bin and tests

[[bin]]
name = "app-api"
path = "src/main.rs"

[dependencies]
axum = { workspace = true }
app_core = { workspace = true }
app-http = { workspace = true }     # same API on both targets
wasi-pg-client = { workspace = true }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasip3 = { workspace = true }       # only for the Guest export in lib.rs
getrandom = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "net", "signal", "macros"] }
axum = { workspace = true, features = ["tokio", "http1"] }
wasi-pg-client = { workspace = true, features = ["tokio-transport"] }

[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]
test-util = { workspace = true }
axum-test = { workspace = true }
```

Rules:

- `wasi-pg-client` uses its P3 socket transport automatically on
  `wasm32-wasip2`. **Only native builds add `tokio-transport`.** When you
  change its features, update the native and wasm declarations together.
- `aws-sdk-s3`: native builds enable `default` and `behavior-version-latest`.
  wasm builds use no default features and add `aws-smithy-runtime-api` and
  `aws-smithy-types` for the connector.
- `app-http` is a normal dependency; it carries the per-target
  dependencies (`wasip3` + `futures-concurrency` on wasm, `reqwest` + tokio
  on native) so other crates do not repeat them.
- `wasip3` appears directly only in crates that export a P3 `Guest`.
- Gate tooling crates (`rustainers`, `tokio` with `process`, `portpicker`,
  `metrics-exporter-prometheus`) to native, or keep them in native-only
  crates.

## `.cargo/config.toml`

```toml
[target.wasm32-wasip2]
# HTTP cdylib components are started with `wasmtime serve` (see deployment.md).
runner = "wasmtime run -W component-model-async=y -S p3=y -S cli=y -S inherit-network=y -S inherit-env=y -S tcp=y -S allow-ip-name-lookup=y"
```

Do not set `build.target`. Native remains the default, so `cargo test` works.
Remove `rustflags = ["--cfg", "tokio_unstable"]` once no wasm crate uses
tokio sockets (it is not needed for P3).

## Lints

Optional `.clippy.toml` to keep functions readable:

```toml
cognitive-complexity-threshold = 30
too-many-lines-threshold = 150
too-many-arguments-threshold = 8
```

CI runs `cargo clippy --workspace --all-targets -- -D warnings` and a wasm32
clippy pass on the server crates and `app-http`.

## Supply-chain policy

Add `deny.toml` (cargo-deny) with `advisories`, `licenses` (MIT, Apache-2.0,
BSD-3-Clause, ISC, Unicode-3.0, …), `bans.wildcards = "deny"`, and
`sources.unknown-git = "deny"`. Every `ignore` entry carries a `reason`.

## Local patches

Avoid forking crates. If a dependency panics under WASI, first look for a
feature flag or a wasm-friendly alternative. If a patch is unavoidable, use
`[patch.crates-io]` pointing to `crates/<name>-patch`, record it in an ADR in
`docs/adr/`, and list the removal conditions. Delete a patch crate as soon as
nothing in `[patch]` references it.
