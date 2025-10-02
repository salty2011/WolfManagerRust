use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, Uri},
    response::Response,
    routing::any,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, warn};
use wm_adapters::wolf_proxy::{error_response, WolfProxyClient};

#[derive(Clone)]
pub struct WolfProxyState {
    pub client: Arc<WolfProxyClient>,
}

/// Health check endpoint for Wolf socket readiness
async fn wolf_ready(State(state): State<WolfProxyState>) -> Response {
    match state.client.check_readiness().await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(r#"{"status":"ok"}"#))
            .unwrap(),
        Err(e) => {
            warn!("Wolf readiness check failed: {}", e);
            error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "UpstreamUnavailable",
                &format!("wolf.sock not reachable: {}", e),
            )
        }
    }
}

/// Catch-all proxy handler for Wolf API
async fn wolf_proxy(
    State(state): State<WolfProxyState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    // Extract request details
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // Check for WebSocket upgrade
    if headers
        .get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
    {
        warn!("WebSocket upgrade attempted on Wolf proxy - not yet supported");
        return error_response(
            StatusCode::NOT_IMPLEMENTED,
            "NotImplemented",
            "WebSocket proxying is not yet supported",
        );
    }

    // Strip /wolfapi prefix from URI
    let stripped_path = uri
        .path()
        .strip_prefix("/wolfapi")
        .unwrap_or(uri.path());

    // Reconstruct URI with stripped path
    let new_uri = if let Some(query) = uri.query() {
        format!("{}?{}", stripped_path, query)
    } else {
        stripped_path.to_string()
    };

    let new_uri = match new_uri.parse::<Uri>() {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to parse URI: {}", e);
            return error_response(
                StatusCode::BAD_REQUEST,
                "InvalidUri",
                &format!("Failed to parse URI: {}", e),
            );
        }
    };

    // Extract body
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return error_response(
                StatusCode::BAD_REQUEST,
                "InvalidBody",
                &format!("Failed to read request body: {}", e),
            );
        }
    };

    // Get client IP
    let client_ip = Some(addr.ip().to_string());

    // Proxy the request
    match state
        .client
        .proxy_request(method, new_uri, headers, body, client_ip)
        .await
    {
        Ok(response) => {
            // Convert hyper response to axum response
            match WolfProxyClient::response_to_axum(response).await {
                Ok(axum_response) => axum_response,
                Err(e) => {
                    error!("Failed to convert response: {}", e);
                    error_response(
                        StatusCode::BAD_GATEWAY,
                        "ResponseConversionError",
                        &format!("Failed to convert upstream response: {}", e),
                    )
                }
            }
        }
        Err(e) => {
            error!("Wolf proxy request failed: {}", e);

            // Determine appropriate error response
            let error_msg = e.to_string();
            if error_msg.contains("timeout") {
                error_response(
                    StatusCode::GATEWAY_TIMEOUT,
                    "UpstreamTimeout",
                    &format!("Wolf API request timed out: {}", e),
                )
            } else if error_msg.contains("connection") || error_msg.contains("connect") {
                error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "UpstreamUnavailable",
                    &format!("Failed to connect to wolf.sock: {}", e),
                )
            } else {
                error_response(
                    StatusCode::BAD_GATEWAY,
                    "UpstreamError",
                    &format!("Wolf API request failed: {}", e),
                )
            }
        }
    }
}

/// Create Wolf API proxy router
pub fn wolf_router(client: Arc<WolfProxyClient>) -> Router {
    let state = WolfProxyState { client };

    Router::new()
        .route("/_ready", any(wolf_ready))
        .fallback(wolf_proxy)
        .with_state(state)
}
