//! Async streaming utilities for large result sets.
//!
//! This module provides streaming iterators for processing large
//! search results incrementally without loading everything into memory.

use crate::models::{Paper, SearchQuery};
use crate::sources::Source;
use futures_util::stream::Stream;
use futures_util::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::warn;

/// A stream that yields papers one at a time from paginated search results.
///
/// This allows processing large result sets without loading everything
/// into memory at once. The stream automatically handles pagination
/// and rate limiting.
pub struct PaperStream<T: Source> {
    source: T,
    query: SearchQuery,
    page_size: usize,
    offset: usize,
    total_fetched: usize,
    total_available: Option<usize>,
    buffer: Vec<Paper>,
    buffer_index: usize,
    rate_limit_delay: Duration,
    done: bool,
}

impl<T: Source> PaperStream<T> {
    /// Create a new paper stream
    ///
    /// - `source`: The source to search
    /// - `query`: The search query
    /// - `page_size`: Number of results to fetch per page (default: 100)
    pub fn new(source: T, query: SearchQuery, page_size: usize) -> Self {
        Self {
            source,
            query,
            page_size,
            offset: 0,
            total_fetched: 0,
            total_available: None,
            buffer: Vec::new(),
            buffer_index: 0,
            rate_limit_delay: Duration::from_millis(200), // Default delay between pages
            done: false,
        }
    }

    /// Set the delay between page fetches (for rate limiting)
    pub fn with_rate_limit_delay(mut self, delay: Duration) -> Self {
        self.rate_limit_delay = delay;
        self
    }

    /// Get the total number of papers available (if known)
    pub fn total_available(&self) -> Option<usize> {
        self.total_available
    }

    /// Get the number of papers already fetched
    pub fn fetched_count(&self) -> usize {
        self.total_fetched
    }

    /// Check if the stream is complete
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Fetch the next page of results
    async fn fetch_next_page(&mut self) -> Result<(), crate::sources::SourceError> {
        // Clone the query and modify it for pagination
        let mut query = self.query.clone();
        query.max_results = self.page_size;

        let response = self.source.search(&query).await?;

        self.buffer = response.papers;
        self.buffer_index = 0;
        self.total_available = response.total_results.or(response.total_results);

        if self.buffer.is_empty() {
            self.done = true;
        } else {
            self.total_fetched += self.buffer.len();
            self.offset += self.buffer.len();

            // Apply rate limiting delay
            if self.rate_limit_delay > Duration::ZERO {
                sleep(self.rate_limit_delay).await;
            }
        }

        Ok(())
    }
}

impl<T: Source + Unpin> Stream for PaperStream<T> {
    type Item = Paper;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        // Refill buffer if needed
        if self.buffer.is_empty() || self.buffer_index >= self.buffer.len() {
            // Can't poll async in sync context, so we return Pending
            // The actual fetching happens in the .next() async method
            return Poll::Pending;
        }

        // Clone the paper to avoid mutable borrow conflict
        let paper = self.buffer[self.buffer_index].clone();
        self.buffer_index += 1;
        Poll::Ready(Some(paper))
    }
}

impl<T: Source> PaperStream<T> {
    /// Get the next paper from the stream (async)
    ///
    /// Returns `None` when the stream is exhausted.
    pub async fn next(&mut self) -> Option<Paper> {
        // Refill buffer if needed
        if self.buffer.is_empty() || self.buffer_index >= self.buffer.len() {
            if self.done {
                return None;
            }

            if let Err(e) = self.fetch_next_page().await {
                warn!("Error fetching next page: {}", e);
                self.done = true;
                return None;
            }
        }

        // Return next paper from buffer
        if self.buffer_index < self.buffer.len() {
            let paper = self.buffer.remove(self.buffer_index);
            Some(paper)
        } else {
            None
        }
    }

    /// Collect all remaining papers into a Vec
    ///
    /// Note: This defeats the purpose of streaming for large result sets.
    /// Use `next()` iteratively instead for large datasets.
    pub async fn collect_all(mut self) -> Vec<Paper> {
        let mut papers = Vec::new();

        while let Some(paper) = self.next().await {
            papers.push(paper);
        }

        papers
    }
}

/// A channel-based concurrent stream for parallel source searches.
///
/// This allows searching multiple sources concurrently and
/// yielding results as they arrive.
#[allow(dead_code)]
pub struct ConcurrentPaperStream {
    receiver: mpsc::Receiver<Paper>,
    pending: usize,
}

impl ConcurrentPaperStream {
    /// Create a new concurrent stream from a list of sources
    ///
    /// Searches all sources concurrently and yields papers in the
    /// order they complete.
    pub async fn from_sources<S: Source + Clone + 'static>(
        sources: Vec<S>,
        query: &SearchQuery,
        max_concurrent: usize,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let sources_len = sources.len();

        for source in sources {
            let query = query.clone();
            let sender = sender.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let source = source.clone();

            tokio::spawn(async move {
                // permit is automatically dropped when this async block ends
                match source.search(&query).await {
                    Ok(response) => {
                        for paper in response.papers {
                            if sender.send(paper).await.is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Source search failed: {}", e);
                    }
                }
                drop(permit);
            });
        }

        // Drop sender to signal completion when all tasks finish
        drop(sender);

        Self {
            receiver,
            pending: sources_len,
        }
    }

    /// Get the next paper from any source
    pub async fn next(&mut self) -> Option<Paper> {
        self.receiver.recv().await
    }

    /// Check if more results are coming
    pub fn is_done(&self) -> bool {
        self.receiver.is_closed()
    }
}

/// Stream filter for year range
pub struct FilterByYearStream<S: Stream<Item = Paper>> {
    stream: S,
    min_year: Option<i32>,
    max_year: Option<i32>,
}

impl<S: Stream<Item = Paper>> FilterByYearStream<S> {
    /// Create a new year filter stream
    pub fn new(stream: S, min_year: Option<i32>, max_year: Option<i32>) -> Self {
        Self {
            stream,
            min_year,
            max_year,
        }
    }
}

impl<S: Stream<Item = Paper> + Unpin> Stream for FilterByYearStream<S> {
    type Item = Paper;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(paper)) => {
                    // Try to extract year from published_date (format: "YYYY-MM-DD" or "YYYY")
                    if let Some(year) = extract_year(&paper.published_date) {
                        if let Some(min) = this.min_year {
                            if year < min {
                                continue;
                            }
                        }
                        if let Some(max) = this.max_year {
                            if year > max {
                                continue;
                            }
                        }
                    }
                    return Poll::Ready(Some(paper));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Extract year from published_date string
fn extract_year(published_date: &Option<String>) -> Option<i32> {
    published_date.as_ref().and_then(|date| {
        // Try to parse "YYYY-MM-DD" or just "YYYY"
        date.split('-')
            .next()
            .or(date.split('/').next())
            .and_then(|y| y.parse::<i32>().ok())
    })
}

/// Stream that limits the number of items
pub struct TakeStream<S: Stream<Item = Paper>> {
    stream: S,
    remaining: usize,
}

impl<S: Stream<Item = Paper>> TakeStream<S> {
    /// Create a new take stream
    pub fn new(stream: S, limit: usize) -> Self {
        Self {
            stream,
            remaining: limit,
        }
    }
}

impl<S: Stream<Item = Paper> + Unpin> Stream for TakeStream<S> {
    type Item = Paper;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                self.remaining -= 1;
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream that skips items
pub struct SkipStream<S: Stream<Item = Paper>> {
    stream: S,
    to_skip: usize,
}

impl<S: Stream<Item = Paper>> SkipStream<S> {
    /// Create a new skip stream
    pub fn new(stream: S, n: usize) -> Self {
        Self { stream, to_skip: n }
    }
}

impl<S: Stream<Item = Paper> + Unpin> Stream for SkipStream<S> {
    type Item = Paper;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.stream).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    if self.to_skip > 0 {
                        self.to_skip -= 1;
                        continue;
                    }
                    return Poll::Ready(Some(item));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<S: Stream<Item = Paper> + Unpin> SkipStream<S> {
    /// Get the next paper from the stream (async)
    pub async fn next(&mut self) -> Option<Paper> {
        loop {
            match self.stream.next().await {
                Some(item) => {
                    if self.to_skip > 0 {
                        self.to_skip -= 1;
                        continue;
                    }
                    return Some(item);
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Paper, SearchResponse, SourceType};
    use crate::sources::mock::MockSource;
    use futures_util::stream::StreamExt;

    fn make_paper(paper_id: &str, title: &str, source_type: SourceType) -> Paper {
        Paper::new(
            paper_id.to_string(),
            title.to_string(),
            format!("http://example.com/{}", paper_id),
            source_type,
        )
    }

    #[tokio::test]
    async fn test_paper_stream_basic() {
        let mock = MockSource::new();
        mock.set_search_response(SearchResponse::new(
            vec![
                make_paper("1", "Paper 1", SourceType::Arxiv),
                make_paper("2", "Paper 2", SourceType::Arxiv),
                make_paper("3", "Paper 3", SourceType::Arxiv),
            ],
            "Mock Source",
            "test",
        ));

        let stream = PaperStream::new(mock, SearchQuery::new("test"), 10);
        let papers: Vec<_> = stream.collect_all().await;

        assert_eq!(papers.len(), 3);
        assert_eq!(papers[0].paper_id, "1");
        assert_eq!(papers[1].paper_id, "2");
        assert_eq!(papers[2].paper_id, "3");
    }

    #[tokio::test]
    async fn test_paper_stream_empty() {
        let mock = MockSource::new();
        mock.set_search_response(SearchResponse::new(Vec::new(), "Mock Source", "test"));

        let stream = PaperStream::new(mock, SearchQuery::new("test"), 10);
        let papers: Vec<_> = stream.collect_all().await;

        assert!(papers.is_empty());
    }

    #[tokio::test]
    async fn test_take_stream() {
        let mock = MockSource::new();
        mock.set_search_response(SearchResponse::new(
            vec![
                make_paper("1", "Paper 1", SourceType::Arxiv),
                make_paper("2", "Paper 2", SourceType::Arxiv),
                make_paper("3", "Paper 3", SourceType::Arxiv),
            ],
            "Mock Source",
            "test",
        ));

        let stream = PaperStream::new(mock, SearchQuery::new("test"), 10);
        let taken = TakeStream::new(stream, 2);
        let papers: Vec<_> = taken.collect::<Vec<_>>().await;

        assert_eq!(papers.len(), 2);
    }

    #[tokio::test]
    async fn test_skip_stream() {
        let mock = MockSource::new();
        mock.set_search_response(SearchResponse::new(
            vec![
                make_paper("1", "Paper 1", SourceType::Arxiv),
                make_paper("2", "Paper 2", SourceType::Arxiv),
                make_paper("3", "Paper 3", SourceType::Arxiv),
            ],
            "Mock Source",
            "test",
        ));

        let stream = PaperStream::new(mock, SearchQuery::new("test"), 10);
        let skipped = SkipStream::new(stream, 1);
        let papers: Vec<_> = skipped.collect::<Vec<_>>().await;

        assert_eq!(papers.len(), 2);
        assert_eq!(papers[0].paper_id, "2");
        assert_eq!(papers[1].paper_id, "3");
    }

    #[test]
    fn test_extract_year() {
        assert_eq!(extract_year(&Some("2023-05-15".to_string())), Some(2023));
        assert_eq!(extract_year(&Some("2023".to_string())), Some(2023));
        assert_eq!(extract_year(&Some("2023/05/15".to_string())), Some(2023));
        assert_eq!(extract_year(&None), None);
        assert_eq!(extract_year(&Some("invalid".to_string())), None);
    }
}
