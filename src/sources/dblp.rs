//! DBLP research source implementation.

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};

const DBLP_BASE_URL: &str = "https://dblp.org";
const DBLP_SEARCH_URL: &str = "https://dblp.org/search/publ/api";

/// DBLP research source
///
/// Uses DBLP XML API for computer science bibliography.
#[derive(Debug, Clone)]
pub struct DblpSource {
    client: Arc<Client>,
}

impl DblpSource {
    pub fn new() -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
        }
    }

    /// Parse DBLP XML response into papers
    fn parse_xml(&self, xml_content: &str) -> Result<Vec<Paper>, SourceError> {
        let mut papers = Vec::new();

        // Simple XML parsing using string operations
        // For production, use a proper XML parser
        for hit_match in xml_content.matches("<hit") {
            // Find the full hit element
            let hit_start = xml_content.find(hit_match).unwrap();
            let hit_end = xml_content[hit_start..].find("</hit>")
                .map(|e| hit_start + e + 6)
                .unwrap_or(xml_content.len());
            let hit_xml = &xml_content[hit_start..hit_end];

            if let Some(paper) = self.parse_hit(hit_xml) {
                papers.push(paper);
            }
        }

        Ok(papers)
    }

    /// Parse a single DBLP hit element
    fn parse_hit(&self, hit_xml: &str) -> Option<Paper> {
        // Extract key attribute
        let key = self.extract_attr(hit_xml, "key")
            .unwrap_or_default();

        // Extract title
        let title = self.extract_element_text(hit_xml, "title")
            .unwrap_or_default();

        if title.is_empty() {
            return None;
        }

        // Extract authors
        let authors = self.extract_all_element_text(hit_xml, "author")
            .join("; ");

        // Extract year
        let year_elem = self.extract_element_text(hit_xml, "year")
            .and_then(|y| y.parse::<i32>().ok());

        let published_date = year_elem
            .map(|y| format!("{}", y))
            .unwrap_or_default();

        // Determine venue and type
        let (venue, pub_type) = if let Some(journal) = self.extract_element_text(hit_xml, "journal") {
            (journal, "journal")
        } else if let Some(booktitle) = self.extract_element_text(hit_xml, "booktitle") {
            (booktitle, "conference")
        } else if let Some(school) = self.extract_element_text(hit_xml, "school") {
            (school, "thesis")
        } else {
            (String::new(), "article")
        };

        // Extract DOI from ee element
        let doi = self.extract_element_text(hit_xml, "ee")
            .map(|ee| {
                if ee.contains("doi.org") {
                    ee.replace("https://doi.org/", "")
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        // Extract other metadata
        let volume = self.extract_element_text(hit_xml, "volume").unwrap_or_default();
        let number = self.extract_element_text(hit_xml, "number").unwrap_or_default();
        let pages = self.extract_element_text(hit_xml, "pages").unwrap_or_default();

        let url = format!("{}/rec/{}.html", DBLP_BASE_URL, key);

        // Abstract from note if present
        let abstract_text = self.extract_element_text(hit_xml, "note")
            .unwrap_or_default();

        Some(PaperBuilder::new(key.clone(), title, url, SourceType::DBLP)
            .authors(authors)
            .abstract_text(abstract_text[..abstract_text.len().min(2000)].to_string())
            .doi(doi)
            .published_date(published_date)
            .categories(venue.clone())
            .build())
    }

    /// Extract attribute value from element
    fn extract_attr(&self, xml: &str, attr_name: &str) -> Option<String> {
        let pattern = format!(r#"{}="([^"]*)""#, attr_name);
        if let Some(pos) = xml.find(&pattern) {
            // Simple extraction
            let start = xml.find(format!(r#"{}=""#, attr_name).as_str())?;
            let start = start + attr_name.len() + 2;
            let end = xml[start..].find('"')?;
            Some(xml[start..start + end].to_string())
        } else {
            None
        }
    }

    /// Extract text content of an element
    fn extract_element_text(&self, xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        let start = xml.find(&start_tag)?;
        let start = start + start_tag.len();
        let end = xml[start..].find(&end_tag)?;

        Some(xml[start..start + end].trim().to_string())
    }

    /// Extract all text content of elements with given tag
    fn extract_all_element_text(&self, xml: &str, tag: &str) -> Vec<String> {
        let mut results = Vec::new();
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);
        let mut search_start = 0;

        while let Some(start) = xml[search_start..].find(&start_tag) {
            let absolute_start = search_start + start;
            let content_start = absolute_start + start_tag.len();
            if let Some(end) = xml[content_start..].find(&end_tag) {
                let text = xml[content_start..content_start + end].trim().to_string();
                if !text.is_empty() {
                    results.push(text);
                }
                search_start = content_start + end + end_tag.len();
            } else {
                break;
            }
        }

        results
    }
}

impl Default for DblpSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for DblpSource {
    fn id(&self) -> &str {
        "dblp"
    }

    fn name(&self) -> &str {
        "DBLP"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let mut url = format!(
            "?q={}&h={}&format=xml",
            urlencoding::encode(&query.query),
            query.max_results.min(1000)
        );

        // Add year filter if specified
        if let Some(year) = &query.year {
            if year.contains('-') {
                let parts: Vec<&str> = year.split('-').collect();
                if parts.len() == 2 {
                    url = format!("{}&yearMin={}&yearMax={}", url, parts[0].trim(), parts[1].trim());
                }
            } else {
                url = format!("{}&yearMin={}&yearMax={}", url, year, year);
            }
        }

        let response = self
            .client
            .get(&format!("{}{}", DBLP_SEARCH_URL, url))
            .header("Accept", "application/xml")
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search DBLP: {}", e)))?;

        if response.status() == 204 {
            return Ok(SearchResponse::new(vec![], "DBLP", &query.query));
        }

        if !response.status().is_success() {
            return Err(SourceError::Api(format!(
                "DBLP API returned status: {}",
                response.status()
            )));
        }

        let xml_content = response
            .text()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to read XML: {}", e)))?;

        let papers = self.parse_xml(&xml_content)?;

        Ok(SearchResponse::new(papers, "DBLP", &query.query))
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        // DBLP doesn't directly support DOI lookup via API
        // Search by DOI string
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let mut query = SearchQuery::new(clean_doi.to_string()).max_results(1);

        let response = self.search(&query).await?;

        response
            .papers
            .first()
            .cloned()
            .ok_or_else(|| SourceError::NotFound("DOI not found".to_string()))
    }
}
