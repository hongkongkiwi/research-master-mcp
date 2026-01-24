//! Mock source for testing purposes.

use async_trait::async_trait;
use std::sync::Mutex;

use crate::models::{Paper, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};

/// A mock source for testing that returns predefined responses.
#[derive(Debug, Default, Clone)]
pub struct MockSource {
    // Note: Clone won't deep clone the mutex contents, but for testing
    // this is acceptable as tests typically use the original reference.
    search_response: std::sync::Arc<Mutex<Option<SearchResponse>>>,
    search_count: std::sync::Arc<Mutex<usize>>,
}

impl MockSource {
    /// Create a new mock source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the search response to return.
    pub fn set_search_response(&self, response: SearchResponse) {
        let mut guard = self.search_response.lock().unwrap();
        *guard = Some(response);
        // Reset search count when setting a new response
        *self.search_count.lock().unwrap() = 0;
    }

    /// Clear the configured response.
    pub fn clear_response(&self) {
        let mut guard = self.search_response.lock().unwrap();
        *guard = None;
        // Reset search count when clearing
        *self.search_count.lock().unwrap() = 0;
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
        // Increment search count
        let mut count = self.search_count.lock().unwrap();
        *count += 1;
        let is_first_call = *count == 1;
        drop(count);

        let guard = self.search_response.lock().unwrap();
        match &*guard {
            Some(response) => {
                if is_first_call {
                    Ok(response.clone())
                } else {
                    // Return empty results for subsequent calls (simulating pagination end)
                    Ok(SearchResponse::new(Vec::new(), "Mock Source", &query.query))
                }
            }
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
