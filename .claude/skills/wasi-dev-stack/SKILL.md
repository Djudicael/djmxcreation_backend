---
name: wasi-dev-stack
description: Build or change DJMXCreation's local-environment CLI (the dev-server crate), which starts Podman PostgreSQL/RustFS, runs migrations, provisions an isolated OpenID Connect WASI Hub realm, builds the frontends and WASI P3 components, serves them with wasmtime (or native binaries), exposes same-origin proxies, runs smoke/P3 runtime/Playwright checks, and stops or destroys the environment. Use when adding a service, container, command, seed, or check to the CLI, or changing start/stop/destroy/status/logs/test behaviour. For simply running the stack, use run-djmx-stack.
---

Read the canonical repository skill at `.agents/skills/wasi-dev-stack/SKILL.md` completely,
including the reference files it links, and follow it. Treat that file as the
canonical, model-independent skill definition.
