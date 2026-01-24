//! Search request and response models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sort order for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Sort field for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    Relevance,
    Date,
    CitationCount,
    Title,
    Author,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Main search query string
    pub query: String,

    /// Maximum number of results to return
    pub max_results: usize,

    /// Year filter (single year, range like "2018-2022", or "2010-" for from, "-2015" for until)
    pub year: Option<String>,

    /// Sort by field
    pub sort_by: Option<SortBy>,

    /// Sort order
    pub sort_order: Option<SortOrder>,

    /// Field-specific filters
    pub filters: HashMap<String, String>,

    /// Author name for author-specific search
    pub author: Option<String>,

    /// Category/subject filter
    pub category: Option<String>,

    /// Whether to fetch detailed information (slower but more complete)
    pub fetch_details: bool,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_results: 10,
            year: None,
            sort_by: None,
            sort_order: None,
            filters: HashMap::new(),
            author: None,
            category: None,
            fetch_details: true,
        }
    }
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set maximum results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set year filter
    pub fn year(mut self, year: impl Into<String>) -> Self {
        self.year = Some(year.into());
        self
    }

    /// Set sort by
    pub fn sort_by(mut self, sort: SortBy) -> Self {
        self.sort_by = Some(sort);
        self
    }

    /// Set sort order
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Add a filter
    pub fn filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.filters.insert(key.into(), value.into());
        self
    }

    /// Set author filter
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set category filter
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Enable/disable detailed fetching
    pub fn fetch_details(mut self, fetch: bool) -> Self {
        self.fetch_details = fetch;
        self
    }
}

/// Request for downloading a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    /// Paper ID (source-specific)
    pub paper_id: String,

    /// Where to save the PDF
    pub save_path: String,

    /// Optional DOI
    pub doi: Option<String>,
}

impl DownloadRequest {
    /// Create a new download request
    pub fn new(paper_id: impl Into<String>, save_path: impl Into<String>) -> Self {
        Self {
            paper_id: paper_id.into(),
            save_path: save_path.into(),
            doi: None,
        }
    }

    /// Set the DOI
    pub fn doi(mut self, doi: impl Into<String>) -> Self {
        self.doi = Some(doi.into());
        self
    }
}

/// Request for reading/parsing a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadRequest {
    /// Paper ID (source-specific)
    pub paper_id: String,

    /// Path where the PDF is saved (or will be saved)
    pub save_path: String,

    /// Whether to download if not found
    pub download_if_missing: bool,
}

impl ReadRequest {
    /// Create a new read request
    pub fn new(paper_id: impl Into<String>, save_path: impl Into<String>) -> Self {
        Self {
            paper_id: paper_id.into(),
            save_path: save_path.into(),
            download_if_missing: true,
        }
    }

    /// Set whether to download if missing
    pub fn download_if_missing(mut self, download: bool) -> Self {
        self.download_if_missing = download;
        self
    }
}

/// Request for getting citations/references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRequest {
    /// Paper ID (source-specific)
    pub paper_id: String,

    /// Maximum results
    pub max_results: usize,
}

impl CitationRequest {
    /// Create a new citation request
    pub fn new(paper_id: impl Into<String>) -> Self {
        Self {
            paper_id: paper_id.into(),
            max_results: 20,
        }
    }

    /// Set max results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }
}

/// Search response containing papers and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Papers found
    pub papers: Vec<crate::models::Paper>,

    /// Total number of results (may be more than returned)
    pub total_results: Option<usize>,

    /// Source of the results
    pub source: String,

    /// Query that was executed
    pub query: String,

    /// Whether more results are available
    pub has_more: bool,
}

impl SearchResponse {
    /// Create a new search response
    pub fn new(
        papers: Vec<crate::models::Paper>,
        source: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            papers,
            total_results: None,
            source: source.into(),
            query: query.into(),
            has_more: false,
        }
    }

    /// Set total results
    pub fn total_results(mut self, total: usize) -> Self {
        self.total_results = Some(total);
        self
    }

    /// Set has_more flag
    pub fn has_more(mut self, has_more: bool) -> Self {
        self.has_more = has_more;
        self
    }
}

/// Result of a download operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    /// Path where the file was saved
    pub path: String,

    /// Number of bytes downloaded
    pub bytes: u64,

    /// Whether the download was successful
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl DownloadResult {
    /// Create a successful download result
    pub fn success(path: impl Into<String>, bytes: u64) -> Self {
        Self {
            path: path.into(),
            bytes,
            success: true,
            error: None,
        }
    }

    /// Create a failed download result
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            path: String::new(),
            bytes: 0,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Batch download request containing multiple individual download requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDownloadRequest {
    /// List of individual download requests
    pub requests: Vec<DownloadRequest>,
}

impl BatchDownloadRequest {
    /// Create a new batch download request from a list of requests
    pub fn new(requests: Vec<DownloadRequest>) -> Self {
        Self { requests }
    }

    /// Add a download request to the batch
    pub fn add_request(&mut self, request: DownloadRequest) {
        self.requests.push(request);
    }

    /// Get the number of requests in the batch
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

/// Result of a batch download operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDownloadResult {
    /// Individual download results
    pub results: Vec<DownloadResult>,

    /// Total number of successful downloads
    pub successful: usize,

    /// Total number of failed downloads
    pub failed: usize,

    /// Total bytes downloaded
    pub total_bytes: u64,
}

impl BatchDownloadResult {
    /// Create a new batch download result from individual results
    pub fn new(results: Vec<DownloadResult>) -> Self {
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        let total_bytes = results.iter().map(|r| r.bytes).sum();

        Self {
            results,
            successful,
            failed,
            total_bytes,
        }
    }

    /// Get the success rate as a percentage (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            0.0
        } else {
            self.successful as f64 / self.results.len() as f64
        }
    }

    /// Check if all downloads succeeded (and there was at least one)
    pub fn is_all_success(&self) -> bool {
        !self.results.is_empty() && self.failed == 0
    }
}

/// Result of a paper read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    /// Extracted text content
    pub text: String,

    /// Number of pages
    pub pages: Option<usize>,

    /// Whether the read was successful
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl ReadResult {
    /// Create a successful read result
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            pages: None,
            success: true,
            error: None,
        }
    }

    /// Set page count
    pub fn pages(mut self, pages: usize) -> Self {
        self.pages = Some(pages);
        self
    }

    /// Create a failed read result
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            pages: None,
            success: false,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_download_request_new() {
        let requests = vec![
            DownloadRequest::new("paper1", "/downloads"),
            DownloadRequest::new("paper2", "/downloads"),
        ];
        let batch = BatchDownloadRequest::new(requests);
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_download_request_add() {
        let mut batch = BatchDownloadRequest::new(vec![]);
        assert!(batch.is_empty());

        batch.add_request(DownloadRequest::new("paper1", "/downloads"));
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_download_result_new() {
        let results = vec![
            DownloadResult::success("/path/to/paper1.pdf", 1024),
            DownloadResult::success("/path/to/paper2.pdf", 2048),
            DownloadResult::error("Failed to download"),
        ];

        let batch = BatchDownloadResult::new(results);

        assert_eq!(batch.successful, 2);
        assert_eq!(batch.failed, 1);
        assert_eq!(batch.total_bytes, 3072);
        assert!((batch.success_rate() - 0.666).abs() < 0.001);
        assert!(!batch.is_all_success());
    }

    #[test]
    fn test_batch_download_result_all_success() {
        let results = vec![
            DownloadResult::success("/path/to/paper1.pdf", 1024),
            DownloadResult::success("/path/to/paper2.pdf", 2048),
        ];

        let batch = BatchDownloadResult::new(results);

        assert_eq!(batch.successful, 2);
        assert_eq!(batch.failed, 0);
        assert_eq!(batch.success_rate(), 1.0);
        assert!(batch.is_all_success());
    }

    #[test]
    fn test_batch_download_result_empty() {
        let batch = BatchDownloadResult::new(vec![]);

        assert_eq!(batch.successful, 0);
        assert_eq!(batch.failed, 0);
        assert_eq!(batch.total_bytes, 0);
        assert_eq!(batch.success_rate(), 0.0);
        assert!(!batch.is_all_success());
    }
}
