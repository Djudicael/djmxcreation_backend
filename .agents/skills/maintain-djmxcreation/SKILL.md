---
name: maintain-djmxcreation
description: Maintain the DJMXCreation Rust and JavaScript monorepo. Use when upgrading Cargo or npm dependencies, changing the Rust toolchain, fixing build or test failures, validating WASI Preview 2, or editing CI, Dependabot, and repository automation.
---

# Maintain DJMXCreation

Work from the repository root. Preserve unrelated user changes and inspect `git status` before editing.

## Use WSL

Run builds, tests, formatters, dependency commands, and package installation inside WSL. Convert the Windows repository path to `/mnt/<drive>/...` and use `wsl.exe --cd <linux-path> bash -lc '<command>'` when the host shell is PowerShell.

Load nvm before Node commands in this development environment:

```bash
source ~/.nvm/nvm.sh
```

Respect `rust-toolchain.toml`; do not silently use a different Rust release.

## Upgrade dependencies

1. Inspect every `Cargo.toml`, `package.json`, and lockfile before changing versions.
2. Check current registry versions. Include incompatible major versions only when the task requests full upgrades.
3. Keep shared Rust versions in `[workspace.dependencies]`. Update `wasi-pg-client` features for native and WASI targets together.
4. Update npm manifests and their corresponding lockfiles in `front/`, `front/admin/`, and `front/portfolio/`.
5. Run npm audit. Prefer a secure supported version over a vulnerable latest release and record intentional pins.
6. Fix source and test incompatibilities caused by upgrades; do not weaken assertions merely to make tests pass.

## Verify changes

Run the quick checks first:

```bash
source ~/.nvm/nvm.sh
npm test
npm run build
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo check --target wasm32-wasip2 -p djmxcreation-backend-wasi --lib
cargo build --locked --profile wasi-release --target wasm32-wasip2 -p djmxcreation-backend-wasi --lib
```

Then run the complete Rust suite:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The integration tests use Podman to launch PostgreSQL and RustFS. Confirm Podman is available before running them and report environmental failures separately from code failures.

## Maintain automation

Keep GitHub Actions aligned with the commands above. Use lockfile-aware caching, minimal permissions, pinned Rust and Node major versions, and artifact upload for tagged releases. Keep Dependabot entries for both Cargo and each independent npm lockfile.

Finish by reviewing `git diff --check`, `git status --short`, dependency audits, and all verification results. Report exact commands and any remaining limitation.
