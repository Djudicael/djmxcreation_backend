---
name: wasi-object-storage
description: Talk to S3-compatible object storage (RustFS, MinIO, AWS S3) from DJMXCreation's servers using aws-sdk-s3 — its default client natively, and a custom connector over app-http in WASI 0.3 (P3 async) components. Use when creating the storage client, uploading/downloading/deleting objects, generating presigned URLs, the direct browser multipart upload flow, media variants from the media worker, bucket CORS/provisioning, or debugging S3 calls under WASI. Do not use rust-s3 or reqwest-based clients in WASI code.
---

Read the canonical repository skill at `.agents/skills/wasi-object-storage/SKILL.md` completely,
including the reference files it links, and follow it. Treat that file as the
canonical, model-independent skill definition.
