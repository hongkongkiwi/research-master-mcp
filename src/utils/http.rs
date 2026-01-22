//! HTTP client utilities with rate limiting support.

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use reqwest::{header, Client, StatusCode};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::models::{DownloadRequest, DownloadResult};
use crate::sources::SourceError;

/// Default rate limit: requests per second
const DEFAULT_REQUESTS_PER_SECOND: u32 = 5;

/// Environment variable for rate limiting (requests per second)
const RATE_LIMIT_ENV_VAR: &str = "RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND";

/// Shared HTTP client with sensible defaults and rate limiting
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Arc<Client>,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

/// Rate-limited request builder - compatible API with reqwest::RequestBuilder
pub struct RateLimitedRequestBuilder {
    inner: reqwest::RequestBuilder,
    rate_limiter: Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
}

impl RateLimitedRequestBuilder {
    /// Send the request (with rate limiting applied first)
    pub async fn send(self) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }
        self.inner.send().await
    }

    /// Add a header (accepts &str for convenience - most common use case)
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.inner = self.inner.header(key.as_ref(), value.as_ref());
        self
    }

    /// Set headers
    pub fn headers(mut self, headers: header::HeaderMap) -> Self {
        self.inner = self.inner.headers(headers);
        self
    }

    /// Basic auth
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: Into<String> + std::fmt::Display,
        P: Into<String> + std::fmt::Display,
    {
        Self {
            inner: self.inner.basic_auth(username, password),
            rate_limiter: self.rate_limiter,
        }
    }

    /// Bearer auth
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: Into<String> + std::fmt::Display,
    {
        Self {
            inner: self.inner.bearer_auth(token),
            rate_limiter: self.rate_limiter,
        }
    }

    /// Query parameters
    pub fn query<T: serde::Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.inner = self.inner.query(query);
        self
    }

    /// Form data
    pub fn form<T: serde::Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.inner = self.inner.form(form);
        self
    }

    /// JSON body
    pub fn json<T: serde::Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.inner = self.inner.json(json);
        self
    }

    /// Build the request
    pub fn build(self) -> Result<reqwest::Request, reqwest::Error> {
        self.inner.build()
    }
}

impl HttpClient {
    /// Create a new HTTP client with default settings and rate limiting
    pub fn new() -> Result<Self, SourceError> {
        Self::with_user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
    }

    /// Create a new HTTP client with a custom user agent
    pub fn with_user_agent(user_agent: &str) -> Result<Self, SourceError> {
        let rate_limiter = Self::create_rate_limiter();

        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| SourceError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            rate_limiter,
        })
    }

    /// Create a new HTTP client without rate limiting
    pub fn without_rate_limit(user_agent: &str) -> Result<Self, SourceError> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| SourceError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            rate_limiter: None,
        })
    }

    /// Create a new HTTP client with a custom rate limit
    pub fn with_rate_limit(user_agent: &str, requests_per_second: u32) -> Result<Self, SourceError> {
        let rate_limiter = if requests_per_second == 0 {
            None
        } else {
            let nonzero = NonZeroU32::new(requests_per_second)
                .expect("requests_per_second should be > 0 when not 0 branch");
            let quota = Quota::per_second(nonzero);
            Some(Arc::new(RateLimiter::direct(quota)))
        };

        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .map_err(|e| SourceError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
            rate_limiter,
        })
    }

    /// Create rate limiter from environment variable or default
    fn create_rate_limiter() -> Option<Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> {
        let requests_per_second = std::env::var(RATE_LIMIT_ENV_VAR)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_REQUESTS_PER_SECOND);

        if requests_per_second == 0 {
            // Rate limiting disabled
            tracing::info!("Rate limiting disabled");
            return None;
        }

        let nonzero = NonZeroU32::new(requests_per_second).expect("requests_per_second should not be zero");
        let quota = Quota::per_second(nonzero);
        let limiter = RateLimiter::direct(quota);

        tracing::info!("Rate limiting enabled: {} requests per second", requests_per_second);

        Some(Arc::new(limiter))
    }

    /// Create from an existing reqwest Client
    pub fn from_client(client: Arc<Client>) -> Self {
        Self {
            client,
            rate_limiter: Self::create_rate_limiter(),
        }
    }

    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Create a rate-limited GET request builder
    pub fn get(&self, url: &str) -> RateLimitedRequestBuilder {
        RateLimitedRequestBuilder {
            inner: self.client.get(url),
            rate_limiter: self.rate_limiter.clone(),
        }
    }

    /// Create a rate-limited POST request builder
    pub fn post(&self, url: &str) -> RateLimitedRequestBuilder {
        RateLimitedRequestBuilder {
            inner: self.client.post(url),
            rate_limiter: self.rate_limiter.clone(),
        }
    }

    /// Download a file from a URL to the specified path
    pub async fn download_to_file(
        &self,
        url: &str,
        request: &DownloadRequest,
        filename: &str,
    ) -> Result<DownloadResult, SourceError> {
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!(
                "Failed to download: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read response: {}", e)))?;

        // Create download directory if it doesn't exist
        std::fs::create_dir_all(&request.save_path).map_err(|e| {
            SourceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory: {}", e),
            ))
        })?;

        let path = Path::new(&request.save_path).join(filename);

        std::fs::write(&path, bytes.as_ref())
            .map_err(|e| SourceError::Io(e.into()))?;

        Ok(DownloadResult::success(
            path.to_string_lossy().to_string(),
            bytes.len() as u64,
        ))
    }

    /// Download a PDF with a sanitized filename
    pub async fn download_pdf(
        &self,
        url: &str,
        request: &DownloadRequest,
        paper_id: &str,
    ) -> Result<DownloadResult, SourceError> {
        let filename = format!("{}.pdf", paper_id.replace('/', "_"));
        self.download_to_file(url, request, &filename).await
    }

    /// Check if a URL returns success status
    pub async fn head(&self, url: &str) -> Result<bool, SourceError> {
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Head request failed: {}", e)))?;
        Ok(response.status() == StatusCode::OK)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rate_limiter_with_default() {
        // Clear the environment variable to test default
        std::env::remove_var(RATE_LIMIT_ENV_VAR);

        let limiter = HttpClient::create_rate_limiter();
        assert!(limiter.is_some(), "Default rate limiter should be created");
    }

    #[test]
    fn test_create_rate_limiter_disabled() {
        std::env::set_var(RATE_LIMIT_ENV_VAR, "0");

        let limiter = HttpClient::create_rate_limiter();
        assert!(limiter.is_none(), "Rate limiter should be disabled when set to 0");

        std::env::remove_var(RATE_LIMIT_ENV_VAR);
    }

    #[test]
    fn test_create_rate_limiter_custom() {
        std::env::set_var(RATE_LIMIT_ENV_VAR, "10");

        let limiter = HttpClient::create_rate_limiter();
        assert!(limiter.is_some(), "Custom rate limiter should be created");

        std::env::remove_var(RATE_LIMIT_ENV_VAR);
    }

    #[test]
    fn test_create_rate_limiter_invalid() {
        std::env::set_var(RATE_LIMIT_ENV_VAR, "invalid");

        let limiter = HttpClient::create_rate_limiter();
        // Should fall back to default when invalid value is provided
        assert!(limiter.is_some(), "Should fall back to default rate limiter");

        std::env::remove_var(RATE_LIMIT_ENV_VAR);
    }
}
