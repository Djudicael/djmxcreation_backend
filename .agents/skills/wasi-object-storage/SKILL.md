---
name: wasi-object-storage
description: Talk to S3-compatible object storage (RustFS, MinIO, AWS S3) from DJMXCreation's servers using aws-sdk-s3 — its default client natively, and a custom connector over app-http in WASI 0.3 (P3 async) components. Use when creating the storage client, uploading/downloading/deleting objects, generating presigned URLs, the direct browser multipart upload flow, media variants from the media worker, bucket CORS/provisioning, or debugging S3 calls under WASI. Do not use rust-s3 or reqwest-based clients in WASI code.
---

# Object storage (S3 / RustFS)

Decision record: `docs/adr/0001-wasi-0.2-architecture-and-networking.md`.
`rust-s3` was abandoned because it hard-wires `reqwest` and TLS. The official
**`aws-sdk-s3`** runs with `default-features = false` on wasm, and HTTP is
delegated to the host's `wasi:http` by a small connector. The result is a
small component with no native TLS, and the same API on both targets.

## Dependencies (`repository/Cargo.toml`)

```toml
[dependencies]
app-http = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
aws-sdk-s3 = { workspace = true, features = ["default", "behavior-version-latest"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
aws-sdk-s3 = { workspace = true }                 # default-features = false at workspace level
aws-smithy-runtime-api = { workspace = true }     # features = ["client"]
aws-smithy-types = { workspace = true }
```

Natively the SDK uses its own hyper/rustls client. In a component it uses
the `WasiHttpClient` connector below. No `wasi` 0.14 bindings, `wstd`, or
`url` crate are needed. The connector and the client builder are the only
`cfg` blocks for storage; repositories and services are target-neutral.

## The storage client

`config/storage.rs` owns `StorageClient { inner: aws_sdk_s3::Client }`,
`get_storage_client(StorageConfiguration)`, `ensure_bucket`, and
`configure_media_cors`. The wasm branch of `get_storage_client` sets:

```rust
S3ConfigBuilder::new()
    .behavior_version(BehaviorVersion::latest())
    .sleep_impl(WasiSleep)              // app_http::sleep-based AsyncSleep
    .credentials_provider(credentials)  // Credentials::new(access, secret, None, None, "Static")
    .region(Region::new(cfg.region))
    .endpoint_url(cfg.endpoint)
    .force_path_style(true)             // required for RustFS/MinIO
    .http_client(WasiHttpClient::new())
```

The native branch uses the same builder without `sleep_impl` and
`http_client`.

Read [the P3 connector](references/wasi-connector.md) for the full
`WasiHttpClient` and `WasiSleep` source before creating or changing them.
Key behaviour: the request goes through `app_http::send` (streamed, on
the shared P3 executor, no manual polling), empty `POST/PUT/PATCH/DELETE`
bodies get `content-length: 0` (RustFS answers 411 otherwise), and response
bodies are read with an explicit size cap.

## Repository pattern

The port lives in core (`IStorageRepository`: `upload_file`,
`get_object_url`, `remove_object`, …), and `storage_repository.rs` implements
it with `StorageClient`. Media sessions use `IMediaRepository`
(`media_repository.rs`). Services depend only on the `DynI…` aliases.

- Map every SDK error to a `app_core::Error::Storage("<operation>")`. Log
  `bucket`, `key`, and `error = ?e`; never log credentials or presigned URLs.
- Object keys: `<feature>/<uuid>-<sanitised-file-name>` (for example
  `projects/<project-id>/<uuid>-cover.jpg`). Validate file names at the API
  boundary (no `..`, `/`, `\`, or NUL).
- Store the **key**, bucket, and MIME type in PostgreSQL, never a URL.
  Resolve URLs on read.

## Serving objects: presigned URLs

- Reads: `get_object().presigned(PresigningConfig::expires_in(d))` with a
  bounded expiry (hours, not days). The frontend renews URLs on an
  expired-link error.
- Presigned URLs are signed for an exact host, path, and query.
  `STORAGE_ENDPOINT` must be the host that browsers can reach, and it must be
  HTTPS when the site is HTTPS. **Never rewrite the host after signing.**
- The storage endpoint must support `HEAD` and ranged `GET` (206) for video
  seeking.
- Images have optimised variants (widths produced by the media worker). The
  API returns the variant set; the frontend picks with `srcset`
  (`front/shared/src/responsive-image.js`).

## Large uploads: direct browser → storage

Videos and large images never stream through a component. The flow
(`router/media_router.rs`, `media_repository.rs`, and
`front/admin/app/utils/direct-upload.js`):

1. `POST /api/portfolio/v1/projects/{project}/uploads` → validates
   `fileName`, `mimeType`, and `size` against the limits in core
   (`MAX_VIDEO_SIZE`), creates an upload session row and an S3 multipart
   upload.
2. `POST …/uploads/{session}/parts/{part}` → returns a 15-minute presigned
   `PUT` URL for one 8 MiB part (`PART_SIZE`).
3. The browser `PUT`s each part directly to storage and keeps the `ETag`s.
4. `POST …/uploads/{session}/complete` → checks the storage-reported part
   sizes, completes the upload, and attaches the media. This call is
   **idempotent**.
5. `POST …/uploads/{session}/abort` → discards the parts.

Sessions expire. New uploads perform best-effort, bounded cleanup of expired
sessions (a few rows per call). Production buckets also need an
`AbortIncompleteMultipartUpload` lifecycle rule for abandoned sessions.

Small uploads (a few MiB) may use multipart form data on the API, with a
per-route `DefaultBodyLimit`.

## Media worker

`media-worker` downloads an original, resizes one width per call
(`image`, `fast_image_resize`, `jpeg-encoder`), uploads the variant, and
records it. Every read is bounded: the original's size is checked against
the processing limit **before** download, and the connector's
`MAX_RESPONSE_BYTES` must be at least that limit. A failed step is recorded
with a retry time; the worker never loops inside one request.

## Provisioning (not request-time)

- `ensure_bucket` and `configure_media_cors` run from `dev-server`, tests, or
  provisioning scripts. A production request handler never creates buckets
  or changes policy.
- CORS rules list explicit origins with `PUT`, `GET`, and `HEAD`, and expose
  `ETag`, `Accept-Ranges`, and `Content-Range`.

## Configuration

`STORAGE_ENDPOINT`, `STORAGE_ACCESS_KEY`, `STORAGE_SECRET_KEY`,
`STORAGE_REGION` (default `us-east-1`), and `STORAGE_BUCKET`. Credentials are
deployment secrets (`[secrets]` in `deploy/wcp/*.example.toml`).

## Testing

- Service tests use the fake storage repository and its probe
  (`fake_storage_repository.rs`, `test-fakes` feature).
- `test_util::shared_rustfs()` starts `docker.io/rustfs/rustfs` once per
  process (Podman). Test upload, presign + GET, multipart complete/abort, and
  delete against it on native.
- The wasm connector is validated by the wasm32 builds and by
  `dev-server smoke` in WASI mode (real uploads and reads through the
  component).
