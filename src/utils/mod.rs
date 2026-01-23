//! Utility modules supporting research operations.
//!
//! This module provides utility functions and types used throughout the library:
//!
//! - [`deduplicate_papers`]: Remove duplicate papers from results using DOI matching and title similarity
//! - [`find_duplicates`]: Find duplicates without modifying the original list
//! - [`DuplicateStrategy`]: Strategy for handling duplicates (KeepFirst, KeepLast, Mark)
//! - [`HttpClient`]: HTTP client with built-in rate limiting
//! - [`RateLimitedRequestBuilder`]: Builder for rate-limited HTTP requests
//! - [`extract_text`]: Extract text content from PDF files
//! - [`is_available`]: Check if PDF extraction is available (requires poppler)
//! - [`PdfExtractError`]: Errors that can occur during PDF extraction
//! - [`RetryConfig`]: Configuration for retry logic with exponential backoff
//! - [`with_retry`]: Execute an operation with automatic retry on transient errors
//!
//! # Deduplication
//!
//! ```rust
//! use research_master_mcp::utils::{deduplicate_papers, DuplicateStrategy};
//! use research_master_mcp::models::Paper;
//!
//! # fn example(papers: Vec<Paper>) {
//! // Remove duplicates, keeping the first occurrence
//! let unique = deduplicate_papers(papers.clone(), DuplicateStrategy::KeepFirst);
//!
//! // Mark duplicates instead of removing them
//! let marked = deduplicate_papers(papers, DuplicateStrategy::Mark);
//! # }
//! ```
//!
//! # HTTP Client with Rate Limiting
//!
//! ```rust,no_run
//! use research_master_mcp::utils::HttpClient;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = HttpClient::new();
//! let response = client.get("https://api.example.com")
//!     .rate_limit_per_second(5)
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Retry with Backoff
//!
//! ```rust,no_run
//! use research_master_mcp::utils::{with_retry, RetryConfig, TransientError};
//!
//! # async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> { Ok("data".to_string()) }
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RetryConfig::default().max_retries(3);
//! let result = with_retry(config, || async {
//!     fetch_data().await.map_err(|e| TransientError::new(e))
//! }).await?;
//! # Ok(())
//! # }
//! ```

mod dedup;
mod http;
mod pdf;
mod retry;

pub use dedup::{deduplicate_papers, find_duplicates, DuplicateStrategy};
pub use http::{HttpClient, RateLimitedRequestBuilder};
pub use pdf::{extract_text, is_available, PdfExtractError};
pub use retry::{
    api_retry_config, strict_rate_limit_retry_config, with_retry, with_retry_detailed, RetryConfig,
    RetryResult, TransientError,
};
