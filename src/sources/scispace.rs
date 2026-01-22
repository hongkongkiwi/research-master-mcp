//! SciSpace research source implementation.
//!
//! Uses the SciSpace (Typeset) API for searching and retrieving research papers.
//! API documentation: https://typeset.io/api
//!
//! SciSpace is free and requires no API key for basic search.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const SCISPACE_API_BASE: &str = "https://api.typeset.io/v1";

/// SciSpace research source
///
/// Uses the SciSpace (Typeset) API for searching and retrieving research papers.
/// Free to use with no API key required.
#[derive(Debug, Clone)]
pub struct ScispaceSource {
    client: Arc<HttpClient>,
}

impl ScispaceSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }
}

impl Default for ScispaceSource {
    fn default() -> Self {
        Self::new().expect("Failed to create ScispaceSource")
    }
}

#[async_trait]
impl Source for ScispaceSource {
    fn id(&self) -> &str {
        "scispace"
    }

    fn name(&self) -> &str {
        "SciSpace"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let max_results = query.max_results.min(50);

        let url = format!(
            "{}/papers/search?query={}&limit={}",
            SCISPACE_API_BASE,
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
                    .map_err(|e| SourceError::Network(format!("Failed to search SciSpace: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(SourceError::Api(format!(
                        "SciSpace API returned status {}: {}",
                        status, text
                    )));
                }

                let json: ScispaceResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse SciSpace response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let total = response.total_results.unwrap_or(0);
        let papers: Result<Vec<Paper>, SourceError> = response
            .papers
            .into_iter()
            .map(|paper| self.parse_result(&paper))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "SciSpace", &query.query);
        response.total_results = Some(total);
        Ok(response)
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let url = format!("{}/papers/doi/{}", SCISPACE_API_BASE, urlencoding::encode(&clean_doi));

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
                    .map_err(|e| SourceError::Network(format!("Failed to lookup DOI in SciSpace: {}", e)))?;

                if response.status() == 404 {
                    return Err(SourceError::NotFound(format!("Paper not found in SciSpace: {}", doi)));
                }

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "SciSpace API returned status: {}",
                        response.status()
                    )));
                }

                let json: ScispacePaper = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse SciSpace response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        self.parse_result(&response)
    }
}

impl ScispaceSource {
    fn parse_result(&self, paper: &ScispacePaper) -> Result<Paper, SourceError> {
        let id = paper.doi.clone().unwrap_or_else(|| paper.id.clone());
        let title = paper.title.clone().unwrap_or_default();
        let abstract_text = paper.abstract_text.clone().unwrap_or_default();

        let doi = paper.doi.clone().unwrap_or_default();

        let authors: String = paper
            .authors
            .iter()
            .filter_map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let year = paper.publication_year.clone().unwrap_or_default();
        let url = if !doi.is_empty() {
            format!("https://doi.org/{}", doi)
        } else {
            format!("https://typeset.io/papers/{}", paper.id)
        };

        let pdf_url = paper.pdf_url.clone();

        Ok(PaperBuilder::new(id, title, url, SourceType::Scispace)
            .authors(&authors)
            .published_date(&year)
            .abstract_text(&abstract_text)
            .doi(&doi)
            .pdf_url(pdf_url.unwrap_or_default())
            .build())
    }
}

/// SciSpace API response
#[derive(Debug, Deserialize)]
struct ScispaceResponse {
    total_results: Option<usize>,
    papers: Vec<ScispacePaper>,
}

#[derive(Debug, Deserialize)]
struct ScispacePaper {
    id: String,
    doi: Option<String>,
    title: Option<String>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    publication_year: Option<String>,
    authors: Vec<ScispaceAuthor>,
    pdf_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScispaceAuthor {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = ScispaceSource::new();
        assert!(source.is_ok());
    }
}
