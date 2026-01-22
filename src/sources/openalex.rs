//! OpenAlex research source implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{
    CitationRequest, DownloadRequest, DownloadResult, ReadRequest, ReadResult, Source,
    SourceCapabilities, SourceError,
};

const OPENALEX_API_BASE: &str = "https://api.openalex.org";

/// OpenAlex research source
///
/// Uses the OpenAlex REST API.
#[derive(Debug, Clone)]
pub struct OpenAlexSource {
    client: Arc<Client>,
    email: Option<String>,
}

impl OpenAlexSource {
    /// Create a new OpenAlex source
    pub fn new() -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION"),
                        " (mailto:)" // Placeholder for polite pool
                    ))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
            email: std::env::var("OPENALEX_EMAIL").ok(),
        }
    }

    /// Create with an email (recommended for better rate limits)
    pub fn with_email(email: String) -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .user_agent(format!(
                        "{}/{} (mailto:{})",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION"),
                        email
                    ))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
            email: Some(email),
        }
    }

    /// Build request URL
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", OPENALEX_API_BASE, endpoint)
    }

    /// Add email to request URL if available (for polite pool)
    fn add_email_if_present(&self, url: &str) -> String {
        if let Some(ref email) = self.email {
            format!("{}&mailto={}", url, urlencoding::encode(email))
        } else {
            url.to_string()
        }
    }

    /// Parse OpenAlex paper data
    fn parse_paper(data: &OAPaper) -> Paper {
        let authors = data
            .authorships
            .iter()
            .filter_map(|a| a.author.display_name.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        let published_date = data.publication_year.as_ref().map(|y| y.to_string());

        let doi = data.doi.clone().unwrap_or_default();

        let url = data.id.clone().unwrap_or_default();

        let paper_id = data
            .id
            .clone()
            .unwrap_or_else(|| format!("OpenAlex:{}", doi));

        let pdf_url = data.best_open_access_pdf.as_ref().and_then(|p| p.url.clone());

        PaperBuilder::new(paper_id, data.title.clone(), url, SourceType::OpenAlex)
            .authors(authors)
            .abstract_text(data.r#abstract.clone().unwrap_or_default())
            .doi(doi)
            .published_date(published_date.unwrap_or_default())
            .pdf_url(pdf_url.unwrap_or_default())
            .citations(data.cited_by_count.unwrap_or(0) as u32)
            .build()
    }
}

impl Default for OpenAlexSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for OpenAlexSource {
    fn id(&self) -> &str {
        "openalex"
    }

    fn name(&self) -> &str {
        "OpenAlex"
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
        let mut url = format!(
            "/works?search={}&per-page={}",
            urlencoding::encode(&query.query),
            query.max_results
        );

        // Add year filter if specified
        if let Some(year) = &query.year {
            if let Some(start) = year.strip_suffix('-') {
                // From year: from_year onwards
                url = format!("{}&filter=publication_year:{}", url, start);
            } else if let Some(end) = year.strip_prefix('-') {
                // Until year
                url = format!("{}&filter=publication_year:|-{}", url, end);
            } else if year.contains('-') {
                // Range
                url = format!("{}&filter=publication_year:{}", url, year);
            } else if year.len() == 4 {
                // Single year
                url = format!("{}&filter=publication_year:{}", url, year);
            }
        }

        url = self.add_email_if_present(&url);

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search OpenAlex: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::Api(format!(
                "OpenAlex API returned status: {}",
                response.status()
            )));
        }

        let data: WorksResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .results
            .into_iter()
            .map(|p| Ok(Self::parse_paper(&p)))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "OpenAlex", &query.query);
        response.total_results = Some(data.meta.count);
        Ok(response)
    }

    async fn search_by_author(
        &self,
        author: &str,
        max_results: usize,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/authors?search={}&per-page=1",
            urlencoding::encode(author)
        );

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search author: {}", e)))?;

        let author_data: AuthorsResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let author_id = author_data
            .results
            .first()
            .and_then(|a| a.id.clone())
            .ok_or_else(|| SourceError::NotFound("Author not found".to_string()))?;

        // Get papers by author
        let papers_url = format!(
            "/authors/{}/works?per-page={}",
            author_id, max_results
        );

        let papers_response = self
            .client
            .get(&self.build_url(&papers_url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch author papers: {}", e)))?;

        let papers_data: WorksResponse = papers_response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = papers_data
            .results
            .into_iter()
            .map(|p| Ok(Self::parse_paper(&p)))
            .collect();

        Ok(SearchResponse::new(papers?, "OpenAlex", author))
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        // Try to get the paper first to find PDF URL
        let url = format!("/works/{}", request.paper_id);

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch paper: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!(
                "Paper not found: {}",
                request.paper_id
            )));
        }

        let data: WorkResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let pdf_url = data
            .best_open_access_pdf
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
            "/works?filter=cites:{}&per-page={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch citations: {}", e)))?;

        let data: WorksResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .results
            .into_iter()
            .map(|p| Ok(Self::parse_paper(&p)))
            .collect();

        Ok(SearchResponse::new(papers?, "OpenAlex", &request.paper_id))
    }

    async fn get_references(
        &self,
        request: &CitationRequest,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/works?filter=referenceds:{}&per-page={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch references: {}", e)))?;

        let data: WorksResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .results
            .into_iter()
            .map(|p| Ok(Self::parse_paper(&p)))
            .collect();

        Ok(SearchResponse::new(papers?, "OpenAlex", &request.paper_id))
    }

    async fn get_related(
        &self,
        request: &CitationRequest,
    ) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "/works?filter=related:{}&per-page={}",
            urlencoding::encode(&request.paper_id),
            request.max_results
        );

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch related: {}", e)))?;

        let data: WorksResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .results
            .into_iter()
            .map(|p| Ok(Self::parse_paper(&p)))
            .collect();

        Ok(SearchResponse::new(papers?, "OpenAlex", &request.paper_id))
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let url = format!("/works/doi:{}", urlencoding::encode(doi));

        let response = self
            .client
            .get(&self.build_url(&url))
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch DOI: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound("DOI not found".to_string()));
        }

        let data: WorkResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        // Parse WorkResponse (same structure as OAPaper)
        let authors = data
            .authorships
            .iter()
            .filter_map(|a| a.author.display_name.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        let published_date = data.publication_year.as_ref().map(|y| y.to_string());
        let doi_value = data.doi.clone().unwrap_or_default();
        let url = data.id.clone().unwrap_or_default();
        let paper_id = data.id.clone().unwrap_or_else(|| format!("OpenAlex:{}", doi_value));
        let pdf_url = data.best_open_access_pdf.as_ref().and_then(|p| p.url.clone()).unwrap_or_default();

        Ok(PaperBuilder::new(paper_id, data.title.clone(), url, SourceType::OpenAlex)
            .authors(authors)
            .abstract_text(data.r#abstract.clone().unwrap_or_default())
            .doi(doi_value)
            .published_date(published_date.unwrap_or_default())
            .pdf_url(pdf_url)
            .citations(data.cited_by_count.unwrap_or(0) as u32)
            .build())
    }
}

// ===== OpenAlex API Types =====

#[derive(Debug, Deserialize)]
struct OAPaper {
    id: Option<String>,
    title: String,
    publication_year: Option<i32>,
    #[serde(rename = "cited_by_count")]
    cited_by_count: Option<i32>,
    doi: Option<String>,
    r#abstract: Option<String>,
    best_open_access_pdf: Option<OAPdf>,
    authorships: Vec<OAAuthorship>,
}

#[derive(Debug, Deserialize)]
struct OAPdf {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAAuthorship {
    author: OAAuthor,
}

#[derive(Debug, Deserialize)]
struct OAAuthor {
    #[serde(rename = "display_name")]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorksResponse {
    results: Vec<OAPaper>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Meta {
    count: usize,
}

#[derive(Debug, Deserialize)]
struct WorkResponse {
    id: Option<String>,
    title: String,
    publication_year: Option<i32>,
    #[serde(rename = "cited_by_count")]
    cited_by_count: Option<i32>,
    doi: Option<String>,
    r#abstract: Option<String>,
    best_open_access_pdf: Option<OAPdf>,
    authorships: Vec<OAAuthorship>,
}

#[derive(Debug, Deserialize)]
struct AuthorsResponse {
    results: Vec<OAAuthorData>,
}

#[derive(Debug, Deserialize)]
struct OAAuthorData {
    id: Option<String>,
}
