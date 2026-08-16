# Repository agent guidance

These instructions apply to every coding agent working in this repository.

## Required skill

Use the `maintain-djmxcreation` skill for dependency upgrades, Rust toolchain changes, builds, tests, WASI work, CI/CD, Dependabot, and repository maintenance:

- Canonical Agent Skills definition: `.agents/skills/maintain-djmxcreation/SKILL.md`
- Claude Code discovery adapter: `.claude/skills/maintain-djmxcreation/SKILL.md`
- OpenCode discovers the canonical `.agents/skills` location directly.

Read the canonical skill completely before doing matching work. Keep the Claude adapter description synchronized if the canonical trigger description changes.

## Project conventions

- Run all build, test, formatting, linting, and dependency commands in WSL.
- Use Rust 1.97.1 from `rust-toolchain.toml` and the `wasm32-wasip2` target.
- Load `~/.nvm/nvm.sh` before Node/npm commands when nvm is installed.
- Keep Rust dependency versions centralized in the workspace manifest where practical.
- Preserve unrelated changes and never replace lockfiles without updating their matching manifests.
- Treat PostgreSQL and RustFS integration tests as Podman-backed tests; distinguish unavailable infrastructure from application failures.
