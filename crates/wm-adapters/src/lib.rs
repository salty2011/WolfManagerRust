use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use tracing::info;

pub struct WolfClient {
    http: Client,
    pub sock_path: String,
}

impl WolfClient {
    pub fn new(sock_path: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { http, sock_path: sock_path.into() })
    }

    // Placeholder: implement SSE stream consumption using unix-socket when wolf API is known.
    pub async fn health(&self) -> Result<()> {
        info!("wolf client using socket: {}", self.sock_path);
        Ok(())
    }
}

// Docker adapter placeholder via bollard; configure later as needed.
pub mod docker {
    use anyhow::Result;
    use bollard::Docker;

    pub async fn connect(sock_path: &str) -> Result<Docker> {
        // bollard defaults to /var/run/docker.sock via Docker::connect_with_unix_defaults()
        let docker = Docker::connect_with_unix(sock_path, 120, bollard::API_DEFAULT_VERSION)?;
        Ok(docker)
    }
}