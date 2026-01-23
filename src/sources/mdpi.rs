//! MDPI research source implementation.
//!
//! Uses the MDPI API for searching and retrieving research papers.
//! API documentation: <https://developer.mdpi.com>

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const MDPI_API_BASE: &str = "https://api.mdpi.com/v1";

/// MDPI research source
///
/// Uses the MDPI API for searching and retrieving research papers.
/// MDPI is free and requires no API key for basic search.
#[derive(Debug, Clone)]
pub struct MdpiSource {
    client: Arc<HttpClient>,
}

impl MdpiSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }
}

impl Default for MdpiSource {
    fn default() -> Self {
        Self::new().expect("Failed to create MdpiSource")
    }
}

#[async_trait]
impl Source for MdpiSource {
    fn id(&self) -> &str {
        "mdpi"
    }

    fn name(&self) -> &str {
        "MDPI"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let max_results = query.max_results.min(100);

        let url = format!(
            "{}?query={}&page_size={}",
            MDPI_API_BASE,
            urlencoding::encode(&query.query),
            max_results
        );

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let request = client.get(&url);

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search MDPI: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(SourceError::Api(format!(
                        "MDPI API returned status {}: {}",
                        status, text
                    )));
                }

                let json: MdpiResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse MDPI response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let total = response.total_results.unwrap_or(0);
        let papers: Result<Vec<Paper>, SourceError> = response
            .items
            .into_iter()
            .map(|item| self.parse_result(&item))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "MDPI", &query.query);
        response.total_results = Some(total);
        Ok(response)
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let url = format!("{}/doi/{}", MDPI_API_BASE, urlencoding::encode(&clean_doi));

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let request = client.get(&url);

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to lookup DOI in MDPI: {}", e)))?;

                if response.status() == 404 {
                    return Err(SourceError::NotFound(format!("Paper not found in MDPI: {}", doi)));
                }

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "MDPI API returned status: {}",
                        response.status()
                    )));
                }

                response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse MDPI response: {}", e)))
            }
        })
        .await?;

        self.parse_result(&response)
    }
}

impl MdpiSource {
    fn parse_result(&self, item: &MdpiItem) -> Result<Paper, SourceError> {
        let id = item.doi.clone().unwrap_or_else(|| item.id.clone());
        let title = item.title.clone().unwrap_or_default();
        let abstract_text = item.abstract_text.clone().unwrap_or_default();

        let doi = item.doi.clone().unwrap_or_default();

        let authors: String = item
            .authors
            .iter()
            .filter_map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let year = item.publication_date.clone().unwrap_or_default();
        let url = if !doi.is_empty() {
            format!("https://doi.org/{}", doi)
        } else {
            format!("https://www.mdpi.com/{}", item.id)
        };

        Ok(PaperBuilder::new(id, title, url, SourceType::Other("mdpi".to_string()))
            .authors(&authors)
            .published_date(&year)
            .abstract_text(&abstract_text)
            .doi(&doi)
            .build())
    }
}

/// MDPI API response
#[derive(Debug, Deserialize)]
struct MdpiResponse {
    total_results: Option<usize>,
    items: Vec<MdpiItem>,
}

#[derive(Debug, Deserialize)]
struct MdpiItem {
    id: String,
    doi: Option<String>,
    title: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    publication_date: Option<String>,
    authors: Vec<MdpiAuthor>,
}

#[derive(Debug, Deserialize)]
struct MdpiAuthor {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = MdpiSource::new();
        assert!(source.is_ok());
    }
}
