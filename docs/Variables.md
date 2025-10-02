# Environment Variables

WolfManager can be configured using environment variables. All variables have sensible defaults for local development.

## Server Configuration

### `WM_BIND_ADDR`
- **Description**: Address and port the API server binds to
- **Default**: `0.0.0.0:8080`
- **Example**: `WM_BIND_ADDR=127.0.0.1:3000`

### `DATABASE_URL`
- **Description**: Database connection string (SQLite or PostgreSQL)
- **Default**: `sqlite://wm.db`
- **Examples**:
  - SQLite: `DATABASE_URL=sqlite:///var/lib/wm/data.db`
  - PostgreSQL: `DATABASE_URL=postgres://user:pass@localhost/wmdb`

## Wolf Integration

### `WM_WOLF_SOCK_PATH`
- **Description**: Path to Wolf Unix domain socket
- **Default**: `/var/run/wolf/wolf.sock`
- **Example**: `WM_WOLF_SOCK_PATH=/tmp/wolf.sock`

### `WM_WOLF_PROXY_CONNECT_TIMEOUT_MS`
- **Description**: Connection timeout for Wolf socket in milliseconds
- **Default**: `2000` (2 seconds)
- **Example**: `WM_WOLF_PROXY_CONNECT_TIMEOUT_MS=5000`

### `WM_WOLF_PROXY_READ_TIMEOUT_MS`
- **Description**: Read timeout for Wolf proxy requests in milliseconds
- **Default**: `10000` (10 seconds)
- **Example**: `WM_WOLF_PROXY_READ_TIMEOUT_MS=30000`

### `WM_WOLF_PROXY_RETRY_ATTEMPTS`
- **Description**: Number of retry attempts for Wolf socket connection (useful during container startup)
- **Default**: `3`
- **Example**: `WM_WOLF_PROXY_RETRY_ATTEMPTS=5`

### `WM_WOLF_PROXY_RETRY_DELAY_MS`
- **Description**: Base delay between retry attempts in milliseconds (uses exponential backoff)
- **Default**: `500` (0.5 seconds)
- **Example**: `WM_WOLF_PROXY_RETRY_DELAY_MS=1000`

## Docker Integration

### `WM_DOCKER_SOCK_PATH`
- **Description**: Path to Docker Unix domain socket
- **Default**: `/var/run/docker.sock`
- **Example**: `WM_DOCKER_SOCK_PATH=/var/run/docker.sock`

## CORS Configuration

### `PUBLIC_URL`
- **Description**: Exact external public origin for the Web UI (scheme+host+port). Used when app is exposed via Cloudflare or reverse proxy.
- **Default**: _None_
- **Examples**:
  - `PUBLIC_URL=https://app.example.com`
  - `PUBLIC_URL=http://localhost:5173`

### `WM_ALLOW_PRIVATE_ORIGINS`
- **Description**: Allow CORS requests from any private IPv4 address (10.x.x.x, 172.16-31.x.x, 192.168.x.x). Designed for LAN-first operation.
- **Default**: `true`
- **Values**: `true`, `false`, `1`, or `0`
- **Example**: `WM_ALLOW_PRIVATE_ORIGINS=false` (restrict to detected local IP, localhost, and PUBLIC_URL only)

## CORS Behavior

WolfManager uses a layered CORS policy designed for LAN-first operation with optional public URL support:

1. **Detected Local IPs** - Auto-detected at startup (any port). Server detects its local interface IP and automatically allows origins from that IP.
   - Example: If server is at `192.168.1.100`, allows `http://192.168.1.100:*`

2. **Localhost & Loopback** - Always allowed (any port)
   - Examples: `http://localhost:3000`, `http://127.0.0.1:5173`, `http://[::1]:8080`

3. **Private IPv4 Ranges** - Allowed by default when `WM_ALLOW_PRIVATE_ORIGINS=true`
   - Ranges: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` (any port)
   - Example: `http://192.168.1.50:5173`, `http://10.0.0.5:3000`

4. **PUBLIC_URL** - Exact match (scheme, host, and port must match)
   - Example: `PUBLIC_URL=https://app.example.com` only allows `https://app.example.com` (not `http://` or `:8080`)

## Example Configurations

### Local Development (Default)
```bash
# All defaults work out-of-box
cargo run -p wm-api
```

### Production with Cloudflare
```bash
export PUBLIC_URL=https://wolf.example.com
export WM_ALLOW_PRIVATE_ORIGINS=false  # Restrict to public URL only
export DATABASE_URL=postgres://user:pass@db.internal/wolfmanager
export WM_BIND_ADDR=0.0.0.0:8080
cargo run -p wm-api --release
```

### LAN-only Deployment
```bash
export WM_ALLOW_PRIVATE_ORIGINS=true  # Allow all LAN IPs (default)
export DATABASE_URL=sqlite:///var/lib/wolfmanager/wm.db
export WM_BIND_ADDR=0.0.0.0:8080
cargo run -p wm-api --release
```

### Development with Custom Wolf Socket
```bash
export WM_WOLF_SOCK_PATH=/tmp/wolf.sock
export WM_WOLF_PROXY_RETRY_ATTEMPTS=5
export WM_WOLF_PROXY_RETRY_DELAY_MS=1000
cargo run -p wm-api
```
