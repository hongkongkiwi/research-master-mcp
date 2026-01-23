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
