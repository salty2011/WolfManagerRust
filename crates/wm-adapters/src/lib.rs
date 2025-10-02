use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::stream;
use http::Method;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for Wolf API communication (passthrough + SSE streaming)
#[async_trait]
pub trait WolfApi: Send + Sync {
    async fn send_passthrough(
        &self,
        method: Method,
        path: &str,
        body: Option<Bytes>,
    ) -> Result<Bytes>;

    async fn sse_stream(
        &self,
        path: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>;
}

/// Mock implementation for testing and scaffolding
#[derive(Default)]
pub struct MockWolfApi;

#[async_trait]
impl WolfApi for MockWolfApi {
    async fn send_passthrough(
        &self,
        _method: Method,
        _path: &str,
        _body: Option<Bytes>,
    ) -> Result<Bytes> {
        // Return canned JSON response
        Ok(Bytes::from_static(b"{\"mock\":true}"))
    }

    async fn sse_stream(
        &self,
        _path: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        // Return dummy stream with one event
        Ok(Box::pin(stream::iter(vec![Ok(Bytes::from_static(
            b"data: {\"type\":\"mock\"}\n\n",
        ))])))
    }
}

/// Smart constructor for mock implementation
pub fn mock_wolf() -> Arc<dyn WolfApi> {
    Arc::new(MockWolfApi::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_passthrough() -> Result<()> {
        let client = mock_wolf();
        let response = client
            .send_passthrough(Method::GET, "/test", None)
            .await?;

        assert_eq!(response, Bytes::from_static(b"{\"mock\":true}"));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_sse_stream() -> Result<()> {
        use futures_util::StreamExt;

        let client = mock_wolf();
        let mut stream = client.sse_stream("/events").await?;

        let event = stream.next().await;
        assert!(event.is_some());

        Ok(())
    }
}