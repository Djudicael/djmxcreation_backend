# Repository agent guidance

These instructions apply to every coding agent working in this repository.

## Required skill

Use the `maintain-djmxcreation` skill for dependency upgrades, Rust toolchain changes, builds, tests, WASI work, CI/CD, Dependabot, and repository maintenance:

- Canonical Agent Skills definition: `.agents/skills/maintain-djmxcreation/SKILL.md`
- Claude Code discovery adapter: `.claude/skills/maintain-djmxcreation/SKILL.md`
- OpenCode discovers the canonical `.agents/skills` location directly.

Read the canonical skill completely before doing matching work. Keep the Claude adapter description synchronized if the canonical trigger description changes.

## Architecture skills

For structural work (where code belongs, new features, refactors, crates, the WASI
P3 entry points, PostgreSQL, object storage, frontends, the dev CLI, and tests),
start with `wasi-app-architecture`; it routes to the specialised skills:

| Skill | Scope |
|---|---|
| `wasi-app-architecture` | target architecture, crate boundaries, library choices, refactor map |
| `wasi-backend-api` | API server (native or WASI), `app-http` runtime adapters, auth, middleware |
| `wasi-postgres` | `wasi-pg-client` repositories, transactions, migrations |
| `wasi-object-storage` | `aws-sdk-s3` on both runtimes, uploads, presigned URLs |
| `wasi-frontend` | admin and portfolio Web Components + lit-html |
| `wasi-frontend-packaging` | serving the built frontends from `frontend-server` |
| `wasi-testing` | test levels, verification checklist, CI |
| `wasi-dev-stack` | changing the local-environment CLI |

Canonical definitions are in `.agents/skills/<name>/SKILL.md`; Claude Code adapters
in `.claude/skills/<name>/SKILL.md` point to them. Keep adapter descriptions
synchronized with the canonical frontmatter.

## Project conventions

For starting, stopping, inspecting, or testing the complete local stack, use
`.agents/skills/run-djmx-stack/SKILL.md`. Its launcher supports WASI and native
application servers and provisions an isolated OpenID Connect Hub realm.

- Run all build, test, formatting, linting, and dependency commands in WSL.
- Use Rust 1.97.1 from `rust-toolchain.toml` and the `wasm32-wasip2` target.
- Load `~/.nvm/nvm.sh` before Node/npm commands when nvm is installed.
- Keep Rust dependency versions centralized in the workspace manifest where practical.
- Preserve unrelated changes and never replace lockfiles without updating their matching manifests.
- Keep admin UI text in English. Do not translate the interface based on the conversation language; implement localization only when requested.
- Keep public documentation deployment-oriented and machine-independent. Put local development setup details in the CLI guide and avoid workstation-specific paths in public examples.
- Treat PostgreSQL and RustFS integration tests as Podman-backed tests; distinguish unavailable infrastructure from application failures.
