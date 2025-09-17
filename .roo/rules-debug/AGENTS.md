# Project Debug Rules (Non-Obvious Only)

- Logs: enable JSON logs with tracing-subscriber and control via RUST_LOG (env-filter). Expect structured output in the API process; see [AGENTS.md](AGENTS.md:1).
- SQLx offline: CI/build failures due to type inference usually mean stale [crates/wm-storage/sqlx-data.json](crates/wm-storage/sqlx-data.json:1). Regenerate using the documented prepare command and commit it. See [AGENTS.md](AGENTS.md:1).
- SSE checks: verify snapshot-before-stream, Last-Event-ID replay from `events`, and a 15s heartbeat on /api/v1/events/stream when diagnosing sync issues. See [AGENTS.md](AGENTS.md:1).
- Adapters: wolf.sock issues are generally path/permissions. Confirm the configured socket path via [crates/wm-config/src/lib.rs](crates/wm-config/src/lib.rs:1). Docker API uses bollard over /var/run/docker.sock. See [AGENTS.md](AGENTS.md:1).
- Devcontainer note: if the claude-code CLI is missing, rebuild the container; it is provisioned by postCreate in [".devcontainer/devcontainer.json"](.devcontainer/devcontainer.json:1) and is not part of runtime.