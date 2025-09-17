# Project Coding Rules (Non-Obvious Only)

- Source of truth: follow the stack, contracts, and crate responsibilities defined in [AGENTS.md](AGENTS.md:1). Node/npm and the Claude CLI in [".devcontainer/devcontainer.json"](.devcontainer/devcontainer.json:1) are tooling-only, not runtime.
- SSE contract: implement GET /api/v1/events/stream to (1) send a snapshot first (from DB/cache), (2) optionally replay from the append-only `events` table using Last-Event-ID, then (3) stream per-user deltas with a 15s heartbeat. Must be non-blocking on Tokio multi-thread. See [AGENTS.md](AGENTS.md:1).
- Security: never expose wolf.sock directly from the API; access it through an adapter abstraction in [crates/wm-adapters/src/lib.rs](crates/wm-adapters/src/lib.rs:1) (reqwest with unix-socket). See [AGENTS.md](AGENTS.md:1).
- Storage: write normalized events to `events` (append-only) and maintain current-state tables (clients, pairings, sessions). Apply migrations on startup and keep SQLx offline data at [crates/wm-storage/sqlx-data.json](crates/wm-storage/sqlx-data.json:1) up to date. See [AGENTS.md](AGENTS.md:1).
- API versioning: expose only versioned routes under /api/v1 and keep OpenAPI (utoipa + utoipa-axum) current with the live routes. See [AGENTS.md](AGENTS.md:1).
- Configuration: read DB URL, wolf.sock path, Docker socket path, and bind address via [crates/wm-config/src/lib.rs](crates/wm-config/src/lib.rs:1). Avoid scattered ad-hoc env reads. See [AGENTS.md](AGENTS.md:1).
- Observability: prefer tracing + tracing-subscriber (JSON logs) over println!. See [AGENTS.md](AGENTS.md:1).