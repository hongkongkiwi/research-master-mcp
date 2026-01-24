//! Mock source for testing purposes.

use async_trait::async_trait;
use std::sync::Mutex;

use crate::models::{Paper, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};

/// A mock source for testing that returns predefined responses.
#[derive(Debug, Default)]
pub struct MockSource {
    search_response: Mutex<Option<SearchResponse>>,
}

impl MockSource {
    /// Create a new mock source.
    pub fn new() -> Self {
        Self {
            search_response: Mutex::new(None),
        }
    }

    /// Set the search response to return.
    pub fn set_search_response(&self, response: SearchResponse) {
        let mut guard = self.search_response.lock().unwrap();
        *guard = Some(response);
    }

    /// Clear the configured response.
    pub fn clear_response(&self) {
        let mut guard = self.search_response.lock().unwrap();
        *guard = None;
    }
}

#[async_trait]
impl Source for MockSource {
    fn id(&self) -> &str {
        "mock"
    }

    fn name(&self) -> &str {
        "Mock Source"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let guard = self.search_response.lock().unwrap();
        match &*guard {
            Some(response) => Ok(response.clone()),
            None => Ok(SearchResponse::new(Vec::new(), "Mock Source", &query.query)),
        }
    }
}

/// Helper function to create a mock paper for testing.
pub fn make_paper(paper_id: &str, title: &str, source_type: SourceType) -> Paper {
    Paper::new(
        paper_id.to_string(),
        title.to_string(),
        format!("http://example.com/{}", paper_id),
        source_type,
    )
}
