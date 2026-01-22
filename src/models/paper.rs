//! Paper model representing a research paper from any source.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The source/repository where the paper was found
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Arxiv,
    PubMed,
    BioRxiv,
    MedRxiv,
    SemanticScholar,
    OpenAlex,
    CrossRef,
    IACR,
    PMC,
    HAL,
    DBLP,
    SSRN,
    GoogleScholar,
    SciHub,
    CORE,
    EuropePMC,
    #[serde(untagged)]
    Other(String),
}

impl SourceType {
    /// Returns the display name of the source
    pub fn name(&self) -> &str {
        match self {
            SourceType::Arxiv => "arXiv",
            SourceType::PubMed => "PubMed",
            SourceType::BioRxiv => "bioRxiv",
            SourceType::MedRxiv => "medRxiv",
            SourceType::SemanticScholar => "Semantic Scholar",
            SourceType::OpenAlex => "OpenAlex",
            SourceType::CrossRef => "CrossRef",
            SourceType::IACR => "IACR ePrint",
            SourceType::PMC => "PubMed Central",
            SourceType::HAL => "HAL",
            SourceType::DBLP => "DBLP",
            SourceType::SSRN => "SSRN",
            SourceType::GoogleScholar => "Google Scholar",
            SourceType::SciHub => "Sci-Hub",
            SourceType::CORE => "CORE",
            SourceType::EuropePMC => "Europe PMC",
            SourceType::Other(s) => s,
        }
    }

    /// Returns the source identifier (for tool naming)
    pub fn id(&self) -> &str {
        match self {
            SourceType::Arxiv => "arxiv",
            SourceType::PubMed => "pubmed",
            SourceType::BioRxiv => "biorxiv",
            SourceType::MedRxiv => "medrxiv",
            SourceType::SemanticScholar => "semantic",
            SourceType::OpenAlex => "openalex",
            SourceType::CrossRef => "crossref",
            SourceType::IACR => "iacr",
            SourceType::PMC => "pmc",
            SourceType::HAL => "hal",
            SourceType::DBLP => "dblp",
            SourceType::SSRN => "ssrn",
            SourceType::GoogleScholar => "google_scholar",
            SourceType::SciHub => "sci_hub",
            SourceType::CORE => "core",
            SourceType::EuropePMC => "europe_pmc",
            SourceType::Other(s) => s,
        }
    }
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A research paper from any academic source
///
/// This struct provides a standardized format for papers across all sources,
/// making it easy to work with papers from multiple repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    /// Unique identifier (source-specific: DOI, PMID, arXiv ID, etc.)
    pub paper_id: String,

    /// Paper title
    pub title: String,

    /// Authors (semicolon-separated)
    pub authors: String,

    /// Abstract text
    pub r#abstract: String,

    /// Digital Object Identifier
    pub doi: Option<String>,

    /// Publication date (ISO format)
    pub published_date: Option<String>,

    /// Last updated date (ISO format)
    pub updated_date: Option<String>,

    /// Direct PDF URL
    pub pdf_url: Option<String>,

    /// Paper page URL
    pub url: String,

    /// Source where the paper was found
    pub source: SourceType,

    /// Categories/tags (semicolon-separated)
    pub categories: Option<String>,

    /// Keywords (semicolon-separated)
    pub keywords: Option<String>,

    /// Citation count
    pub citations: Option<u32>,

    /// Reference IDs (semicolon-separated)
    pub references: Option<String>,

    /// Source-specific metadata (flexible JSON)
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl Paper {
    /// Create a new paper with required fields
    pub fn new(paper_id: String, title: String, url: String, source: SourceType) -> Self {
        Self {
            paper_id,
            title,
            authors: String::new(),
            r#abstract: String::new(),
            doi: None,
            published_date: None,
            updated_date: None,
            pdf_url: None,
            url,
            source,
            categories: None,
            keywords: None,
            citations: None,
            references: None,
            extra: None,
        }
    }

    /// Returns the primary identifier for this paper (DOI if available, else paper_id)
    pub fn primary_id(&self) -> &str {
        self.doi.as_ref().unwrap_or(&self.paper_id)
    }

    /// Returns the author names as a vector
    pub fn author_list(&self) -> Vec<&str> {
        self.authors
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Returns the categories as a vector
    pub fn category_list(&self) -> Vec<&str> {
        self.categories
            .as_ref()
            .map(|c| c.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }

    /// Returns the keywords as a vector
    pub fn keyword_list(&self) -> Vec<&str> {
        self.keywords
            .as_ref()
            .map(|k| k.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }

    /// Check if paper has a downloadable PDF
    pub fn has_pdf(&self) -> bool {
        self.pdf_url.is_some()
    }
}

/// Builder for constructing Paper objects
#[derive(Debug, Clone)]
pub struct PaperBuilder {
    paper: Paper,
}

impl PaperBuilder {
    /// Create a new builder with required fields
    pub fn new(paper_id: impl Into<String>, title: impl Into<String>, url: impl Into<String>, source: SourceType) -> Self {
        Self {
            paper: Paper::new(paper_id.into(), title.into(), url.into(), source),
        }
    }

    /// Set authors
    pub fn authors(mut self, authors: impl Into<String>) -> Self {
        self.paper.authors = authors.into();
        self
    }

    /// Set abstract
    pub fn abstract_text(mut self, abstract_text: impl Into<String>) -> Self {
        self.paper.r#abstract = abstract_text.into();
        self
    }

    /// Set DOI
    pub fn doi(mut self, doi: impl Into<String>) -> Self {
        self.paper.doi = Some(doi.into());
        self
    }

    /// Set publication date
    pub fn published_date(mut self, date: impl Into<String>) -> Self {
        self.paper.published_date = Some(date.into());
        self
    }

    /// Set updated date
    pub fn updated_date(mut self, date: impl Into<String>) -> Self {
        self.paper.updated_date = Some(date.into());
        self
    }

    /// Set PDF URL
    pub fn pdf_url(mut self, url: impl Into<String>) -> Self {
        self.paper.pdf_url = Some(url.into());
        self
    }

    /// Set categories
    pub fn categories(mut self, categories: impl Into<String>) -> Self {
        self.paper.categories = Some(categories.into());
        self
    }

    /// Set keywords
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.paper.keywords = Some(keywords.into());
        self
    }

    /// Set citation count
    pub fn citations(mut self, count: u32) -> Self {
        self.paper.citations = Some(count);
        self
    }

    /// Set references
    pub fn references(mut self, references: impl Into<String>) -> Self {
        self.paper.references = Some(references.into());
        self
    }

    /// Add extra metadata
    pub fn extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.paper.extra
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }

    /// Build the Paper
    pub fn build(self) -> Paper {
        self.paper
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_builder() {
        let paper = PaperBuilder::new("1234.5678", "Test Paper", "https://example.com", SourceType::Arxiv)
            .authors("John Doe; Jane Smith")
            .abstract_text("This is a test abstract.")
            .doi("10.1234/test.1234")
            .pdf_url("https://example.com/paper.pdf")
            .citations(42)
            .build();

        assert_eq!(paper.paper_id, "1234.5678");
        assert_eq!(paper.title, "Test Paper");
        assert_eq!(paper.authors, "John Doe; Jane Smith");
        assert_eq!(paper.doi, Some("10.1234/test.1234".to_string()));
        assert_eq!(paper.citations, Some(42));
    }

    #[test]
    fn test_author_list() {
        let paper = PaperBuilder::new(
            "1234".to_string(),
            "Test".to_string(),
            "https://example.com".to_string(),
            SourceType::Arxiv,
        )
        .authors("John Doe; Jane Smith; Bob Jones")
        .build();

        let authors = paper.author_list();
        assert_eq!(authors, vec!["John Doe", "Jane Smith", "Bob Jones"]);
    }

    #[test]
    fn test_primary_id() {
        let with_doi = PaperBuilder::new(
            "1234".to_string(),
            "Test".to_string(),
            "https://example.com".to_string(),
            SourceType::Arxiv,
        )
        .doi("10.1234/test")
        .build();

        assert_eq!(with_doi.primary_id(), "10.1234/test");

        let without_doi = Paper::new(
            "1234".to_string(),
            "Test".to_string(),
            "https://example.com".to_string(),
            SourceType::Arxiv,
        );

        assert_eq!(without_doi.primary_id(), "1234");
    }
}
