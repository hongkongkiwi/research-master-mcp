//! Unified tool handlers with smart source selection.

use std::sync::Arc;

use serde_json::Value;

use super::tools::ToolHandler;

/// Helper function to auto-detect the appropriate source for a paper ID
fn auto_detect_source(
    sources: &Arc<Vec<Arc<dyn crate::sources::Source>>>,
    paper_id: &str,
) -> Result<Arc<dyn crate::sources::Source>, String> {
    let paper_id_lower = paper_id.to_lowercase();

    // arXiv: arXiv:1234.5678 or numeric format like 1234.5678
    if paper_id_lower.starts_with("arxiv:")
        || (paper_id.len() > 4 && paper_id.chars().take(9).all(|c| c.is_numeric() || c == '.'))
    {
        return sources
            .iter()
            .find(|s| s.id() == "arxiv")
            .cloned()
            .ok_or_else(|| "arXiv source not available".to_string());
    }

    // PMC: PMC followed by digits
    if paper_id_upper_start(paper_id, "PMC") {
        return sources
            .iter()
            .find(|s| s.id() == "pmc")
            .cloned()
            .ok_or_else(|| "PMC source not available".to_string());
    }

    // HAL: hal- followed by digits
    if paper_id_lower.starts_with("hal-") {
        return sources
            .iter()
            .find(|s| s.id() == "hal")
            .cloned()
            .ok_or_else(|| "HAL source not available".to_string());
    }

    // IACR: format like 2023/1234
    if paper_id.chars().filter(|&c| c == '/').count() == 1 {
        return sources
            .iter()
            .find(|s| s.id() == "iacr")
            .cloned()
            .ok_or_else(|| "IACR source not available".to_string());
    }

    // DOI format (10.xxxx/xxxxxx) - prefer Semantic Scholar for DOI lookup
    if paper_id.starts_with("10.") {
        // First, explicitly check for Semantic Scholar (preferred)
        if let Some(source) = sources
            .iter()
            .find(|s| s.id() == "semantic" && s.supports_doi_lookup())
        {
            return Ok(Arc::clone(source));
        }
        // Fallback to any DOI-capable source
        if let Some(source) = sources.iter().find(|s| s.supports_doi_lookup()) {
            return Ok(Arc::clone(source));
        }
    }

    // Default: try arXiv first, then semantic
    if let Some(source) = sources.iter().find(|s| s.id() == "arxiv") {
        return Ok(Arc::clone(source));
    }

    if let Some(source) = sources.iter().find(|s| s.id() == "semantic") {
        return Ok(Arc::clone(source));
    }

    Err("Could not auto-detect source. Please specify source explicitly.".to_string())
}

/// Helper function to check if a string starts with a specific prefix (case-insensitive)
fn paper_id_upper_start(paper_id: &str, prefix: &str) -> bool {
    if paper_id.len() < prefix.len() {
        return false;
    }

    paper_id[..prefix.len()].to_uppercase() == prefix
}

/// Handler for searching papers across all or specific sources
#[derive(Debug)]
pub struct SearchPapersHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for SearchPapersHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'query' parameter")?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let year = args
            .get("year")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let source_filter = args.get("source").and_then(|v| v.as_str());

        let mut all_results = Vec::new();

        for source in self.sources.iter() {
            // Filter by source if specified
            if let Some(filter) = source_filter {
                if source.id() != filter {
                    continue;
                }
            }

            // Skip sources that don't support search
            if !source.supports_search() {
                continue;
            }

            let mut search_query = crate::models::SearchQuery::new(query).max_results(max_results);

            if let Some(ref year) = year {
                search_query = search_query.year(year);
            }
            if let Some(ref cat) = category {
                search_query = search_query.category(cat);
            }

            match source.search(&search_query).await {
                Ok(response) => {
                    all_results.extend(response.papers);
                }
                Err(e) => {
                    tracing::warn!("Search failed for {}: {}", source.id(), e);
                }
            }
        }

        serde_json::to_value(all_results).map_err(|e| e.to_string())
    }
}

/// Handler for searching papers by author
#[derive(Debug)]
pub struct SearchByAuthorHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for SearchByAuthorHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let author = args
            .get("author")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'author' parameter")?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let year = args.get("year").and_then(|v| v.as_str());

        let source_filter = args.get("source").and_then(|v| v.as_str());

        let mut all_results = Vec::new();

        for source in self.sources.iter() {
            // Filter by source if specified
            if let Some(filter) = source_filter {
                if source.id() != filter {
                    continue;
                }
            }

            // Skip sources that don't support author search
            if !source.supports_author_search() {
                continue;
            }

            match source.search_by_author(author, max_results, year).await {
                Ok(response) => {
                    all_results.extend(response.papers);
                }
                Err(e) => {
                    tracing::warn!("Author search failed for {}: {}", source.id(), e);
                }
            }
        }

        serde_json::to_value(all_results).map_err(|e| e.to_string())
    }
}

/// Handler for getting paper metadata with auto-detection
#[derive(Debug)]
pub struct GetPaperHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for GetPaperHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let source_override = args.get("source").and_then(|v| v.as_str());

        // Find the appropriate source
        let source = self.find_source(paper_id, source_override)?;

        // For now, we'll do a search with the paper ID as the query
        let search_query = crate::models::SearchQuery::new(paper_id).max_results(1);

        let response = source
            .search(&search_query)
            .await
            .map_err(|e| e.to_string())?;

        if response.papers.is_empty() {
            return Err(format!("Paper '{}' not found in {}", paper_id, source.id()));
        }

        serde_json::to_value(&response.papers[0]).map_err(|e| e.to_string())
    }
}

/// Handler for downloading papers with auto-detection
#[derive(Debug)]
pub struct DownloadPaperHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for DownloadPaperHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let source_override = args.get("source").and_then(|v| v.as_str());

        let output_path = args
            .get("output_path")
            .and_then(|v| v.as_str())
            .unwrap_or("./downloads");

        // Find the appropriate source
        let source = self.find_source(paper_id, source_override)?;

        let request = crate::models::DownloadRequest::new(paper_id, output_path);

        let result = source.download(&request).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

/// Handler for reading papers (PDF text extraction) with auto-detection
#[derive(Debug)]
pub struct ReadPaperHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for ReadPaperHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let source_override = args.get("source").and_then(|v| v.as_str());

        // Find the appropriate source
        let source = self.find_source(paper_id, source_override)?;

        let request = crate::models::ReadRequest::new(paper_id, "./downloads");

        let result = source.read(&request).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }
}

/// Handler for getting citations
#[derive(Debug)]
pub struct GetCitationsHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for GetCitationsHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let source_override = args.get("source").and_then(|v| v.as_str());

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        // Default to Semantic Scholar if not specified
        let source_id = source_override.unwrap_or("semantic");

        let source = self
            .sources
            .iter()
            .find(|s| s.id() == source_id)
            .ok_or_else(|| format!("Source '{}' not found", source_id))?;

        if !source.supports_citations() {
            return Err(format!("Source '{}' does not support citations", source_id));
        }

        let request = crate::models::CitationRequest::new(paper_id).max_results(max_results);

        let response = source
            .get_citations(&request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| e.to_string())
    }
}

/// Handler for getting references
#[derive(Debug)]
pub struct GetReferencesHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for GetReferencesHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let paper_id = args
            .get("paper_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'paper_id' parameter")?;

        let source_override = args.get("source").and_then(|v| v.as_str());

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;

        // Default to Semantic Scholar if not specified
        let source_id = source_override.unwrap_or("semantic");

        let source = self
            .sources
            .iter()
            .find(|s| s.id() == source_id)
            .ok_or_else(|| format!("Source '{}' not found", source_id))?;

        if !source.supports_citations() {
            return Err(format!(
                "Source '{}' does not support references",
                source_id
            ));
        }

        let request = crate::models::CitationRequest::new(paper_id).max_results(max_results);

        let response = source
            .get_references(&request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| e.to_string())
    }
}

/// Handler for DOI lookup
#[derive(Debug)]
pub struct LookupByDoiHandler {
    pub sources: Arc<Vec<Arc<dyn crate::sources::Source>>>,
}

#[async_trait::async_trait]
impl ToolHandler for LookupByDoiHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let doi = args
            .get("doi")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'doi' parameter")?;

        let source_filter = args.get("source").and_then(|v| v.as_str());

        // Try each source that supports DOI lookup
        for source in self.sources.iter() {
            // Filter by source if specified
            if let Some(filter) = source_filter {
                if source.id() != filter {
                    continue;
                }
            }

            // Skip sources that don't support DOI lookup
            if !source.supports_doi_lookup() {
                continue;
            }

            match source.get_by_doi(doi).await {
                Ok(paper) => {
                    return serde_json::to_value(paper).map_err(|e| e.to_string());
                }
                Err(e) => {
                    tracing::debug!("DOI lookup failed for {}: {}", source.id(), e);
                }
            }
        }

        Err(format!("Paper with DOI '{}' not found", doi))
    }
}

/// Handler for deduplicating papers
#[derive(Debug)]
pub struct DeduplicatePapersHandler;

#[async_trait::async_trait]
impl ToolHandler for DeduplicatePapersHandler {
    async fn execute(&self, args: Value) -> Result<Value, String> {
        let papers: Vec<crate::models::Paper> = serde_json::from_value(
            args.get("papers")
                .ok_or("Missing 'papers' parameter")?
                .clone(),
        )
        .map_err(|e| format!("Invalid papers array: {}", e))?;

        let strategy_str = args
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("first");

        let strategy = match strategy_str {
            "last" => crate::utils::DuplicateStrategy::Last,
            "mark" => crate::utils::DuplicateStrategy::Mark,
            _ => crate::utils::DuplicateStrategy::First,
        };

        let deduped = crate::utils::deduplicate_papers(papers, strategy);

        serde_json::to_value(deduped).map_err(|e| e.to_string())
    }
}

// Helper trait for source auto-detection
impl GetPaperHandler {
    fn find_source(
        &self,
        paper_id: &str,
        source_override: Option<&str>,
    ) -> Result<Arc<dyn crate::sources::Source>, String> {
        // If source is explicitly specified, use it
        if let Some(source_id) = source_override {
            return self
                .sources
                .iter()
                .find(|s| s.id() == source_id)
                .cloned()
                .ok_or_else(|| format!("Source '{}' not found", source_id));
        }

        // Use shared auto-detection logic
        auto_detect_source(&self.sources, paper_id)
    }
}

impl DownloadPaperHandler {
    fn find_source(
        &self,
        paper_id: &str,
        source_override: Option<&str>,
    ) -> Result<Arc<dyn crate::sources::Source>, String> {
        // If source is explicitly specified, use it
        if let Some(source_id) = source_override {
            return self
                .sources
                .iter()
                .find(|s| s.id() == source_id)
                .cloned()
                .ok_or_else(|| format!("Source '{}' not found", source_id));
        }

        // Use shared auto-detection logic
        auto_detect_source(&self.sources, paper_id)
    }
}

impl ReadPaperHandler {
    fn find_source(
        &self,
        paper_id: &str,
        source_override: Option<&str>,
    ) -> Result<Arc<dyn crate::sources::Source>, String> {
        // If source is explicitly specified, use it
        if let Some(source_id) = source_override {
            return self
                .sources
                .iter()
                .find(|s| s.id() == source_id)
                .cloned()
                .ok_or_else(|| format!("Source '{}' not found", source_id));
        }

        // Use shared auto-detection logic
        auto_detect_source(&self.sources, paper_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_id_upper_start_basic() {
        assert!(paper_id_upper_start("PMC12345", "PMC"));
        assert!(paper_id_upper_start("pmc12345", "PMC"));
        assert!(paper_id_upper_start("Pmc12345", "PMC"));
        assert!(!paper_id_upper_start("ABC12345", "PMC"));
        assert!(!paper_id_upper_start("PM", "PMC")); // Too short
    }

    #[test]
    fn test_paper_id_upper_start_edge_cases() {
        assert!(!paper_id_upper_start("", "PMC"));
        assert!(!paper_id_upper_start("PM", "PMC"));
    }
}
