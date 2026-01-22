//! Semantic Scholar research source implementation.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{
    CitationRequest, DownloadRequest, DownloadResult, ReadRequest, ReadResult, Source,
    SourceCapabilities, SourceError,
};
use crate::utils::{api_retry_config, with_retry, HttpClient, RateLimitedRequestBuilder};

const SEMANTIC_API_BASE: &str = "https://api.semanticscholar.org/graph/v1";

/// Environment variable for Semantic Scholar rate limit (requests per second)
const SEMANTIC_SCHOLAR_RATE_LIMIT_ENV: &str = "SEMANTIC_SCHOLAR_RATE_LIMIT";

/// Default rate limit for Semantic Scholar (1 request per second without API key)
const DEFAULT_SEMANTIC_RATE_LIMIT: u32 = 1;

/// Semantic Scholar research source
///
/// Uses Semantic Scholar GraphQL/REST API.
#[derive(Debug, Clone)]
pub struct SemanticScholarSource {
    client: Arc<HttpClient>,
    api_key: Option<String>,
}

impl SemanticScholarSource {
    /// Get the rate limit from environment variable or use default
    fn get_rate_limit() -> u32 {
        std::env::var(SEMANTIC_SCHOLAR_RATE_LIMIT_ENV)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(DEFAULT_SEMANTIC_RATE_LIMIT)
    }

    /// Create a new Semantic Scholar source
    pub fn new() -> Result<Self, SourceError> {
        let rate_limit = Self::get_rate_limit();
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        Ok(Self {
            client: Arc::new(HttpClient::with_rate_limit(user_agent, rate_limit)?),
            api_key: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
        })
    }

    /// Create with an API key (optional, for higher rate limits)
    #[allow(dead_code)]
    pub fn with_api_key(api_key: String) -> Result<Self, SourceError> {
        let rate_limit = Self::get_rate_limit();
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

        Ok(Self {
            client: Arc::new(HttpClient::with_rate_limit(user_agent, rate_limit)?),
            api_key: Some(api_key),
        })
    }

    /// Build request URL with optional API key
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", SEMANTIC_API_BASE, endpoint)
    }

    /// Add API key to request headers if available
    fn add_api_key_if_present(&self, builder: RateLimitedRequestBuilder) -> RateLimitedRequestBuilder {
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
        Self::new().expect("Failed to create SemanticScholarSource")
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

        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let api_key = self.api_key.clone();
        let url_for_retry = url.clone();

        let data: S2SearchResponse = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let api_key = api_key.clone();
            let url = url_for_retry.clone();
            async move {
                let mut request = client.get(&format!("{}{}", SEMANTIC_API_BASE, url));
                if let Some(ref key) = api_key {
                    request = request.header("x-api-key", key);
                }
                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search Semantic Scholar: {}", e)))?;

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "Semantic Scholar API returned status: {}",
                        response.status()
                    )));
                }

                response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))
            }
        })
        .await?;

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
        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let api_key = self.api_key.clone();

        // First, search for the author
        let author_url = format!("/author/search?query={}&limit=1", urlencoding::encode(author));

        let author_data: AuthorSearchResponse = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let api_key = api_key.clone();
            let url = author_url.clone();
            async move {
                let mut request = client.get(&format!("{}{}", SEMANTIC_API_BASE, url));
                if let Some(ref key) = api_key {
                    request = request.header("x-api-key", key);
                }
                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search author: {}", e)))?;

                response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))
            }
        })
        .await?;

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

        let papers_data: PapersResponse = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let api_key = api_key.clone();
            let url = papers_url.clone();
            async move {
                let mut request = client.get(&format!("{}{}", SEMANTIC_API_BASE, url));
                if let Some(ref key) = api_key {
                    request = request.header("x-api-key", key);
                }
                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to fetch author papers: {}", e)))?;

                response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))
            }
        })
        .await?;

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
        let download_result = self.download(&download_request).await?;

        let pdf_path = std::path::Path::new(&download_result.path);
        match crate::utils::extract_text(pdf_path) {
            Ok(text) => {
                let pages = (text.len() / 3000).max(1);
                Ok(ReadResult::success(text).pages(pages))
            }
            Err(e) => {
                Ok(ReadResult::error(format!("PDF downloaded but text extraction failed: {}", e)))
            }
        }
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
