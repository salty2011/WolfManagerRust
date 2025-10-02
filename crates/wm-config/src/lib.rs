use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bind_addr: String,
    pub db_url: String,
    pub wolf_sock_path: String,
    pub docker_sock_path: String,
    pub wolf_proxy_connect_timeout_ms: u64,
    pub wolf_proxy_read_timeout_ms: u64,
    pub wolf_proxy_retry_attempts: u32,
    pub wolf_proxy_retry_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".into(),
            db_url: "sqlite://wm.db".into(),
            wolf_sock_path: "/var/run/wolf/wolf.sock".into(),
            docker_sock_path: "/var/run/docker.sock".into(),
            wolf_proxy_connect_timeout_ms: 2000,
            wolf_proxy_read_timeout_ms: 10000,
            wolf_proxy_retry_attempts: 3,
            wolf_proxy_retry_delay_ms: 500,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut cfg = Self::default();
        if let Ok(v) = env::var("WM_BIND_ADDR") {
            if !v.is_empty() {
                cfg.bind_addr = v;
            }
        }
        if let Ok(v) = env::var("DATABASE_URL") {
            if !v.is_empty() {
                cfg.db_url = v;
            }
        }
        if let Ok(v) = env::var("WM_WOLF_SOCK_PATH") {
            if !v.is_empty() {
                cfg.wolf_sock_path = v;
            }
        }
        if let Ok(v) = env::var("WM_DOCKER_SOCK_PATH") {
            if !v.is_empty() {
                cfg.docker_sock_path = v;
            }
        }
        if let Ok(v) = env::var("WM_WOLF_PROXY_CONNECT_TIMEOUT_MS") {
            if let Ok(parsed) = v.parse::<u64>() {
                cfg.wolf_proxy_connect_timeout_ms = parsed;
            }
        }
        if let Ok(v) = env::var("WM_WOLF_PROXY_READ_TIMEOUT_MS") {
            if let Ok(parsed) = v.parse::<u64>() {
                cfg.wolf_proxy_read_timeout_ms = parsed;
            }
        }
        if let Ok(v) = env::var("WM_WOLF_PROXY_RETRY_ATTEMPTS") {
            if let Ok(parsed) = v.parse::<u32>() {
                cfg.wolf_proxy_retry_attempts = parsed;
            }
        }
        if let Ok(v) = env::var("WM_WOLF_PROXY_RETRY_DELAY_MS") {
            if let Ok(parsed) = v.parse::<u64>() {
                cfg.wolf_proxy_retry_delay_ms = parsed;
            }
        }
        Ok(cfg)
    }
}