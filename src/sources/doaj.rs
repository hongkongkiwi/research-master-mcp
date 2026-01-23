//! DOAJ (Directory of Open Access Journals) research source implementation.
//!
//! Uses the DOAJ API for searching open access journals and articles.
//! API documentation: <https://doaj.org/api/v2>
//!
//! DOAJ is free and requires no API key for basic search.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const DOAJ_API_BASE: &str = "https://doaj.org/api/v2/search/articles";

/// DOAJ research source
///
/// Uses the DOAJ API for searching open access journals and articles.
/// Free to use with no API key required.
#[derive(Debug, Clone)]
pub struct DoajSource {
    client: Arc<HttpClient>,
}

impl DoajSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }
}

impl Default for DoajSource {
    fn default() -> Self {
        Self::new().expect("Failed to create DoajSource")
    }
}

#[async_trait]
impl Source for DoajSource {
    fn id(&self) -> &str {
        "doaj"
    }

    fn name(&self) -> &str {
        "DOAJ"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let max_results = query.max_results.min(100);

        // DOAJ uses Elasticsearch query syntax
        let search_query = format!(
            "query={}",
            urlencoding::encode(&query.query)
        );

        let url = format!(
            "{}?{}&pageSize={}",
            DOAJ_API_BASE,
            search_query,
            max_results
        );

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let request = client.get(&url)
                    .header("Accept", "application/json");

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search DOAJ: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(SourceError::Api(format!(
                        "DOAJ API returned status {}: {}",
                        status, text
                    )));
                }

                let json: DoajResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse DOAJ response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let total = response.total_results.unwrap_or(0);
        let papers: Result<Vec<Paper>, SourceError> = response
            .results
            .into_iter()
            .map(|article| self.parse_result(&article))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "DOAJ", &query.query);
        response.total_results = Some(total);
        Ok(response)
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let url = format!("{}/doi/{}", DOAJ_API_BASE, urlencoding::encode(&clean_doi));

        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let request = client.get(&url)
                    .header("Accept", "application/json");

                let response = request
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to lookup DOI in DOAJ: {}", e)))?;

                if response.status() == 404 {
                    return Err(SourceError::NotFound(format!("Paper not found in DOAJ: {}", doi)));
                }

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "DOAJ API returned status: {}",
                        response.status()
                    )));
                }

                let json: DoajArticle = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse DOAJ response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        self.parse_result(&response)
    }
}

impl DoajSource {
    fn parse_result(&self, article: &DoajArticle) -> Result<Paper, SourceError> {
        let id = article.id.clone();
        let title = article.title.clone().unwrap_or_default();
        let abstract_text = article.abstract_text.clone().unwrap_or_default();

        let doi = article.doi.clone().unwrap_or_default();

        let authors: String = article
            .authors
            .iter()
            .filter_map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let year = article.publication_year.clone().unwrap_or_default();
        let url = if !doi.is_empty() {
            format!("https://doi.org/{}", doi)
        } else {
            format!("https://doaj.org/article/{}", id)
        };

        Ok(PaperBuilder::new(id, title, url, SourceType::Doaj)
            .authors(&authors)
            .published_date(&year)
            .abstract_text(&abstract_text)
            .doi(&doi)
            .build())
    }
}

/// DOAJ API response
#[derive(Debug, Deserialize)]
struct DoajResponse {
    total_results: Option<usize>,
    results: Vec<DoajArticle>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DoajArticle {
    id: String,
    doi: Option<String>,
    title: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    publication_year: Option<String>,
    authors: Vec<DoajAuthor>,
    journal: Option<DoajJournal>,
}

#[derive(Debug, Deserialize)]
struct DoajAuthor {
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DoajJournal {
    title: Option<String>,
    publisher: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = DoajSource::new();
        assert!(source.is_ok());
    }
}
