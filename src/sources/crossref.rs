//! CrossRef research source implementation.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const CROSSREF_API_BASE: &str = "https://api.crossref.org";

/// CrossRef research source
///
/// Uses CrossRef REST API for DOI metadata lookup and search.
#[derive(Debug, Clone)]
pub struct CrossRefSource {
    client: Arc<HttpClient>,
}

impl CrossRefSource {
    pub fn new() -> Result<Self, SourceError> {
        let user_agent = format!(
            "{} / {} (mailto:crossref@crossref.org)",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        Ok(Self {
            client: Arc::new(HttpClient::with_user_agent(&user_agent)?),
        })
    }
}

impl Default for CrossRefSource {
    fn default() -> Self {
        Self::new().expect("Failed to create CrossRefSource")
    }
}

#[async_trait]
impl Source for CrossRefSource {
    fn id(&self) -> &str {
        "crossref"
    }

    fn name(&self) -> &str {
        "CrossRef"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let mut url = format!(
            "{}/works?query={}&rows={}",
            CROSSREF_API_BASE,
            urlencoding::encode(&query.query),
            query.max_results
        );

        // Add year filter if specified
        if let Some(year) = &query.year {
            url = format!("{}&filter=from-pub-date-year:{}", url, year);
        }

        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let response = client.get(&url).send().await.map_err(|e| {
                    SourceError::Network(format!("Failed to search CrossRef: {}", e))
                })?;

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "CrossRef API returned status: {}",
                        response.status()
                    )));
                }

                Ok(response)
            }
        })
        .await?;

        let data: CRResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let papers: Vec<Paper> = data
            .message
            .items
            .into_iter()
            .filter_map(|item| {
                let title = item.title.clone().unwrap_or_default();

                let authors = item
                    .author
                    .iter()
                    .filter_map(|a| a.given.as_ref())
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");

                let doi = item.doi.clone().unwrap_or_default();

                let url = item.url.clone().unwrap_or_default();

                let published_date = item
                    .published_print
                    .as_ref()
                    .and_then(|d| d.date.clone())
                    .unwrap_or_default();

                if title.is_empty() {
                    return None;
                }

                Some(
                    PaperBuilder::new(doi.clone(), title, url, SourceType::CrossRef)
                        .authors(authors)
                        .doi(doi)
                        .published_date(published_date)
                        .build(),
                )
            })
            .collect();

        Ok(SearchResponse::new(papers, "CrossRef", &query.query)
            .total_results(data.message.total_results))
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let url = format!("{}/works/{}", CROSSREF_API_BASE, urlencoding::encode(doi));

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch DOI: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound("DOI not found".to_string()));
        }

        let data: CRResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let item = data
            .message
            .items
            .first()
            .ok_or_else(|| SourceError::NotFound("DOI not found".to_string()))?;

        let title = item.title.clone().unwrap_or_default();

        let authors = item
            .author
            .iter()
            .filter_map(|a| a.given.as_ref())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        let doi = item.doi.clone().unwrap_or_default();

        let url = item.url.clone().unwrap_or_default();

        let published_date = item
            .published_print
            .as_ref()
            .and_then(|d| d.date.clone())
            .unwrap_or_default();

        Ok(
            PaperBuilder::new(doi.clone(), title, url, SourceType::CrossRef)
                .authors(authors)
                .doi(doi)
                .published_date(published_date)
                .build(),
        )
    }
}

// ===== CrossRef API Types =====

#[derive(Debug, Deserialize)]
struct CRResponse {
    message: CRMessage,
}

#[derive(Debug, Deserialize)]
struct CRMessage {
    #[serde(rename = "total-results")]
    total_results: usize,
    items: Vec<CRItem>,
}

#[derive(Debug, Deserialize)]
struct CRIter {
    #[serde(rename = "given")]
    given: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CRItem {
    title: Option<String>,
    doi: Option<String>,
    url: Option<String>,
    author: Vec<CRIter>,
    #[serde(rename = "published-print")]
    published_print: Option<CRDate>,
}

#[derive(Debug, Deserialize)]
struct CRDate {
    date: Option<String>,
}
