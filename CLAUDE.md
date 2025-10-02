# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WolfManagerRust is a Rust-based service that normalizes and streams events from a wolf.sock Unix socket endpoint. The architecture follows a clean separation between API, domain logic, storage, adapters, and configuration.

## Common Commands

### Building and Running
```bash
# Build all workspace crates
cargo build

# Run the API service
cargo run -p wm-api

# Run with custom config via environment variables
WM_BIND_ADDR="127.0.0.1:3000" DATABASE_URL="sqlite://dev.db" cargo run -p wm-api

# Build in release mode
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p wm-core

# Run specific test
cargo test -p wm-api test_name
```

### Database Operations
```bash
# Migrations run automatically on startup when pool is initialized
# To prepare offline SQLx data for CI (when needed):
cargo sqlx prepare --workspace -- --all-targets

# Migrations are located in:
# crates/wm-storage/migrations/
```

### Development Utilities
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings
```

## Architecture

### Workspace Structure (Cargo Workspace)
- **wm-api**: Axum HTTP server, SSE endpoints, OpenAPI/Swagger UI, middlewares (CORS/Trace/Compression)
- **wm-core**: Domain types (UserId, ClientId, PairingId, SessionId), Event enum, normalization traits
- **wm-adapters**: Wolf.sock client (reqwest with unix-socket), Docker adapter (bollard)
- **wm-storage**: SQLx pool management, migrations, database repositories
- **wm-config**: Configuration loading from environment variables with defaults

### Event Flow
1. Global wolf.sock SSE reader consumes events from Unix socket
2. Events are normalized via the `Normalize` trait (wm-core)
3. Normalized events appended to `events` table (append-only)
4. Materialized current-state tables updated: `clients`, `pairings`, `sessions_current`
5. Per-user deltas published to RealtimeHub (DashMap-backed in-memory cache)
6. SSE endpoint `/api/v1/events/stream` streams updates to authenticated clients

### Database Schema
- **users**: User identities
- **sessions**: Authentication/login sessions (distinct from streaming sessions)
- **events**: Append-only normalized event log (kind, payload JSON, timestamp)
- **clients**, **pairings**, **sessions_current**: Materialized current-state tables

### Configuration
Configuration is loaded from environment variables with defaults defined in wm-config:
- `WM_BIND_ADDR` (default: "0.0.0.0:8080")
- `DATABASE_URL` (default: "sqlite://wm.db")
- `WM_WOLF_SOCK_PATH` (default: "/var/run/wolf.sock")
- `WM_DOCKER_SOCK_PATH` (default: "/var/run/docker.sock")

Defaults are also available in `config/default.toml` for reference.

### API Endpoints
All public APIs are versioned under `/api/v1`:
- `GET /healthz`: Health check
- `GET /api/v1/events/stream`: SSE stream (authenticated, snapshot + deltas + heartbeat every 15s)
- `GET /docs`: Swagger UI for OpenAPI documentation
- `GET /api/v1/openapi.json`: OpenAPI spec

### SSE Endpoint Behavior
- Authenticated endpoint (JWT or signed session cookie)
- Sends snapshot first (from DB/cache)
- Optional replay from `events` table via Last-Event-ID header
- Continuous per-user deltas with 15-second heartbeat

### Key Dependencies
- **Web**: axum 0.7, tokio (multi-thread), tower, tower-http
- **OpenAPI**: utoipa, utoipa-axum, utoipa-swagger-ui
- **Database**: sqlx (SQLite + Postgres support, runtime-tokio, macros, migrate, offline)
- **Adapters**: bollard (Docker), reqwest (with unix-socket for wolf.sock)
- **State/Cache**: dashmap (concurrent HashMap)
- **Observability**: tracing, tracing-subscriber (JSON logs)

## Important Context

### Current Repository State
The codebase has basic scaffolding in place with placeholder implementations. The core architecture is established but several features are marked as TODO or placeholders:
- Wolf.sock SSE stream consumption (wm-adapters/src/lib.rs:19)
- Database pool initialization is commented out in main (wm-api/src/main.rs:82-84)
- Realtime delta publishing logic is not yet implemented
- Authentication/authorization middleware is planned but not implemented

### Security Considerations
- Never expose wolf.sock directly to clients
- All events must be normalized and filtered through the domain layer
- Enforce per-user scoping at route/middleware layer
- Consider storing raw events optionally for debugging/auditing

### Development Notes
- Node/npm in `.devcontainer/` is only for Claude CLI support, not part of runtime stack
- Dependabot is scoped to devcontainer updates only
- Use Rust 1.80+ (specified in rust-toolchain.toml)
- SQLx offline mode for CI uses `crates/wm-storage/sqlx-data.json` (prepare when needed)
