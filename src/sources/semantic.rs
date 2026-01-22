//! Semantic Scholar research source implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{
    CitationRequest, DownloadRequest, DownloadResult, ReadRequest, ReadResult, Source,
    SourceCapabilities, SourceError,
};

const SEMANTIC_API_BASE: &str = "https://api.semanticscholar.org/graph/v1";

/// Semantic Scholar research source
///
/// Uses Semantic Scholar GraphQL/REST API.
#[derive(Debug, Clone)]
pub struct SemanticScholarSource {
    client: Arc<Client>,
    api_key: Option<String>,
}

impl SemanticScholarSource {
    /// Create a new Semantic Scholar source
    pub fn new() -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
            api_key: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
        }
    }

    /// Create with an API key (optional, for higher rate limits)
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
            api_key: Some(api_key),
        }
    }

    /// Build request URL with optional API key
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", SEMANTIC_API_BASE, endpoint)
    }

    /// Add API key to request headers if available
    fn add_api_key_if_present(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.api_key {
            builder.header("x-api-key", key)
        } else {
            builder
        }
    }

    /// Parse Semantic Scholar paper data
    fn parse_paper(data: &S2Paper) -> Paper {
        let authors = data
            .authors
            .iter()
            .filter_map(|a| a.name.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        let published_date = data.year.as_ref().map(|y| y.to_string());

        let doi = data.doi.clone().unwrap_or_default();

        let url = data.url.clone().unwrap_or_else(|| {
            if !doi.is_empty() {
                format!("https://doi.org/{}", doi)
            } else {
                String::new()
            }
        });

        let paper_id = data
            .paper_id
            .clone()
            .unwrap_or_else(|| format!("CorpusId:{}", data.corpus_id));

        let pdf_url = data.open_access_pdf.as_ref().and_then(|p| p.url.clone()).unwrap_or_default();

        PaperBuilder::new(paper_id, data.title.clone(), url, SourceType::SemanticScholar)
            .authors(authors)
            .abstract_text(data.r#abstract.clone().unwrap_or_default())
            .doi(doi)
            .published_date(published_date.unwrap_or_default())
            .pdf_url(pdf_url)
            .citations(data.citation_count.unwrap_or(0) as u32)
            .build()
    }
}

impl Default for SemanticScholarSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for SemanticScholarSource {
    fn id(&self) -> &str {
        "semantic"
    }

    fn name(&self) -> &str {
        "Semantic Scholar"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH
            | SourceCapabilities::DOWNLOAD
            | SourceCapabilities::READ
            | SourceCapabilities::CITATIONS
            | SourceCapabilities::DOI_LOOKUP
            | SourceCapabilities::AUTHOR_SEARCH
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/paper/search?query={}&limit={}",
            urlencoding::encode(&query.query),
            query.max_results
        );

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search Semantic Scholar: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::Api(format!(
                "Semantic Scholar API returned status: {}",
                response.status()
            )));
        }

        let data: S2SearchResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .data
            .into_iter()
            .map(|item| Ok(Self::parse_paper(&item)))
            .collect();

        Ok(SearchResponse::new(papers?, "Semantic Scholar", &query.query))
    }

    async fn search_by_author(
        &self,
        author: &str,
        max_results: usize,
    ) -> Result<SearchResponse, SourceError> {
        // First, search for the author
        let url = format!(
            "/author/search?query={}&limit=1",
            urlencoding::encode(author)
        );

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search author: {}", e)))?;

        let author_data: AuthorSearchResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let author_id = author_data
            .data
            .first()
            .and_then(|a| a.author_id.clone())
            .ok_or_else(|| SourceError::NotFound("Author not found".to_string()))?;

        // Then get papers by that author
        let papers_url = format!(
            "/author/{}/papers?limit={}",
            author_id, max_results
        );

        let papers_response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&papers_url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch author papers: {}", e)))?;

        let papers_data: PapersResponse = papers_response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = papers_data
            .data
            .into_iter()
            .map(|item| Ok(Self::parse_paper(&item)))
            .collect();

        Ok(SearchResponse::new(papers?, "Semantic Scholar", author))
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        // Try to get the paper first to find PDF URL
        let url = format!("/paper/{}", urlencoding::encode(&request.paper_id));

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch paper: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!(
                "Paper not found: {}",
                request.paper_id
            )));
        }

        let data: PaperResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let pdf_url = data
            .data
            .open_access_pdf
            .and_then(|p| p.url.clone())
            .ok_or_else(|| SourceError::NotFound("No PDF available".to_string()))?;

        let pdf_response = self
            .client
            .get(&pdf_url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download PDF: {}", e)))?;

        if !pdf_response.status().is_success() {
            return Err(SourceError::NotFound("PDF not available".to_string()));
        }

        let bytes = pdf_response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read PDF: {}", e)))?;

        std::fs::create_dir_all(&request.save_path).map_err(|e| {
            SourceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory: {}", e),
            ))
        })?;

        let filename = format!("{}.pdf", request.paper_id.replace('/', "_"));
        let path = std::path::Path::new(&request.save_path).join(&filename);

        std::fs::write(&path, bytes.as_ref())
            .map_err(|e| SourceError::Io(e.into()))?;

        Ok(DownloadResult::success(path.to_string_lossy().to_string(), bytes.len() as u64))
    }

    async fn read(&self, request: &ReadRequest) -> Result<ReadResult, SourceError> {
        let download_request = DownloadRequest::new(&request.paper_id, &request.save_path);
        self.download(&download_request).await?;

        Ok(ReadResult::success(
            "PDF text extraction not yet fully implemented".to_string(),
        ))
    }

    async fn get_citations(
        &self,
        request: &CitationRequest,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/paper/{}/citations?limit={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch citations: {}", e)))?;

        let data: CitationsResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .data
            .into_iter()
            .map(|item| Ok(Self::parse_paper(&item)))
            .collect();

        Ok(SearchResponse::new(papers?, "Semantic Scholar", &request.paper_id))
    }

    async fn get_references(
        &self,
        request: &CitationRequest,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/paper/{}/references?limit={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch references: {}", e)))?;

        let data: ReferencesResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .data
            .into_iter()
            .map(|item| Ok(Self::parse_paper(&item)))
            .collect();

        Ok(SearchResponse::new(papers?, "Semantic Scholar", &request.paper_id))
    }

    async fn get_related(
        &self,
        request: &CitationRequest,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/paper/{}/related?limit={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch related: {}", e)))?;

        let data: RelatedResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .data
            .into_iter()
            .map(|item| Ok(Self::parse_paper(&item)))
            .collect();

        Ok(SearchResponse::new(papers?, "Semantic Scholar", &request.paper_id))
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        // Search by DOI using the search API
        let url = format!("/paper/search?query={}&limit=1", urlencoding::encode(doi));

        let response = self
            .add_api_key_if_present(self.client.get(&self.build_url(&url)))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search DOI: {}", e)))?;

        let data: S2SearchResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        data.data
            .first()
            .map(|p| Self::parse_paper(p))
            .ok_or_else(|| SourceError::NotFound("DOI not found".to_string()))
    }
}

// ===== Semantic Scholar API Types =====

#[derive(Debug, Deserialize)]
struct S2Paper {
    #[serde(rename = "paperId")]
    paper_id: Option<String>,
    #[serde(rename = "corpusId")]
    corpus_id: String,
    title: String,
    r#abstract: Option<String>,
    year: Option<i32>,
    #[serde(rename = "citationCount")]
    citation_count: Option<i32>,
    authors: Vec<S2Author>,
    doi: Option<String>,
    url: Option<String>,
    #[serde(rename = "openAccessPdf")]
    open_access_pdf: Option<S2OpenAccessPdf>,
}

#[derive(Debug, Deserialize)]
struct S2OpenAccessPdf {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2Author {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2AuthorData {
    #[serde(rename = "authorId")]
    author_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2SearchResponse {
    data: Vec<S2Paper>,
}

#[derive(Debug, Deserialize)]
struct AuthorSearchResponse {
    data: Vec<S2AuthorData>,
}

#[derive(Debug, Deserialize)]
struct PapersResponse {
    data: Vec<S2Paper>,
}

#[derive(Debug, Deserialize)]
struct PaperResponse {
    data: S2Paper,
}

#[derive(Debug, Deserialize)]
struct CitationsResponse {
    data: Vec<S2Paper>,
}

#[derive(Debug, Deserialize)]
struct ReferencesResponse {
    data: Vec<S2Paper>,
}

#[derive(Debug, Deserialize)]
struct RelatedResponse {
    data: Vec<S2Paper>,
}
