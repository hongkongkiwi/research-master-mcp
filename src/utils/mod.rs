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
//! ```
//! use research_master_mcp::utils::DuplicateStrategy;
//!
//! // Example: deduplicate_papers takes papers and a strategy
//! let strategy = DuplicateStrategy::First;
//! assert_eq!(strategy, DuplicateStrategy::First);
//! ```
//!
//! # HTTP Client with Rate Limiting
//!
//! ```ignore
//! use research_master_mcp::utils::HttpClient;
//!
//! let client = HttpClient::new();
//! // Use the client to make rate-limited requests
//! ```
//!
//! # Retry with Backoff
//!
//! ```ignore
//! use research_master_mcp::utils::{with_retry, RetryConfig, TransientError};
//!
//! let config = RetryConfig::default().max_retries(3);
//! // Use with_retry to execute operations with automatic retry
//! ```

mod cache;
mod dedup;
mod http;
mod pdf;
mod retry;

pub use cache::{CacheResult, CacheService, CacheStats};
pub use dedup::{deduplicate_papers, find_duplicates, DuplicateStrategy};
pub use http::{HttpClient, RateLimitedRequestBuilder};
pub use pdf::{extract_text, is_available, PdfExtractError};
pub use retry::{
    api_retry_config, strict_rate_limit_retry_config, with_retry, with_retry_detailed, RetryConfig,
    RetryResult, TransientError,
};
