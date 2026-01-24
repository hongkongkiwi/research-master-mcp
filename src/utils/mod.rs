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
mod circuit_breaker;
mod dedup;
mod http;
mod pdf;
mod progress;
mod retry;
mod streaming;
mod update;
mod validate;

pub use streaming::{
    collect_papers, filter_by_year, paper_stream, ConcurrentPaperStream, FilterByYearStream,
    SkipStream, TakeStream,
};

pub use cache::{CacheResult, CacheService, CacheStats};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerManager, CircuitResult, CircuitState};
pub use dedup::{deduplicate_papers, fast_deduplicate_papers, find_duplicates, DuplicateStrategy};
pub use http::{HttpClient, RateLimitedRequestBuilder};
pub use pdf::{
    extract_text, extract_text_simple, get_extraction_info, has_poppler, has_tesseract,
    ExtractionInfo, ExtractionMethod, PdfExtractError,
};
pub use progress::{ProgressReporter, SharedProgress};
pub use retry::{
    api_retry_config, strict_rate_limit_retry_config, with_retry, with_retry_detailed, RetryConfig,
    RetryResult, TransientError,
};
pub use update::{
    cleanup_temp_files, compute_sha256, detect_installation, download_and_extract_asset,
    fetch_and_verify_sha256, fetch_latest_release, fetch_sha256_signature, find_asset_for_platform,
    get_current_target, get_update_instructions, replace_binary, verify_gpg_signature,
    verify_sha256, InstallationMethod, ReleaseAsset, ReleaseInfo,
};
pub use validate::{
    sanitize_filename, sanitize_paper_id, validate_doi, validate_url, ValidationError,
};
