mod middleware;
mod routes;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, sse::{Sse, Event}},
    routing::{any, get},
    Json, Router,
};
use http::{Method, header, HeaderName, HeaderValue};
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};
use futures_util::stream;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;

use wm_adapters::wolf_proxy::{WolfProxyClient, WolfProxyConfig};
use wm_config::Config;
use wm_storage::{new_pool, migrate};

#[derive(Clone)]
struct AppState {
    pool: sqlx::SqlitePool,
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "OK")
    )
)]
async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

#[utoipa::path(
    get,
    path = "/api/v1/events/stream",
    responses(
        (status = 200, description = "SSE stream")
    )
)]
async fn events_stream(State(_state): State<AppState>) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let tick_stream = stream::unfold(tokio::time::interval(Duration::from_secs(5)), |mut interval| async move {
        interval.tick().await;
        Some((Ok(Event::default().data(json!({"type": "heartbeat"}).to_string())), interval))
    });

    Sse::new(tick_stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

#[utoipa::path(
    get,
    path = "/api/v1/ping",
    responses(
        (status = 200, description = "Ping with DB check"),
        (status = 500, description = "Database error")
    )
)]
async fn ping(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Test DB connection with simple query
    let result: Result<i64, _> = sqlx::query_scalar("SELECT 1")
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(_) => Ok(Json(json!({"ok": true, "db": "up"}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(healthz, events_stream, ping),
    components(schemas()),
    tags(
        (name = "wm-api", description = "WolfManager API")
    )
)]
struct ApiDoc;

/// Build CORS layer with browser-friendly origin checking
fn build_cors_layer(config: &Config) -> CorsLayer {
    let public_url = config.public_url.clone();
    let allow_private = config.allow_private_origins;

    // Detect local IPs at startup for CORS allowlist
    let local_ips = middleware::cors::detect_local_ips();

    // Create origin predicate that checks if browser's Origin header is allowed
    let origin_pred = AllowOrigin::predicate(move |origin: &HeaderValue, _req| {
        middleware::cors::origin_allowed(origin, public_url.as_deref(), &local_ips, allow_private)
    });

    CorsLayer::new()
        .allow_origin(origin_pred)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-api-key"),
            HeaderName::from_static("x-requested-with"),
        ])
        .max_age(Duration::from_secs(3600))
        .allow_credentials(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing (JSON logs)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_current_span(false)
        .without_time()
        .init();

    let config = Config::load()?;
    info!("Starting wm-api on {}", config.bind_addr);

    // Initialize DB
    let pool = new_pool(&config.db_url).await?;
    migrate(&pool).await?;

    let state = AppState {
        pool: pool.clone(),
    };

    // Build a regular Router with manual OpenAPI serving
    let api = ApiDoc::openapi();

    // Create Wolf proxy client
    let wolf_config = WolfProxyConfig::new(
        config.wolf_sock_path.clone(),
        config.wolf_proxy_connect_timeout_ms,
        config.wolf_proxy_read_timeout_ms,
    )
    .with_retry(
        config.wolf_proxy_retry_attempts,
        config.wolf_proxy_retry_delay_ms,
    );
    let wolf_client = Arc::new(WolfProxyClient::new(wolf_config));
    let wolf_router = routes::wolf::wolf_router(wolf_client);

    // Build CORS layer
    let cors = build_cors_layer(&config);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/events/stream", get(events_stream))
        .route("/api/v1/ping", get(ping))
        .route("/openapi.json", get(|| async move { Json(api) }))
        .with_state(state)
        .nest("/wolfapi", wolf_router)
        .fallback(any(|| async { "" })) // Catch-all for OPTIONS preflight
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    info!("Listening on {}", config.bind_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}