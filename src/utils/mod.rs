//! Utility modules.

mod dedup;
mod http;

pub use dedup::{deduplicate_papers, find_duplicates, DuplicateStrategy};
pub use http::HttpClient;
