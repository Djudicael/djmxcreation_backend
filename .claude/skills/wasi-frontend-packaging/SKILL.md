---
name: wasi-frontend-packaging
description: Package DJMXCreation's built admin and portfolio frontends (front/<app>/dist) into the frontend-server crate, which serves the embedded static assets with SPA fallback as a native binary or a WASI 0.3 (P3 async) component. Use when changing the frontend server crate (currently djmxcreation-frontend-wasi), its build.rs asset table, the FRONTEND_APP selection, content types, cache or security headers, or the build order between npm and cargo.
---

Read the canonical repository skill at `.agents/skills/wasi-frontend-packaging/SKILL.md` completely,
including the reference files it links, and follow it. Treat that file as the
canonical, model-independent skill definition.
