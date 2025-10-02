# WolfManager

A web-based admin and orchestration frontend for [Wolf](https://games-on-whales.github.io/wolf/stable/) - the Moonlight/Sunshine game streaming platform.

WolfManager provides a modern HTTP API with real-time event streaming, database persistence, and transparent reverse proxy to Wolf's Unix socket API. Built in Rust for performance and reliability.

## Features

- **Wolf API Reverse Proxy** - Transparent proxy at `/wolfapi/*` forwarding to Wolf over Unix Domain Socket
  - Supports all HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS)
  - Server-Sent Events (SSE) streaming support
  - Automatic retry with exponential backoff for container startup delays
  - Configurable timeouts and retry behavior
  - Readiness check endpoint at `/wolfapi/_ready`

- **Real-time Event Streaming** - Server-Sent Events (SSE) endpoint with snapshot + delta updates
  - Per-user event streams at `/api/v1/events/stream`
  - 15-second heartbeat keep-alive
  - Optional event replay via Last-Event-ID header

- **Database Persistence** - SQLite and PostgreSQL support with automatic migrations
  - Append-only event log
  - Materialized current-state tables (clients, pairings, sessions)

- **LAN-First CORS** - Browser-friendly CORS designed for local network operation
  - Auto-detects server's local IP at startup
  - Allows private IP ranges by default (10.x, 172.16-31.x, 192.168.x)
  - Optional PUBLIC_URL support for Cloudflare/reverse proxy deployments

- **OpenAPI Documentation** - Auto-generated API docs served at `/openapi.json`

## Quick Start

### Prerequisites

- Rust 1.80+ (see `rust-toolchain.toml`)
- Wolf running with socket at `/var/run/wolf/wolf.sock` (or configure `WM_WOLF_SOCK_PATH`)

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd WolfManagerRust

# Build all workspace crates
cargo build

# Run the API service (uses defaults)
cargo run -p wm-api

# Run with custom configuration
WM_BIND_ADDR="127.0.0.1:3000" \
DATABASE_URL="sqlite://dev.db" \
cargo run -p wm-api
```

The API will be available at `http://localhost:8080` (or your configured bind address).

## API Endpoints

- `GET /healthz` - Health check
- `GET /api/v1/ping` - Ping with database health check
- `GET /api/v1/events/stream` - Server-Sent Events stream (authenticated)
- `GET /openapi.json` - OpenAPI specification
- `ALL /wolfapi/*` - Transparent proxy to Wolf socket
- `GET /wolfapi/_ready` - Wolf readiness check

## Configuration

WolfManager is configured via environment variables. See **[docs/Variables.md](docs/Variables.md)** for complete documentation.

### Key Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WM_BIND_ADDR` | `0.0.0.0:8080` | Server bind address |
| `DATABASE_URL` | `sqlite://wm.db` | Database connection string |
| `WM_WOLF_SOCK_PATH` | `/var/run/wolf/wolf.sock` | Path to Wolf Unix socket |
| `PUBLIC_URL` | _none_ | Public-facing URL for CORS (e.g., `https://app.example.com`) |
| `WM_ALLOW_PRIVATE_ORIGINS` | `true` | Allow CORS from private IPs (LAN mode) |

## Project Structure

This is a Cargo workspace with the following crates:

- **wm-api** - HTTP server (Axum), SSE endpoints, Wolf reverse proxy, OpenAPI docs
- **wm-core** - Domain types, event normalization traits
- **wm-adapters** - External integrations (Wolf proxy client, Docker adapter)
- **wm-storage** - Database layer (SQLx), migrations, repositories
- **wm-config** - Configuration management with environment variables

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p wm-api

# Run CORS tests specifically
cargo test -p wm-api middleware::cors::tests
```

### Database Migrations

Migrations run automatically on startup when the database pool is initialized. Migration files are located in `crates/wm-storage/migrations/`.

### Code Quality

```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Architecture

### Event Flow

1. Global Wolf socket SSE reader consumes events from Unix socket
2. Events normalized via `Normalize` trait (wm-core)
3. Normalized events appended to `events` table (append-only)
4. Materialized current-state tables updated
5. Per-user deltas published to RealtimeHub (in-memory cache)
6. SSE endpoint streams updates to authenticated clients

### Database Schema

- **users** - User identities
- **sessions** - Authentication/login sessions
- **events** - Append-only normalized event log
- **clients**, **pairings**, **sessions_current** - Materialized current-state tables

## License

MIT

## Contributing

See [CLAUDE.md](CLAUDE.md) for development context and architecture notes.
