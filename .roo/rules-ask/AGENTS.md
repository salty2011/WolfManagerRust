# Project Documentation Rules (Non-Obvious Only)

- Treat [AGENTS.md](AGENTS.md:1) as the canonical spec: stack, crates, event flow, SSE contract, auth/routing, and SQLx offline/migrations. Do not present Node/npm/Claude CLI as part of the runtime stack (they are tooling-only).
- Public API must be versioned under /api/v1 and the SSE endpoint must document: snapshot-first, optional Last-Event-ID replay from `events`, and 15s heartbeat. See [AGENTS.md](AGENTS.md:1).
- Security guidance to emphasize: never expose wolf.sock directly; normalize/filter events before publishing deltas. See [AGENTS.md](AGENTS.md:1).
- Schema/migrations documentation should live with the storage crate once added (e.g., [crates/wm-storage/migrations/0001_init.sql](crates/wm-storage/migrations/0001_init.sql:1)) and reference the SQLx offline file [crates/wm-storage/sqlx-data.json](crates/wm-storage/sqlx-data.json:1). See [AGENTS.md](AGENTS.md:1).