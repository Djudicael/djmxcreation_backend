---
name: run-djmx-stack
description: Start, stop, inspect, or test the complete local DJMXCreation stack with OpenID Connect WASI Hub, admin, portfolio, PostgreSQL, and RustFS. Use for local end-to-end testing in WASI or native mode, not production deployment.
---

# Run the DJMXCreation stack

Use the repository launcher from the repository root. It owns the process lifecycle,
provisions a private development realm, and tests real Hub tokens. Default to `wasi`
unless the user requests native mode. In WASI mode all five application servers are
components; the Rust CLI is the host orchestrator/reverse proxy, and PostgreSQL/RustFS
remain infrastructure services in Podman.

Run commands in WSL. Load `~/.nvm/nvm.sh` when installed before npm commands.
From PowerShell, use `wsl.exe --cd /mnt/d/dev/djmxcreation_backend bash -lc '<command>'`.
Read [the launcher guide](../../../crates/dev-server/README.md) when changing ports,
using a different Hub checkout, diagnosing a failure, or running browser tests.

- Check `cargo run -p dev-server -- status --json` first. Reuse a ready stack in the
  requested mode. Do not replace an existing running stack without a user request
  to restart or switch it.
- Start with `cargo run -p dev-server -- start --mode wasi --detach`. The sibling
  `../openid_connect_wasi` checkout is the default provider source. Use `--hub-dir`
  when it is elsewhere. Use `--mode native` for a native comparison.
- While building, inspect status and `cargo run -p dev-server -- logs`, and keep the
  user informed. Report `ready` only after the launcher completes its smoke checks.
  If startup fails, inspect the named service/build log, fix applicable local issues,
  and retry without changing unrelated Hub source or toolchain pins.
- Report the actual admin, portfolio, Hub, and API URLs from status, the mode, and
  the credential-file location. Do not paste seed logs, tokens, signing keys, or
  generated passwords into progress messages. Provide local test credentials when
  the user asks for them.
- Test with `cargo run -p dev-server -- test`; add `--browser` to exercise the real
  keycloak-js/PKCE browser round trip. Browser tests use the Hub's locked Playwright
  installation. If Chromium is missing, install it in WSL using the Hub's documented
  `npm --prefix front/admin exec playwright install chromium` command.
- Stop with `cargo run -p dev-server -- stop` when requested or when cleaning up a
  temporary validation run. This preserves volumes and credentials. Do not use
  broad `pkill`, `podman rm --all`, or delete unrelated runtime files.

`start` automatically installs locked npm dependencies, builds assets before
embedding them, builds with each repository's pinned Rust toolchain, starts the
services, and provisions only its private databases. `--skip-build` is for a restart
of already-built artifacts with unchanged embedded URL configuration; otherwise
build normally. A launch authorizes local test data, not a production deployment.
