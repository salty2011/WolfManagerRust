mod routes;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, sse::{Sse, Event}},
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};
use futures_util::stream;
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

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/v1/events/stream", get(events_stream))
        .route("/api/v1/ping", get(ping))
        .route("/openapi.json", get(|| async move { Json(api) }))
        .with_state(state)
        .nest("/wolfapi", wolf_router);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    info!("Listening on {}", config.bind_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}