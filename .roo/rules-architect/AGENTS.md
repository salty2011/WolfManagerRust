# Project Architecture Rules (Non-Obvious Only)

- Event pipeline: a single global wolf.sock SSE reader → normalize → append to `events` (append-only) → update materialized current-state tables → publish per-user deltas via a RealtimeHub. See [AGENTS.md](AGENTS.md:1).
- Layering and responsibilities (enforce crate boundaries from [AGENTS.md](AGENTS.md:1)):
  - [crates/wm-core](crates/wm-core/src/lib.rs:1): domain/business logic and event normalization traits/types
  - [crates/wm-adapters](crates/wm-adapters/src/lib.rs:1): wolf.sock (reqwest unix-socket) and Docker (bollard)
  - [crates/wm-storage](crates/wm-storage/src/lib.rs:1): SQLx pool, migrations, repositories
  - [crates/wm-config](crates/wm-config/src/lib.rs:1): configuration (env + TOML)
  - [crates/wm-api](crates/wm-api/src/main.rs:1): Axum app (routes, auth, SSE, OpenAPI) with CORS/Trace/Compression
- SSE endpoint constraints under /api/v1: send snapshot first, optionally replay from `events` via Last-Event-ID, then continuous per-user deltas + heartbeat every 15s. See [AGENTS.md](AGENTS.md:1).
- Auth and routing: start simple (JWT or signed session cookie); enforce per-user scoping at route/middleware; all public routes versioned under /api/v1. See [AGENTS.md](AGENTS.md:1).
- Security: never expose wolf.sock directly; normalize/filter events; optionally store raw. See [AGENTS.md](AGENTS.md:1).
- CI posture: use SQLx offline mode for deterministic builds (prepare [crates/wm-storage/sqlx-data.json](crates/wm-storage/sqlx-data.json:1)). See [AGENTS.md](AGENTS.md:1).