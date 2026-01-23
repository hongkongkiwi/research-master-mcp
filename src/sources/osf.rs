//! OSF Preprints research source implementation.
//!
//! Uses the OSF API for searching preprints from the Open Science Framework.
//! API documentation: <https://developer.osf.io>
//!
//! OSF is free and requires no API key for public data.

use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const OSF_API_BASE: &str = "https://api.osf.io/v2";

/// OSF Preprints research source
///
/// Uses the OSF API for searching preprints from the Open Science Framework.
/// Free to use with no API key required.
#[derive(Debug, Clone)]
pub struct OsfSource {
    client: Arc<HttpClient>,
}

impl OsfSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }
}

impl Default for OsfSource {
    fn default() -> Self {
        Self::new().expect("Failed to create OsfSource")
    }
}

#[async_trait]
impl Source for OsfSource {
    fn id(&self) -> &str {
        "osf"
    }

    fn name(&self) -> &str {
        "OSF Preprints"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD | SourceCapabilities::DOI_LOOKUP
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let max_results = query.max_results.min(100);

        let url = format!(
            "{}?filter[title]={}&page[size]={}",
            OSF_API_BASE,
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
                    .map_err(|e| SourceError::Network(format!("Failed to search OSF: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    return Err(SourceError::Api(format!(
                        "OSF API returned status {}: {}",
                        status, text
                    )));
                }

                let json: OsfResponse = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse OSF response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        let total = response.total_results.unwrap_or(0);
        let papers: Result<Vec<Paper>, SourceError> = response
            .data
            .into_iter()
            .map(|item| self.parse_result(&item))
            .collect();

        let papers = papers?;
        let mut response = SearchResponse::new(papers, "OSF Preprints", &query.query);
        response.total_results = Some(total);
        Ok(response)
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let url = format!("{}/preprints/{}", OSF_API_BASE, urlencoding::encode(&clean_doi));

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
                    .map_err(|e| SourceError::Network(format!("Failed to lookup DOI in OSF: {}", e)))?;

                if response.status() == 404 {
                    return Err(SourceError::NotFound(format!("Paper not found in OSF: {}", doi)));
                }

                if !response.status().is_success() {
                    return Err(SourceError::Api(format!(
                        "OSF API returned status: {}",
                        response.status()
                    )));
                }

                let json: OsfPreprint = response
                    .json()
                    .await
                    .map_err(|e| SourceError::Parse(format!("Failed to parse OSF response: {}", e)))?;

                Ok(json)
            }
        })
        .await?;

        self.parse_result(&response)
    }

    async fn download(&self, request: &crate::models::DownloadRequest) -> Result<crate::models::DownloadResult, SourceError> {
        // Use PDF URL from paper_id or DOI
        let paper_id = request.paper_id.clone();
        let pdf_url = request.doi.clone()
            .map(|doi| format!("https://doi.org/{}", doi))
            .ok_or_else(|| SourceError::InvalidRequest("DOI required for OSF download".to_string()))?;
        let save_path = request.save_path.clone();

        let client = Arc::clone(&self.client);

        let response = client.get(&pdf_url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download from OSF: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::Api(format!("Download failed with status: {}", response.status())));
        }

        let bytes_vec = response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read download: {}", e)))?;

        tokio::fs::write(&save_path, &bytes_vec)
            .await
            .map_err(|e| SourceError::Io(e))?;

        Ok(crate::models::DownloadResult {
            path: save_path,
            bytes: bytes_vec.len() as u64,
            success: true,
            error: None,
        })
    }
}

impl OsfSource {
    fn parse_result(&self, preprint: &OsfPreprint) -> Result<Paper, SourceError> {
        let id = preprint.id.clone();
        let title = preprint.attributes.title.clone().unwrap_or_default();
        let abstract_text = preprint.attributes.description.clone().unwrap_or_default();

        let doi = preprint.attributes.doi.clone().unwrap_or_default();

        let authors: String = preprint
            .relationships
            .authors
            .data
            .iter()
            .filter_map(|a| a.attributes.name.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let date_created = preprint.attributes.date_created.clone().unwrap_or_default();
        let year = date_created.split('-').next().unwrap_or(&date_created).to_string();
        let url = preprint.links.html.clone().unwrap_or_default();
        let pdf_url = preprint.links.download.clone();

        Ok(PaperBuilder::new(id, title, url, SourceType::Osf)
            .authors(&authors)
            .published_date(&year)
            .abstract_text(&abstract_text)
            .doi(&doi)
            .pdf_url(pdf_url.unwrap_or_default())
            .build())
    }
}

/// OSF API response
#[derive(Debug, Deserialize)]
struct OsfResponse {
    total_results: Option<usize>,
    data: Vec<OsfPreprint>,
}

#[derive(Debug, Deserialize)]
struct OsfPreprint {
    id: String,
    attributes: OsfAttributes,
    relationships: OsfRelationships,
    links: OsfLinks,
}

#[derive(Debug, Deserialize)]
struct OsfAttributes {
    title: Option<String>,
    description: Option<String>,
    doi: Option<String>,
    date_created: Option<String>,
    date_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsfRelationships {
    authors: OsfAuthors,
}

#[derive(Debug, Deserialize)]
struct OsfAuthors {
    data: Vec<OsfAuthor>,
}

#[derive(Debug, Deserialize)]
struct OsfAuthor {
    id: String,
    attributes: OsfAuthorAttributes,
}

#[derive(Debug, Deserialize)]
struct OsfAuthorAttributes {
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsfLinks {
    html: Option<String>,
    download: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_creation() {
        let source = OsfSource::new();
        assert!(source.is_ok());
    }
}
