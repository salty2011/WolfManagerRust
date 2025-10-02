use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use http::{header, HeaderMap, HeaderName, Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use std::path::Path;
use std::time::Duration;
use tokio::net::UnixStream;
use tracing::{info, warn};

/// Configuration for the Wolf proxy client
#[derive(Debug, Clone)]
pub struct WolfProxyConfig {
    pub socket_path: String,
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
}

impl WolfProxyConfig {
    pub fn new(
        socket_path: String,
        connect_timeout_ms: u64,
        read_timeout_ms: u64,
    ) -> Self {
        Self {
            socket_path,
            connect_timeout: Duration::from_millis(connect_timeout_ms),
            read_timeout: Duration::from_millis(read_timeout_ms),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
        }
    }

    pub fn with_retry(mut self, attempts: u32, delay_ms: u64) -> Self {
        self.retry_attempts = attempts;
        self.retry_delay = Duration::from_millis(delay_ms);
        self
    }
}

/// Hop-by-hop headers that should not be forwarded
fn hop_by_hop_headers() -> Vec<HeaderName> {
    vec![
        header::CONNECTION,
        header::PROXY_AUTHENTICATE,
        header::PROXY_AUTHORIZATION,
        header::TE,
        header::TRAILER,
        header::TRANSFER_ENCODING,
        header::UPGRADE,
        HeaderName::from_static("keep-alive"),
    ]
}

/// Wolf API reverse proxy client over Unix Domain Socket
pub struct WolfProxyClient {
    config: WolfProxyConfig,
}

impl WolfProxyClient {
    pub fn new(config: WolfProxyConfig) -> Self {
        Self { config }
    }

    /// Check if Wolf socket is available and connectable
    pub async fn check_readiness(&self) -> Result<()> {
        let path = Path::new(&self.config.socket_path);
        if !path.exists() {
            return Err(anyhow!("wolf.sock not found at {}", self.config.socket_path));
        }

        // Try to connect
        tokio::time::timeout(
            self.config.connect_timeout,
            UnixStream::connect(&self.config.socket_path),
        )
        .await
        .context("connection timeout")?
        .context("failed to connect to wolf.sock")?;

        Ok(())
    }

    /// Proxy an HTTP request to Wolf over the Unix socket
    pub async fn proxy_request(
        &self,
        method: Method,
        uri: http::Uri,
        headers: HeaderMap,
        body: Bytes,
        client_ip: Option<String>,
    ) -> Result<Response<Incoming>> {
        let start = std::time::Instant::now();

        // Retry connection with exponential backoff
        let stream = {
            let mut attempt = 0;
            loop {
                attempt += 1;

                match tokio::time::timeout(
                    self.config.connect_timeout,
                    UnixStream::connect(&self.config.socket_path),
                )
                .await
                {
                    Ok(Ok(stream)) => break stream,
                    Ok(Err(e)) => {
                        if attempt >= self.config.retry_attempts {
                            return Err(anyhow::Error::from(e).context("failed to connect to wolf.sock after retries"));
                        }
                        warn!(
                            attempt = attempt,
                            max_attempts = self.config.retry_attempts,
                            "Wolf connection failed, retrying..."
                        );
                        tokio::time::sleep(self.config.retry_delay * attempt).await;
                    }
                    Err(_) => {
                        if attempt >= self.config.retry_attempts {
                            return Err(anyhow!("connection timeout after {} attempts", attempt));
                        }
                        warn!(
                            attempt = attempt,
                            max_attempts = self.config.retry_attempts,
                            "Wolf connection timeout, retrying..."
                        );
                        tokio::time::sleep(self.config.retry_delay * attempt).await;
                    }
                }
            }
        };

        let io = TokioIo::new(stream);

        // Build the request
        let mut req_builder = Request::builder()
            .method(method.clone())
            .uri(&uri);

        // Copy headers, filtering hop-by-hop headers
        let hop_headers = hop_by_hop_headers();
        for (name, value) in headers.iter() {
            if !hop_headers.contains(name) {
                req_builder = req_builder.header(name, value);
            }
        }

        // Add X-Forwarded-* headers
        if let Some(ip) = client_ip {
            req_builder = req_builder.header("x-forwarded-for", ip);
        }
        req_builder = req_builder.header("x-forwarded-proto", "http");
        if let Some(host) = headers.get(header::HOST) {
            req_builder = req_builder.header("x-forwarded-host", host);
        }

        let req = req_builder.body(Full::new(body))?;

        // Send request and get response
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                warn!("Wolf proxy connection error: {}", e);
            }
        });

        let response = tokio::time::timeout(
            self.config.read_timeout,
            sender.send_request(req),
        )
        .await
        .context("read timeout")??;

        let status = response.status();
        let elapsed = start.elapsed();

        info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = elapsed.as_millis(),
            "Wolf proxy request completed"
        );

        Ok(response)
    }

    /// Convert hyper Response to axum Response
    pub async fn response_to_axum(response: Response<Incoming>) -> Result<Response<axum::body::Body>> {
        let (parts, body) = response.into_parts();

        // Filter hop-by-hop headers from response
        let hop_headers = hop_by_hop_headers();
        let mut filtered_headers = HeaderMap::new();
        for (name, value) in parts.headers.iter() {
            if !hop_headers.contains(name) {
                filtered_headers.insert(name.clone(), value.clone());
            }
        }

        // Convert body
        let bytes = body.collect().await?.to_bytes();

        let mut response = Response::new(axum::body::Body::from(bytes));
        *response.status_mut() = parts.status;
        *response.headers_mut() = filtered_headers;
        *response.version_mut() = parts.version;

        Ok(response)
    }
}

/// Build error response with JSON payload
pub fn error_response(status: StatusCode, error: &str, detail: &str) -> Response<axum::body::Body> {
    let body = serde_json::json!({
        "error": error,
        "detail": detail,
    });

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}
