//! IACR ePrint research source implementation.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{
    DownloadRequest, DownloadResult, ReadRequest, ReadResult, Source, SourceCapabilities,
    SourceError,
};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const IACR_SEARCH_URL: &str = "https://eprint.iacr.org/search";
const IACR_PDF_URL: &str = "https://eprint.iacr.org";

/// IACR ePrint research source
///
/// Uses web scraping for IACR ePrint archive.
#[derive(Debug, Clone)]
pub struct IacrSource {
    client: Arc<HttpClient>,
}

impl IacrSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }
}

impl Default for IacrSource {
    fn default() -> Self {
        Self::new().expect("Failed to create IacrSource")
    }
}

#[async_trait]
impl Source for IacrSource {
    fn id(&self) -> &str {
        "iacr"
    }

    fn name(&self) -> &str {
        "IACR ePrint"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD | SourceCapabilities::READ
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let url = format!("?q={}&format=json", urlencoding::encode(&query.query));

        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                client
                    .get(&format!("{}{}", IACR_SEARCH_URL, url))
                    .header("Accept", "application/json")
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search IACR: {}", e)))
            }
        })
        .await?;

        if !response.status().is_success() {
            return Err(SourceError::Api(format!(
                "IACR API returned status: {}",
                response.status()
            )));
        }

        let data: IACRResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Result<Vec<Paper>, SourceError> = data
            .papers
            .iter()
            .take(query.max_results)
            .map(|p| {
                let authors = p
                    .authors
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");

                let url = format!("{}{}", IACR_SEARCH_URL, p.url);

                Ok(
                    PaperBuilder::new(p.id.clone(), p.title.clone(), url, SourceType::IACR)
                        .authors(authors)
                        .abstract_text(p.r#abstract.clone().unwrap_or_default())
                        .published_date(p.published.clone())
                        .pdf_url(format!("{}/{}.pdf", IACR_PDF_URL, p.id))
                        .build(),
                )
            })
            .collect();

        let papers = papers?;
        Ok(SearchResponse::new(papers, "IACR", &query.query))
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        let pdf_url = format!("{}/{}.pdf", IACR_PDF_URL, request.paper_id);

        let response = self
            .client
            .get(&pdf_url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download PDF: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!(
                "Paper not found: {}",
                request.paper_id
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read PDF: {}", e)))?;

        std::fs::create_dir_all(&request.save_path).map_err(|e| {
            SourceError::Io(std::io::Error::other(format!(
                "Failed to create directory: {}",
                e
            )))
        })?;

        let filename = format!("{}.pdf", request.paper_id);
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

// ===== IACR API Types =====

#[derive(Debug, Deserialize)]
struct IACRResponse {
    papers: Vec<IACRPaper>,
}

#[derive(Debug, Deserialize)]
struct IACRPaper {
    id: String,
    title: String,
    authors: Vec<IACRAuthor>,
    r#abstract: Option<String>,
    published: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct IACRAuthor {
    name: String,
}
