---
name: wasi-dev-stack
description: Build or change DJMXCreation's local-environment CLI (the dev-server crate), which starts Podman PostgreSQL/RustFS, runs migrations, provisions an isolated OpenID Connect WASI Hub realm, builds the frontends and WASI P3 components, serves them with wasmtime (or native binaries), exposes same-origin proxies, runs smoke/P3 runtime/Playwright checks, and stops or destroys the environment. Use when adding a service, container, command, seed, or check to the CLI, or changing start/stop/destroy/status/logs/test behaviour. For simply running the stack, use run-djmx-stack.
---

# Local environment CLI (`dev-server`)

The repository ships **one** Rust CLI that brings the full local environment
up and tears it down. Do not add a second orchestration path (shell scripts,
compose files for the app, Makefiles). Agents and humans use the same
commands; the operational guide for agents is the `run-djmx-stack` skill.

The crate is `crates/dev-server` (`cargo run -p dev-server -- …`). It is
native tooling; the servers it starts run either natively or as WASI
components (`--mode`). Server crate and artifact names used below are the
target names (`app-api`, `frontend-server`, `media-worker`); until those
renames land, use the current ones from the refactor map in
`wasi-app-architecture`.

## Command surface

```text
cargo run -p dev-server -- <command> [options]
```

| Command | Effect | Data |
|---|---|---|
| `start` (alias `up`) | Preflight → build → infra → provision → services → proxies → smoke → `ready`. Foreground until Ctrl-C, or background with `--detach`. | creates or reuses volumes |
| `stop` (alias `down`) | Stops owned processes and labelled containers. | **preserves** volumes, credentials, state |
| `destroy` | `stop`, then removes the labelled containers, **volumes**, and the state directory. Requires `--yes`; without it, prints a dry run. | **deletes everything for this checkout** |
| `status [--json]` | Phase, mode, liveness, URLs, ports, state dir. Agents call this first. | read only |
| `logs [<service>]` | Prints `launcher.log` or `<service>.log`. | read only |
| `test [--browser]` | HTTP smoke checks against a `ready` stack; `--browser` runs Playwright. | test data cleaned up |
| `smoke` | One-shot `start` → `test` → `stop` (CI-friendly). | preserves volumes |
| `runtime-test [--skip-build]` | Builds and runs the `app-http` `http-client-smoke` component under Wasmtime: P3 HTTP server/client, PostgreSQL sockets, and deadlines in a real component. | temporary database |
| `infra` (alias `db-only`) | Containers, migrations, and seed data only; prints connection strings. For `cargo test` or an IDE-run native API. | creates or reuses volumes |
| `migrate` | Runs `migration::run_migrations` against `DATABASE_URL`. | schema changes |

Options for `start`, `infra`, and `smoke`:

| Option | Meaning |
|---|---|
| `--mode wasi` (default) or `--mode native` | WASI mode runs every application server as a P3 component under `wasmtime serve`; native mode runs the native binaries. Aliases: `wasm`, `native`. |
| `--detach` | Re-executes itself as a hidden `_serve` supervisor in its own process group and returns immediately. |
| `--skip-build` | Reuses existing artifacts; fails clearly if one is missing. |
| `--port-offset N` | Shifts every public port together, so two checkouts can run side by side. |
| `--with-media-worker` | Builds and starts the optional media worker component. |
| `--hub-dir PATH` (or `OIDC_HUB_DIR`) | Checkout of the OpenID Connect WASI Hub used as the development identity provider. |

`destroy`, `runtime-test`, `infra`, and the `up`/`down`/`db-only` aliases
are target behaviour that the current crate does not have yet; add them
before documenting them in `run-djmx-stack`. Keep the usage string in
`execute()`, the crate README, and `run-djmx-stack` in sync with what is
implemented.

## What runs

| Service | WASI mode | Native mode | Default public URL |
|---|---|---|---|
| OIDC provider (Hub API + console) | two P3 components | native binaries | `http://localhost:3000` |
| DJMX admin | `frontend_server.wasm` with `FRONTEND_APP=admin` | `frontend-server` binary, same env | `http://localhost:3008` |
| DJMX portfolio | `frontend_server.wasm` with `FRONTEND_APP=portfolio` | native binary | `http://localhost:3009` |
| DJMX API | `app_api.wasm` | native `app-api` | `http://localhost:8081` |
| Media worker (optional) | `media_worker.wasm` | native binary | `http://localhost:3010` |
| PostgreSQL, RustFS | Podman containers | Podman containers | random loopback ports |

Ports shift with `--port-offset`. Always read the real URLs from `status`.

## Non-negotiable behaviour

1. **Native only, Linux/WSL only.** `ensure!(cfg!(target_os = "linux"), "Run this CLI in WSL/Linux")`.
   The crate is in `members` but not `default-members`. Never compile it for
   wasm32.
2. **Isolation per checkout.** `tag = hash(canonical repo root)`. Everything
   owned by the stack includes the tag:
   - state dir: `$XDG_STATE_HOME` or `~/.local/state`, then
     `/djmx-stack-<tag>/` (mode `0700`, files `0600`, written atomically
     through `.tmp` + rename);
   - containers: `djmx-stack-<tag>-<suffix>`, label `dev.djmx.stack=<tag>`;
   - volumes: `djmx-stack-<tag>-<suffix>`, with the **same label** (create
     them with `podman volume create --label …` so `destroy` can find them).
3. **Never touch what you do not own.** Before replacing a container, inspect
   its label and refuse when it does not match. No `podman rm --all`,
   `pkill`, or broad prune commands. Cleanup addresses only recorded PIDs,
   process groups, and labelled resources.
4. **One controller at a time.** Take an exclusive `File::lock()` on
   `<state>/control.lock` for every mutating command. `start` refuses when a
   live stack exists ("use status or stop").
5. **Liveness that survives PID reuse.** `runtime.json` stores
   `pid` + `start_time` (field 22 of `/proc/<pid>/stat`). A stack is alive only
   when both match.
6. **Secrets generated locally, once.** `credentials.json` (0600) holds
   random database and storage passwords, signing keys, and test-user
   passwords. They are reused across starts and deleted only by `destroy`.
   Never print them in progress output.
7. **Preflight before work.** Check `podman version`, `wasmtime --version`
   (minimum version with the WASI 0.3 snapshot used by `Cargo.lock`;
   `WASMTIME_BIN` or `<state>/runtime/wasmtime` override the system binary),
   node/npm, that the Hub checkout exists, and **bind-test every public port**
   before any build starts.
8. **Every step is logged and named.** Each subprocess writes
   `<state>/<label>.log`. A failure message names the step and its log.
   `runtime.json.phase` is updated before each step; on failure the error is
   stored in `runtime.json.error`.
9. **Failure cleans up.** If `start` fails at any phase, it stops what it
   started and records `phase = "failed"`.
10. **Reuse application code.** Provisioning calls the project's own crates:
    `migration::run_migrations`,
    `repository::config::storage::{get_storage_client, ensure_bucket, configure_media_cors}`,
    and `app_config` structs. Never reimplement SQL or S3 logic in the CLI.
11. **Each repository builds with its own toolchain.** Build the Hub from its
    own directory with its own `rust-toolchain.toml`; never override either
    pin.

## Start sequence

```text
preflight ─► build ─► infra ─► provision ─► services ─► proxies ─► smoke ─► ready
```

1. **Build** (skipped with `--skip-build`): `npm ci` per lockfile →
   `npm --prefix front run build` → in WASI mode
   `cargo build --locked --profile wasi-release --target wasm32-wasip2 -p app-api -p frontend-server [-p media-worker] --lib`;
   in native mode `cargo build --locked -p <bins>`. Then build the Hub from
   its checkout. Load nvm by running npm through
   `bash -lc '. ~/.nvm/nvm.sh; …'` when present.
2. **Infra**: `podman run -d --name … --label … -p 127.0.0.1::<port> -v <volume>:<mount>`
   with the random host port read back via `podman port`. Postgres readiness
   is a `pg_isready` loop (90-second deadline); RustFS readiness is
   `GET /health`. Pin images to a major version.
3. **Provision** (idempotent, safe on every start): create the private
   databases (app and Hub), run migrations, ensure the bucket and CORS for
   the local admin/portfolio origins, and provision the Hub realm
   `portfolio`, its public PKCE client, the API audience mapper, the admin
   role, and the admin and ordinary test users.
4. **Services**: spawn each server in its own process group
   (`cmd.process_group(0)`) with an explicit env map. Never rewrite either
   repository's `.env`.
   - WASI: `wasmtime serve --addr 127.0.0.1:<port> -W component-model-async=y -S p3=y -S cli=y -S inherit-env=y -S inherit-network=y -S tcp=y -S allow-ip-name-lookup=y <artifact>.wasm`
     (the media worker also gets `--max-instance-reuse-count 1`).
   - Native: `target/debug/<bin>` with the same env.
   - Wait for each health URL; while waiting, check whether the child has
     exited and fail fast with its log path.
5. **Proxies**: an in-process axum + reqwest reverse proxy per public origin
   (`proxy.rs`). On the admin and portfolio origins, `/api/*`, `/ping`, and
   `/health*` go to the API and everything else to the frontend server.
   This gives a same-origin setup without CORS, like production. Strip
   hop-by-hop headers; preserve cookies and status codes.
6. **Smoke**: public reads, rejected anonymous writes, rejected non-admin
   writes, real Hub login and JWKS validation, project CRUD and refresh,
   direct multipart upload to RustFS, and frontend deep-link fallback. Set
   `phase = "ready"` only after they pass.
7. **Supervise**: loop on `select!{ SIGINT/SIGTERM, 1s tick }`. If any child
   exits, fail with its name. On exit, run cleanup.

## Stop and destroy

`cleanup()` (shared by stop, failure, and Ctrl-C):

1. Abort the proxy tasks.
2. For children in **reverse** start order, `kill -TERM -<pgid>`, wait up to
   10 seconds, then `kill -KILL -<pgid>`.
3. `podman rm -f <name>` for every container recorded in this run.
4. Set `phase = "stopped"` (or `"failed"` with the error).

`stop`: when the stack is alive, send `SIGTERM` to the supervisor PID, then
poll `runtime.json` until `stopped` or `failed` (about 2 minutes, then point
to `launcher.log`). Print "Stack stopped; volumes and credentials preserved".

`destroy --yes`:

1. Take the control lock. If the stack is alive, stop it first.
2. `podman ps -a --filter label=dev.djmx.stack=<tag> -q` → `podman rm -f`
   each (also removes leftovers from crashed runs).
3. `podman volume ls --filter label=dev.djmx.stack=<tag> -q` →
   `podman volume rm` each. For volumes created before labels existed, also
   remove the exact names `djmx-stack-<tag>-<suffix>` from the known suffix
   list. Never use a wildcard.
4. Delete the state directory after verifying that it is under the state
   base and named `djmx-stack-<tag>`.
5. Print what was removed. Without `--yes`, print the same list as a dry run
   and exit non-zero.

`destroy` never deletes build artifacts (`target/`, `dist/`) or the Hub
checkout.

## Status output

`status --json` returns `{ running, phase, mode, urls, ports, state_dir, error? }`.
Agents parse it. The human form shows mode, phase, liveness, one URL per
line, and the state dir.

Read [the CLI skeleton](references/cli-skeleton.md) for the module layout,
types, and key function bodies before changing the crate's structure.

## Done when

- `start`, `status`, `test`, `stop`, `start` (data still present), and
  `destroy --yes` (nothing labelled remains) all work in WASI and native
  mode.
- `runtime-test` passes after P3 HTTP or database changes.
- A second checkout with `--port-offset 100` runs concurrently without
  conflicts.
- Killing a component during `ready` makes the supervisor fail, clean up, and
  record the error.
- `cargo clippy -p dev-server --all-targets -- -D warnings` and the crate's
  unit tests pass.
