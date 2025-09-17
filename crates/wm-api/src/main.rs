use axum::{
    extract::State,
    response::{IntoResponse, Response, sse::{Sse, Event}},
    routing::get,
    Json, Router,
};
use dashmap::DashMap;
use std::{convert::Infallible, time::Duration, sync::Arc};
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tower_http::{cors::CorsLayer, trace::TraceLayer, compression::CompressionLayer};
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use wm_config::Config;
use wm_storage::{new_pool, migrate};

#[derive(Clone)]
struct AppState {
    realtime: Arc<DashMap<String, String>>,
    config: Config,
    // Using sqlx AnyPool behind an Arc would be ideal; keep simple here.
    // pool: sqlx::AnyPool,
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
    let mut ticker = IntervalStream::new(interval(Duration::from_secs(5)))
        .map(|_| Ok(Event::default().json_data(serde_json::json!({"type": "heartbeat"})).unwrap()));
    Sse::new(async_stream::stream! {
        while let Some(ev) = ticker.next().await {
            yield ev;
        }
    })
    .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

#[derive(OpenApi)]
#[openapi(
    paths(healthz, events_stream),
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

    // Initialize DB (optional at bootstrap; uncomment when needed)
    // let pool = new_pool(&config.db_url).await?;
    // migrate(&pool).await?;

    let state = AppState {
        realtime: Arc::new(DashMap::new()),
        config,
        // pool,
    };

    let api = OpenApiRouter::new()
        .routes(
            Router::new()
                .route("/healthz", get(healthz))
                .route("/api/v1/events/stream", get(events_stream))
        )
        .with_openapi(ApiDoc::openapi());

    let app = Router::new()
        .merge(api)
        .merge(SwaggerUi::new("/docs").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::Server::bind(&config.bind_addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}