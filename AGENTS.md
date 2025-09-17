# AGENTS.md

This file provides guidance to agents when working with code in this repository.

Technology stack (authoritative plan; to be codified in [Cargo.toml](Cargo.toml:1)):
- Rust 1.80+, Axum 0.7, Tokio (multi-thread), Tower/tower-http (CORS, trace, compression)
- SQLx (SQLite + Postgres; runtime-tokio, macros, migrate, offline for CI)
- OpenAPI: utoipa + utoipa-axum (+ Swagger UI)
- Adapters: bollard (Docker), reqwest (with unix-socket) for wolf.sock
- State/cache: dashmap; Observability: tracing + tracing-subscriber (JSON logs)

Workspace architecture (planned crates and responsibilities):
- [crates/wm-api/src/main.rs](crates/wm-api/src/main.rs:1): Axum app (routes, auth, SSE, OpenAPI), middlewares (CORS/Trace/Compression)
- [crates/wm-core/src/lib.rs](crates/wm-core/src/lib.rs:1): domain/business logic, event normalization traits/types
- [crates/wm-adapters/src/lib.rs](crates/wm-adapters/src/lib.rs:1): wolf.sock client (reqwest unix-socket) and Docker adapter (bollard)
- [crates/wm-storage/src/lib.rs](crates/wm-storage/src/lib.rs:1): SQLx pool, migrations, repositories
- [crates/wm-config/src/lib.rs](crates/wm-config/src/lib.rs:1): serde config (env + TOML), e.g., DB URL, wolf.sock path, docker socket path

Event ingestion and state flow:
- Single global wolf.sock SSE reader → normalize events → append-only [events] table
- Update materialized current-state tables (clients, pairings, sessions)
- Publish per-user deltas to a RealtimeHub

SSE endpoint contract (under /api/v1):
- Authenticated
- Sends snapshot first (from DB/cache)
- Optional replay from `events` via Last-Event-ID
- Continuous per-user deltas + heartbeat every 15s

Auth and routing:
- Start simple (JWT or signed session cookie); enforce per-user scoping at route/middleware layer
- All public API versioned under /api/v1
- Security: never expose wolf.sock directly; normalize/filter events; optionally store raw

SQLx and migrations (planned structure):
- Offline mode for CI with prepared data file [crates/wm-storage/sqlx-data.json](crates/wm-storage/sqlx-data.json:1)
- Migrations in [crates/wm-storage/migrations/0001_init.sql](crates/wm-storage/migrations/0001_init.sql:1) for users, sessions, events, clients, pairings, sessions_current

Repo state vs plan (important context):
- Current repo is code-light (no [Cargo.toml](Cargo.toml:1) yet); this document captures the target architecture to implement next
- Node/npm and the Claude CLI noted in [".devcontainer/devcontainer.json"](.devcontainer/devcontainer.json:1) exist only to support the CLI; they are not part of the WolfManagerRust runtime stack
- Dependabot is scoped to devcontainers updates only [.github/dependabot.yml](.github/dependabot.yml:1)