---
name: wasi-backend-api
description: Build or change the DJMXCreation API server (app-api, currently the djmxcreation-backend-wasi crate) and the app-http runtime-adapter crate. One axum router runs as a native tokio binary or as a WASI 0.3 (P3 async) component through the wasip3 HTTP handler. Use when adding routes, handlers, middleware, OIDC bearer-token authentication, configuration, error-to-HTTP mapping, health or metrics endpoints, outbound HTTP calls, sleeps or timeouts, the ServiceRegister composition root, or the native/P3 entry points of any server.
---

Read the canonical repository skill at `.agents/skills/wasi-backend-api/SKILL.md` completely,
including the reference files it links, and follow it. Treat that file as the
canonical, model-independent skill definition.
