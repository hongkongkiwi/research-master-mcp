//! Springer research source implementation.
//!
//! Uses the Springer Nature API for searching publications from Springer Link.
//! API documentation: <https://dev.springernature.com>
//!
//! Requires a free API key from developer.springernature.com

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const SPRINGER_API_BASE: &str = "https://api.springernature.com/metadata/json";

/// Springer research source
///
/// Uses the Springer Nature API for searching publications from Springer Link.
/// Requires a free API key from developer.springernature.com
#[derive(Debug, Clone)]
pub struct SpringerSource {
    client: Arc<HttpClient>,
    api_key: Option<String>,
}

impl SpringerSource {
    pub fn new() -> Result<Self, SourceError> {
        let api_key = std::env::var("SPRINGER_API_KEY").ok();
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
            api_key,
        })
    }
}

impl Default for SpringerSource {
    fn default() -> Self {
        Self::new().expect("Failed to create SpringerSource")
    }
}

#[async_trait]
impl Source for SpringerSource {
    fn id(&self) -> &str {
        "springer"
    }

    fn name(&self) -> &str {
        "Springer"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        if self.api_key.is_none() {
            tracing::warn!("SPRINGER_API_KEY not set - limited functionality");
        }

        let max_results = query.max_results.min(100);

        let url = format!(
            "{}?q={}&p={}",
            SPRINGER_API_BASE,
            urlencoding::encode(&query.query),
            max_results
        );

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();
        let api_key = self.api_key.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            let api_key = api_key.clone();
            async move {
                let mut request = client.get(&url);

                if let Some(key) = api_key {
                    request = request.header("Authorization", format!("Bearer {}", key));
                }

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search Springer: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(SourceError::Api(format!(
                        "Springer API returned status {}: {}",
                        status, text
                    )));
                }

                let json: SpringerResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse Springer response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let total = response.total_results.unwrap_or(0);
        let papers: Result<Vec<Paper>, SourceError> = response
            .records
            .into_iter()
            .map(|record| self.parse_result(&record))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "Springer", &query.query);
        response.total_results = Some(total);
        Ok(response)
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let url = format!("{}?q=doi:\"{}\"&p=1", SPRINGER_API_BASE, urlencoding::encode(&clean_doi));

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();
        let api_key = self.api_key.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            let api_key = api_key.clone();
            async move {
                let mut request = client.get(&url);

                if let Some(key) = api_key {
                    request = request.header("Authorization", format!("Bearer {}", key));
                }

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to lookup DOI in Springer: {}", e)))?;

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "Springer API returned status: {}",
                        response.status()
                    )));
                }

                let json: SpringerResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse Springer response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let record = response.records.into_iter().next()
            .ok_or_else(|| SourceError::NotFound(format!("Paper not found in Springer: {}", doi)))?;

        self.parse_result(&record)
    }
}

impl SpringerSource {
    fn parse_result(&self, record: &SpringerRecord) -> Result<Paper, SourceError> {
        let id = record.doi.clone().unwrap_or_else(|| record.id.clone());
        let title = record.title.clone().unwrap_or_default();
        let abstract_text = record.abstract_text.clone().unwrap_or_default();

        let doi = record.doi.clone().unwrap_or_default();

        let authors: String = record
            .authors
            .iter()
            .filter_map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let year = record.publication_date.clone().unwrap_or_default();
        let url = if !doi.is_empty() {
            format!("https://doi.org/{}", doi)
        } else {
            record.url.clone().unwrap_or_default()
        };

        Ok(PaperBuilder::new(id, title, url, SourceType::Springer)
            .authors(&authors)
            .published_date(&year)
            .abstract_text(&abstract_text)
            .doi(&doi)
            .build())
    }
}

/// Springer API response
#[derive(Debug, Deserialize)]
struct SpringerResponse {
    total_results: Option<usize>,
    records: Vec<SpringerRecord>,
}

#[derive(Debug, Deserialize)]
struct SpringerRecord {
    id: String,
    doi: Option<String>,
    title: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    publication_date: Option<String>,
    authors: Vec<SpringerAuthor>,
    url: Option<String>,
    journal: Option<SpringerJournal>,
}

#[derive(Debug, Deserialize)]
struct SpringerAuthor {
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SpringerJournal {
    title: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = SpringerSource::new();
        assert!(source.is_ok());
    }
}
