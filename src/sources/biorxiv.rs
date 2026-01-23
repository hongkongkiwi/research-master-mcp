//! bioRxiv/medRxiv research source implementation.
//!
//! This module provides a shared implementation for both bioRxiv and medRxiv
//! since they use the same API with just a different server prefix.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{
    DownloadRequest, DownloadResult, ReadRequest, ReadResult, Source, SourceCapabilities,
    SourceError,
};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const BIORXIV_API_URL: &str = "https://api.biorxiv.org";
const MEDRXIV_API_URL: &str = "https://api.medrxiv.org";

/// Server type for biorxiv/medrxiv
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerType {
    BioRxiv,
    MedRxiv,
}

impl ServerType {
    fn name(&self) -> &str {
        match self {
            ServerType::BioRxiv => "biorxiv",
            ServerType::MedRxiv => "medrxiv",
        }
    }

    fn display_name(&self) -> &str {
        match self {
            ServerType::BioRxiv => "bioRxiv",
            ServerType::MedRxiv => "medRxiv",
        }
    }

    fn api_url(&self) -> &str {
        match self {
            ServerType::BioRxiv => BIORXIV_API_URL,
            ServerType::MedRxiv => MEDRXIV_API_URL,
        }
    }

    fn source_type(&self) -> SourceType {
        match self {
            ServerType::BioRxiv => SourceType::BioRxiv,
            ServerType::MedRxiv => SourceType::MedRxiv,
        }
    }

    fn pdf_url(&self, doi: &str) -> String {
        let server = match self {
            ServerType::BioRxiv => "biorxiv",
            ServerType::MedRxiv => "medrxiv",
        };
        format!("https://www.{}/content/{}.full.pdf", server, doi)
    }
}

/// Shared implementation for bioRxiv/medRxiv
#[derive(Debug, Clone)]
struct BiorxivMedrxivSource {
    client: Arc<HttpClient>,
    server_type: ServerType,
}

impl BiorxivMedrxivSource {
    fn new(server_type: ServerType) -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
            server_type,
        })
    }

    /// Get papers from the server (cursor-based pagination)
    async fn get_papers(
        &self,
        cursor: &str,
        max_results: usize,
    ) -> Result<Vec<Paper>, SourceError> {
        let url = format!(
            "{}/details/{}/*/{}/{}",
            self.server_type.api_url(),
            self.server_type.name(),
            max_results,
            cursor
        );

        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();
        let display_name = self.server_type.display_name().to_string();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            let display_name = display_name.clone();
            async move {
                let response = client.get(&url).send().await.map_err(|e| {
                    SourceError::Network(format!("Failed to fetch from {}: {}", display_name, e))
                })?;

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "{} API returned status: {}",
                        display_name,
                        response.status()
                    )));
                }

                Ok(response)
            }
        })
        .await?;

        let json: ApiResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let mut papers = Vec::new();

        for collection in json.collection {
            for paper in collection.papers {
                let authors = paper.authors.unwrap_or_default().clone().join("; ");

                let categories = paper.category.unwrap_or_default().clone();

                let published_date = paper.date.clone();

                let doi = paper.doi.clone().unwrap_or_default();

                let url = paper
                    .server_url
                    .clone()
                    .unwrap_or_else(|| format!("https://doi.org/{}", doi));

                papers.push(
                    PaperBuilder::new(
                        doi.clone(),
                        paper.title,
                        url,
                        self.server_type.source_type(),
                    )
                    .authors(authors)
                    .abstract_text(paper.r#abstract.unwrap_or_default())
                    .doi(doi.clone())
                    .published_date(published_date)
                    .categories(categories)
                    .pdf_url(self.server_type.pdf_url(&doi))
                    .build(),
                );
            }
        }

        Ok(papers)
    }

    /// Parse a DOI to get the paper ID
    fn parse_doi(&self, doi: &str) -> Result<String, SourceError> {
        // bioRxiv DOIs look like: 10.1101/2023.123.456789
        let trimmed = doi.trim();

        if trimmed.is_empty() {
            return Err(SourceError::InvalidRequest("Empty DOI".to_string()));
        }

        Ok(trimmed.to_string())
    }
}

impl Default for BiorxivMedrxivSource {
    fn default() -> Self {
        Self::new(ServerType::BioRxiv).expect("Failed to create BiorxivMedrxivSource")
    }
}

#[async_trait]
impl Source for BiorxivMedrxivSource {
    fn id(&self) -> &str {
        self.server_type.name()
    }

    fn name(&self) -> &str {
        self.server_type.display_name()
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD | SourceCapabilities::READ
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let mut cursor = "0".to_string();
        let mut all_papers = Vec::new();
        let remaining = query.max_results;

        // bioRxiv/medRxiv API is cursor-based, fetch until we have enough
        while all_papers.len() < query.max_results {
            let batch_size = remaining.clamp(10, 100);
            let papers = self.get_papers(&cursor, batch_size).await?;

            if papers.is_empty() {
                break;
            }

            let count = papers.len();
            all_papers.extend(papers);

            // Update cursor (rough estimate - API doesn't return actual cursor)
            cursor = format!("{}", all_papers.len());

            if count < batch_size {
                // Got less than requested, probably no more results
                break;
            }
        }

        // Filter by query if specified (simple text search)
        let filtered = if !query.query.is_empty() {
            let query_lower = query.query.to_lowercase();
            all_papers
                .into_iter()
                .filter(|p| {
                    p.title.to_lowercase().contains(&query_lower)
                        || p.r#abstract.to_lowercase().contains(&query_lower)
                        || p.authors.to_lowercase().contains(&query_lower)
                })
                .collect()
        } else {
            all_papers
        };

        // Truncate to max_results
        let papers = filtered.into_iter().take(query.max_results).collect();

        Ok(SearchResponse::new(
            papers,
            self.server_type.display_name(),
            &query.query,
        ))
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        let doi = self.parse_doi(&request.paper_id)?;

        // First, we need to get the paper to find the PDF URL
        // Search for the paper by DOI
        let papers = self.get_papers("0", 1).await?;
        let paper = papers
            .iter()
            .find(|p| p.doi.as_ref().map(|d| d == &doi).unwrap_or(false))
            .ok_or_else(|| SourceError::NotFound(format!("Paper not found: {}", doi)))?;

        let pdf_url = paper
            .pdf_url
            .as_ref()
            .ok_or_else(|| SourceError::NotFound("No PDF available".to_string()))?;

        let response = self
            .client
            .get(pdf_url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download PDF: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!("Paper not found: {}", doi)));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read PDF: {}", e)))?;

        // Create download directory if it doesn't exist
        std::fs::create_dir_all(&request.save_path).map_err(|e| {
            SourceError::Io(std::io::Error::other(format!(
                "Failed to create directory: {}",
                e
            )))
        })?;

        let filename = format!("{}.pdf", doi.replace('/', "_"));
        let path = std::path::Path::new(&request.save_path).join(&filename);

        std::fs::write(&path, bytes.as_ref()).map_err(SourceError::Io)?;

        Ok(DownloadResult::success(
            path.to_string_lossy().to_string(),
            bytes.len() as u64,
        ))
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
            Err(e) => Ok(ReadResult::error(format!(
                "PDF downloaded but text extraction failed: {}",
                e
            ))),
        }
    }
}

/// API response structure for bioRxiv/medRxiv
#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "collection")]
    collection: Vec<Collection>,
    #[allow(dead_code)]
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct Collection {
    #[serde(rename = "Articles")]
    papers: Vec<Article>,
}

#[derive(Debug, Deserialize)]
struct Article {
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "authors")]
    authors: Option<Vec<String>>,
    #[serde(rename = "abstract")]
    r#abstract: Option<String>,
    #[serde(rename = "date")]
    date: String,
    #[serde(rename = "category")]
    category: Option<String>,
    #[serde(rename = "doi")]
    doi: Option<String>,
    #[serde(rename = "server")]
    server_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Message {
    #[serde(rename = "text")]
    #[allow(dead_code)]
    text: String,
}

/// bioRxiv research source
#[derive(Debug, Clone)]
pub struct BiorxivSource {
    inner: BiorxivMedrxivSource,
}

impl BiorxivSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            inner: BiorxivMedrxivSource::new(ServerType::BioRxiv)?,
        })
    }
}

impl Default for BiorxivSource {
    fn default() -> Self {
        Self::new().expect("Failed to create BiorxivSource")
    }
}

#[async_trait]
impl Source for BiorxivSource {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn capabilities(&self) -> SourceCapabilities {
        self.inner.capabilities()
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        self.inner.search(query).await
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        self.inner.download(request).await
    }

    async fn read(&self, request: &ReadRequest) -> Result<ReadResult, SourceError> {
        self.inner.read(request).await
    }
}

/// medRxiv research source
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MedrxivSource {
    inner: BiorxivMedrxivSource,
}

#[allow(dead_code)]
impl MedrxivSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            inner: BiorxivMedrxivSource::new(ServerType::MedRxiv)?,
        })
    }
}

impl Default for MedrxivSource {
    fn default() -> Self {
        Self::new().expect("Failed to create MedrxivSource")
    }
}

#[async_trait]
impl Source for MedrxivSource {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn capabilities(&self) -> SourceCapabilities {
        self.inner.capabilities()
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        self.inner.search(query).await
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        self.inner.download(request).await
    }

    async fn read(&self, request: &ReadRequest) -> Result<ReadResult, SourceError> {
        self.inner.read(request).await
    }
}
