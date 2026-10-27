# Local stack CLI

The Rust `dev-server` CLI starts an isolated local development/test stack with
DJMXCreation and OpenID Connect WASI Hub. It is optional tooling for contributors;
deploying DJMXCreation with an externally hosted provider does not require a Hub
checkout. See the [deployment documentation](../../README.md#deployment).

Run the CLI from the repository root on Linux or in WSL on Windows.

```bash
cargo run -p dev-server -- start --mode wasi
# Ctrl-C stops the services started by this command.
```

For an agent-managed background session:

```bash
cargo run -p dev-server -- start --mode wasi --detach
cargo run -p dev-server -- status
cargo run -p dev-server -- logs
cargo run -p dev-server -- test
cargo run -p dev-server -- test --browser
cargo run -p dev-server -- stop
```

`start` defaults to WASI. `wasm` is an alias for `start --mode wasi`; `native` is an
alias for `start --mode native`. `smoke` starts the entire stack, runs real-provider
HTTP checks, and shuts it down. `migrate` retains the standalone migration command
using `DATABASE_URL` from `.env`.

## Prerequisites and configuration

- WSL/Linux, Podman, OpenSSL, and Wasmtime 50+ with component async/P3 support.
  The current Hub imports WASI 0.3 interfaces absent from Wasmtime 43. Select a
  compatible binary with `WASMTIME_BIN` ([tested release: 50.0.0](https://github.com/bytecodealliance/wasmtime/releases/tag/v50.0.0)), or place a portable binary at
  `runtime/wasmtime` inside the stack state directory. The launcher checks the
  version before building; the portable binary leaves the system install intact.
- The Rust releases and WASI targets pinned by both repositories. DJMXCreation uses
  its own `rust-toolchain.toml`; the Hub uses its own pin. Neither is overridden.
- Node/npm for asset builds. The launcher loads nvm automatically when available.
- A checkout of [OpenID Connect WASI Hub](https://github.com/Djudicael/OpenID-Connect-WASI-Hub)
  to build for this development stack. Set `--hub-dir PATH` or `OIDC_HUB_DIR` to
  select it. The launcher's default is the relative sibling directory
  `../openid_connect_wasi`; no machine-specific directory is required.
- Browser tests use the Hub's locked Playwright dependency. If Chromium is missing,
  run `npm --prefix front/admin exec playwright install chromium` from the Hub in WSL.

Default URLs:

| Application | URL |
| --- | --- |
| Hub console / identity provider | http://localhost:3000 |
| DJMX admin | http://localhost:3008 |
| Public portfolio | http://localhost:3009 |
| Direct API | http://localhost:8081 |

Use `--port-offset 100` to shift all public ports together. Occupied ports cause an
explicit failure. Frontend API calls use relative URLs through the Rust proxy;
OIDC uses the printed Hub URL. Runtime environments are provided by the launcher,
without rewriting either repository's `.env`.

## What runs

WASI mode builds and starts five application servers with `wasmtime serve`:

- Hub API and Hub admin console;
- DJMX API;
- DJMX admin and public portfolio, using the embedded-asset frontend component
  with `FRONTEND_APP=admin` or `FRONTEND_APP=portfolio`.

No Node/Express server serves the apps in this mode. The Rust CLI provides the
local reverse proxies, and Podman runs PostgreSQL and RustFS as infrastructure.
Frontend HTML, JavaScript, CSS, and resources are embedded at build time; runtime
filesystem access is not required. The same frontend binary serves both apps.

Native mode uses native application binaries with the same isolated infrastructure,
realm, URLs, and smoke checks. Stop the running stack before switching modes.

The launcher installs npm dependencies from lockfiles, builds assets before their
Rust wrappers, applies migrations to its private databases, and provisions the
`portfolio` realm, public PKCE client, API audience mapper, admin role, and admin
and ordinary test users. Signing keys are generated once and shared across WASI
instances. HTTP smoke checks verify public reads, rejected anonymous writes,
rejected non-admin writes, real Hub login/JWKS validation, project CRUD and refresh.
Browser checks additionally exercise the actual `keycloak-js` PKCE redirect flow,
deep links/reloads, project creation and publication, media/cover management,
profile/photo edits, contact links on the public portfolio, Home introduction and
featured-project settings, mobile navigation, and sign-out. They delete their test
projects and restore the profile, contact, and Home data.

## State, logs and cleanup

Both repositories pin `wasi-pg-client` to `0.2.0`: the `0.2.1` transport requires a
WASI 0.3 async entry point, whereas the current HTTP components use wstd/Preview 2.
Do not remove that pin until the HTTP and database runtimes are migrated together.

`status` prints the state directory under `$XDG_STATE_HOME` or `~/.local/state`.
`credentials.json` there contains generated local test credentials. Seed logs and
signing keys are private local files; do not publish them. Service/build logs are
separate, and startup errors identify the relevant log.

For the portfolio admin, use the `email` and `password` fields in that file
(the generated development email defaults to `admin@portfolio.local`). The
`reader_email` field identifies the ordinary test user. These accounts belong to
the isolated development realm and are not deployment defaults.

`stop` and Ctrl-C stop only owned processes and labeled containers. PostgreSQL and
RustFS named volumes, test users, and local credentials are preserved for the next
start. A startup failure cleans up its processes and containers as well.

`--skip-build` restarts already-built artifacts when embedded URL configuration has
not changed. Rebuild after changing source/assets or public URLs. `status --json`
is available for automation. This is a local development/test launcher; production
still needs its own gateway, TLS, secret management, and service supervision.

The repository skill `$run-djmx-stack` invokes this CLI and reports readiness and
URLs. It does not implement a second orchestration path.
