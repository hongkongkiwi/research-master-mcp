//! Core data models for research papers and search operations.

mod paper;
mod search;

pub use paper::{Paper, PaperBuilder, SourceType};
pub use search::{
    CitationRequest, DownloadRequest, DownloadResult, ReadRequest, ReadResult, SearchQuery,
    SearchResponse, SortBy, SortOrder,
};
