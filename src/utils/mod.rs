//! Utility modules.

mod dedup;
mod http;
mod pdf;
mod retry;

pub use dedup::{deduplicate_papers, find_duplicates, DuplicateStrategy};
pub use http::{HttpClient, RateLimitedRequestBuilder};
pub use pdf::{extract_text, is_available, PdfExtractError};
pub use retry::{
    api_retry_config, strict_rate_limit_retry_config, with_retry, with_retry_detailed,
    RetryConfig, RetryResult, TransientError,
};
