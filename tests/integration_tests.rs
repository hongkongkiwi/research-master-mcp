//! Integration tests for Research Master MCP
//!
//! These tests verify the full functionality of the MCP server and research sources.

use research_master_mcp::mcp::server::McpServer;
use research_master_mcp::models::{SearchQuery, SourceType};
use research_master_mcp::sources::{SourceCapabilities, SourceRegistry};
use std::sync::Arc;

/// Test that the server can be created successfully
#[tokio::test]
async fn test_server_initialization() {
    let registry = SourceRegistry::new();
    let server = McpServer::new(Arc::new(registry));
    assert!(server.is_ok());
}

/// Test that all sources are registered
#[tokio::test]
async fn test_all_sources_registered() {
    let registry = SourceRegistry::new();
    let sources: Vec<_> = registry.all().collect();

    // Should have 28 sources with all features
    assert_eq!(sources.len(), 28);

    // Check each source exists
    let source_ids: Vec<&str> = sources.iter().map(|s| s.id()).collect();
    assert!(source_ids.contains(&"arxiv"));
    assert!(source_ids.contains(&"pubmed"));
    assert!(source_ids.contains(&"biorxiv"));
    assert!(source_ids.contains(&"semantic"));
    assert!(source_ids.contains(&"openalex"));
    assert!(source_ids.contains(&"crossref"));
    assert!(source_ids.contains(&"iacr"));
    assert!(source_ids.contains(&"pmc"));
    assert!(source_ids.contains(&"hal"));
    assert!(source_ids.contains(&"dblp"));
    assert!(source_ids.contains(&"ssrn"));
    assert!(source_ids.contains(&"dimensions"));
    assert!(source_ids.contains(&"ieee_xplore"));
    assert!(source_ids.contains(&"europe_pmc"));
    assert!(source_ids.contains(&"core"));
    assert!(source_ids.contains(&"zenodo"));
    assert!(source_ids.contains(&"unpaywall"));
    assert!(source_ids.contains(&"mdpi"));
    assert!(source_ids.contains(&"jstor"));
    assert!(source_ids.contains(&"scispace"));
    assert!(source_ids.contains(&"acm"));
    assert!(source_ids.contains(&"connected_papers"));
    assert!(source_ids.contains(&"doaj"));
    assert!(source_ids.contains(&"worldwidescience"));
    assert!(source_ids.contains(&"osf"));
    assert!(source_ids.contains(&"base"));
    assert!(source_ids.contains(&"springer"));
    assert!(source_ids.contains(&"google_scholar"));
}

/// Test source capabilities are properly reported
#[tokio::test]
async fn test_source_capabilities() {
    let registry = SourceRegistry::new();

    // arXiv should support search, download, and read
    let arxiv = registry.get("arxiv");
    assert!(arxiv.is_some());
    let caps = arxiv.unwrap().capabilities();
    assert!(caps.contains(SourceCapabilities::SEARCH));
    assert!(caps.contains(SourceCapabilities::DOWNLOAD));
    assert!(caps.contains(SourceCapabilities::READ));

    // CrossRef should support search and DOI lookup
    let crossref = registry.get("crossref");
    assert!(crossref.is_some());
    let caps = crossref.unwrap().capabilities();
    assert!(caps.contains(SourceCapabilities::SEARCH));
    assert!(caps.contains(SourceCapabilities::DOI_LOOKUP));
}

/// Test SearchQuery builder
#[test]
fn test_search_query_builder() {
    let query = SearchQuery::new("machine learning")
        .max_results(20)
        .year("2020-")
        .author("Hinton");

    assert_eq!(query.query, "machine learning");
    assert_eq!(query.max_results, 20);
    assert_eq!(query.year, Some("2020-".to_string()));
    assert_eq!(query.author, Some("Hinton".to_string()));
}

/// Test SourceType display and serialization
#[test]
fn test_source_type() {
    assert_eq!(SourceType::Arxiv.to_string(), "arXiv");
    assert_eq!(SourceType::PubMed.to_string(), "PubMed");
    assert_eq!(SourceType::SemanticScholar.to_string(), "Semantic Scholar");
}

/// Test error handling for invalid queries
#[test]
fn test_invalid_query_handling() {
    // Empty query should still be valid (some sources support listing)
    let query = SearchQuery::new("").max_results(10);
    assert_eq!(query.query, "");
    assert_eq!(query.max_results, 10);

    // Very large max_results should be accepted
    let query = SearchQuery::new("test").max_results(10000);
    assert_eq!(query.max_results, 10000);
}

/// Test source retrieval by name
#[tokio::test]
async fn test_get_source_by_name() {
    let registry = SourceRegistry::new();

    // Test getting existing sources
    assert!(registry.get("arxiv").is_some());
    assert!(registry.get("pubmed").is_some());
    assert!(registry.get("semantic").is_some());

    // Test getting non-existent source
    assert!(registry.get("nonexistent").is_none());
}

/// Test getting sources by capability
#[tokio::test]
async fn test_get_sources_by_capability() {
    let registry = SourceRegistry::new();

    // Get all searchable sources
    let searchable = registry.with_capability(SourceCapabilities::SEARCH);

    assert!(!searchable.is_empty());
    assert!(searchable.len() >= 8); // At least 8 sources should support search

    // Get all DOI lookup sources
    let doi_lookup = registry.with_capability(SourceCapabilities::DOI_LOOKUP);

    assert!(!doi_lookup.is_empty());
}

/// Test helper methods on registry
#[tokio::test]
async fn test_registry_helper_methods() {
    let registry = SourceRegistry::new();

    // Test has() method
    assert!(registry.has("arxiv"));
    assert!(!registry.has("nonexistent"));

    // Test len() method - should be 28 with all features
    assert_eq!(registry.len(), 28);

    // Test is_empty() method
    assert!(!registry.is_empty());

    // Test searchable() helper
    let searchable = registry.searchable();
    assert!(!searchable.is_empty());

    // Test downloadable() helper
    let downloadable = registry.downloadable();
    assert!(!downloadable.is_empty());
}

/// Test source metadata
#[tokio::test]
async fn test_source_metadata() {
    let registry = SourceRegistry::new();
    let arxiv = registry.get("arxiv").unwrap();

    assert_eq!(arxiv.id(), "arxiv");
    assert_eq!(arxiv.name(), "arXiv");
}

/// Test Paper model
#[test]
fn test_paper_model() {
    use research_master_mcp::models::PaperBuilder;

    let paper = PaperBuilder::new(
        "1234.5678",
        "Test Paper",
        "https://example.com",
        SourceType::Arxiv,
    )
    .authors("John Doe; Jane Smith")
    .abstract_text("This is a test abstract.")
    .doi("10.1234/test")
    .published_date("2024")
    .citations(42)
    .build();

    assert_eq!(paper.paper_id, "1234.5678");
    assert_eq!(paper.title, "Test Paper");
    assert_eq!(paper.authors, "John Doe; Jane Smith");
    assert_eq!(paper.r#abstract, "This is a test abstract.");
    assert_eq!(paper.doi, Some("10.1234/test".to_string()));
    assert_eq!(paper.citations, Some(42));

    // Test helper methods
    assert_eq!(paper.primary_id(), "10.1234/test");
    assert_eq!(paper.author_list(), vec!["John Doe", "Jane Smith"]);
    assert!(!paper.has_pdf()); // No PDF URL set
}

/// Test Paper with PDF
#[test]
fn test_paper_with_pdf() {
    use research_master_mcp::models::PaperBuilder;

    let paper = PaperBuilder::new("1234", "Test", "https://example.com", SourceType::Arxiv)
        .pdf_url("https://example.com/paper.pdf")
        .build();

    assert!(paper.has_pdf());
    assert_eq!(
        paper.pdf_url,
        Some("https://example.com/paper.pdf".to_string())
    );
}

/// Test Paper categories and keywords
#[test]
fn test_paper_categories_keywords() {
    use research_master_mcp::models::PaperBuilder;

    let paper = PaperBuilder::new("1234", "Test", "https://example.com", SourceType::Arxiv)
        .categories("Machine Learning; AI")
        .keywords("deep learning; neural networks")
        .build();

    assert_eq!(paper.category_list(), vec!["Machine Learning", "AI"]);
    assert_eq!(
        paper.keyword_list(),
        vec!["deep learning", "neural networks"]
    );
}

/// Test registry ids() iterator
#[tokio::test]
async fn test_registry_ids() {
    let registry = SourceRegistry::new();
    let ids: Vec<&str> = registry.ids().collect();

    assert_eq!(ids.len(), 28);
    assert!(ids.contains(&"arxiv"));
    assert!(ids.contains(&"pubmed"));
    assert!(ids.contains(&"semantic"));
}
