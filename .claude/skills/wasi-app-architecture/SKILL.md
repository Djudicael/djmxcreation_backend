---
name: wasi-app-architecture
description: Target architecture of DJMXCreation — one Rust codebase that runs either as native binaries or as WASI 0.3 (P3 async) components on the wasm32-wasip2 target, with PostgreSQL through the latest wasi-pg-client, RustFS/S3 through aws-sdk-s3, a media worker, and two lit-html frontends (admin, portfolio). Use when adding a crate or feature, deciding where code belongs, choosing a library, refactoring the workspace toward this layout, or reviewing a change for structure. Start here; it routes to the specialised wasi-* skills.
---

Read the canonical repository skill at `.agents/skills/wasi-app-architecture/SKILL.md` completely,
including the reference files it links, and follow it. Treat that file as the
canonical, model-independent skill definition.
