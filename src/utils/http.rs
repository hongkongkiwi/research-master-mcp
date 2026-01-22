//! HTTP client utilities.

use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// Shared HTTP client with sensible defaults
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Arc<Client>,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        Self::with_user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
    }

    /// Create a new HTTP client with a custom user agent
    pub fn with_user_agent(user_agent: &str) -> Self {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
        }
    }

    /// Create from an existing reqwest Client
    pub fn from_client(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
