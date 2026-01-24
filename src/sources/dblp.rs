//! DBLP research source implementation.
//!
//! Uses DBLP XML API for computer science bibliography.

use async_trait::async_trait;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, SearchQuery, SearchResponse, SourceType};
use crate::sources::{Source, SourceCapabilities, SourceError};
use crate::utils::{api_retry_config, with_retry, HttpClient};

const DBLP_BASE_URL: &str = "https://dblp.org";
const DBLP_SEARCH_URL: &str = "https://dblp.org/search/publ/api";

/// DBLP research source
///
/// Uses DBLP XML API for computer science bibliography.
#[derive(Debug, Clone)]
pub struct DblpSource {
    client: Arc<HttpClient>,
}

impl DblpSource {
    pub fn new() -> Result<Self, SourceError> {
        Ok(Self {
            client: Arc::new(HttpClient::new()?),
        })
    }

    /// Parse DBLP XML response into papers using quick-xml
    fn parse_xml(&self, xml_content: &str) -> Result<Vec<Paper>, SourceError> {
        let mut reader = Reader::from_str(xml_content);
        // trim_text is not available in newer quick-xml, configure via config
        let mut buf = Vec::new();

        let mut papers = Vec::new();

        // State for parsing hit elements
        let mut in_hit = false;
        let mut current_hit_key = String::new();
        let mut current_hit_title = String::new();
        let mut current_authors: Vec<String> = Vec::new();
        let mut current_year = String::new();
        let mut current_venue = String::new();
        let mut current_pub_type = String::new();
        let mut current_doi = String::new();
        let mut current_abstract = String::new();
        let mut hit_depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"hit" {
                        in_hit = true;
                        hit_depth = 1;
                        // Extract key attribute
                        if let Some(key) = get_attr(e, "key") {
                            current_hit_key = key;
                        }
                        // Reset other fields
                        current_hit_title.clear();
                        current_authors.clear();
                        current_year.clear();
                        current_venue.clear();
                        current_pub_type.clear();
                        current_doi.clear();
                        current_abstract.clear();
                    } else if in_hit {
                        hit_depth += 1;
                        // Track what element we're entering for text extraction
                        // Convert element name to owned String to avoid lifetime issues
                        let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        match tag_name.as_str() {
                            "author" => {}
                            "year" => {}
                            "title" => {}
                            "journal" | "booktitle" | "school" => {}
                            "ee" => {}
                            "note" => {}
                            _ => {}
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_hit {
                        let text = e.unescape().unwrap_or_default();
                        let text = text.trim().to_string();
                        if !text.is_empty() {
                            // Store the text - we'll determine which field it belongs to
                            // based on the previous Start element
                            if current_hit_title.is_empty() {
                                current_hit_title = text;
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"hit" {
                        if hit_depth == 1 && !current_hit_key.is_empty() {
                            // Create paper from parsed hit
                            let url = format!("{}/rec/{}.html", DBLP_BASE_URL, current_hit_key);

                            // Limit abstract length
                            let abstract_text = if current_abstract.len() > 2000 {
                                current_abstract[..2000].to_string()
                            } else {
                                current_abstract.clone()
                            };

                            let paper = PaperBuilder::new(
                                current_hit_key.clone(),
                                current_hit_title.clone(),
                                url,
                                SourceType::DBLP,
                            )
                            .authors(current_authors.join("; "))
                            .abstract_text(abstract_text)
                            .doi(current_doi.clone())
                            .published_date(current_year.clone())
                            .categories(current_venue.clone())
                            .build();

                            papers.push(paper);
                        }
                        in_hit = false;
                    } else if in_hit {
                        hit_depth -= 1;
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {
                    // Ignore other events (Empty, Comment, CData, Decl, PI)
                    if in_hit {
                        // Handle Empty events for self-closing tags
                    }
                }
                Err(e) => {
                    return Err(SourceError::Parse(format!("XML parsing error: {}", e)));
                }
            }
            buf.clear();
        }

        // Fallback: if quick-xml parsing didn't find hits, try the text-based parsing
        // This handles edge cases where DBLP's XML format varies
        if papers.is_empty() {
            papers = self.parse_xml_fallback(xml_content)?;
        }

        Ok(papers)
    }

    /// Fallback text-based XML parsing for edge cases
    fn parse_xml_fallback(&self, xml_content: &str) -> Result<Vec<Paper>, SourceError> {
        let mut papers = Vec::new();

        // Find all hit elements using a more robust approach
        let hits: Vec<&str> = find_elements(xml_content, "hit")?;

        for hit_xml in hits {
            if let Some(paper) = self.parse_hit_fallback(hit_xml) {
                papers.push(paper);
            }
        }

        Ok(papers)
    }

    /// Parse a single DBLP hit element (fallback using simple text extraction)
    fn parse_hit_fallback(&self, hit_xml: &str) -> Option<Paper> {
        // Extract key attribute
        let key = extract_attribute(hit_xml, "key")
            .ok()
            .flatten()
            .unwrap_or_default();

        // Extract title
        let title = extract_element_text(hit_xml, "title")
            .ok()
            .flatten()
            .unwrap_or_default();

        if title.is_empty() {
            return None;
        }

        // Extract authors
        let authors: Vec<String> = extract_all_element_text(hit_xml, "author");
        let authors_str = authors.join("; ");

        // Extract year
        let year_elem = extract_element_text(hit_xml, "year")
            .ok()
            .flatten()
            .and_then(|y| y.parse::<i32>().ok());

        let published_date = year_elem.map(|y| format!("{}", y)).unwrap_or_default();

        // Determine venue and type
        let (venue, _pub_type) = extract_element_text(hit_xml, "journal")
            .ok()
            .flatten()
            .map(|j| (j, "journal"))
            .or_else(|| {
                extract_element_text(hit_xml, "booktitle")
                    .ok()
                    .flatten()
                    .map(|b| (b, "conference"))
            })
            .or_else(|| {
                extract_element_text(hit_xml, "school")
                    .ok()
                    .flatten()
                    .map(|s| (s, "thesis"))
            })
            .unwrap_or_else(|| (String::new(), "article"));

        // Extract DOI from ee element
        let doi = extract_element_text(hit_xml, "ee")
            .ok()
            .flatten()
            .map(|ee| {
                if ee.contains("doi.org") {
                    ee.replace("https://doi.org/", "")
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        let url = format!("{}/rec/{}.html", DBLP_BASE_URL, key);

        // Abstract from note if present
        let abstract_text: String = extract_element_text(hit_xml, "note")
            .ok()
            .flatten()
            .unwrap_or_default()
            .chars()
            .take(2000)
            .collect();

        Some(
            PaperBuilder::new(key.clone(), title, url, SourceType::DBLP)
                .authors(authors_str)
                .abstract_text(abstract_text)
                .doi(doi)
                .published_date(published_date)
                .categories(venue.clone())
                .build(),
        )
    }
}

impl Default for DblpSource {
    fn default() -> Self {
        Self::new().expect("Failed to create DblpSource")
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
                    url = format!(
                        "{}&yearMin={}&yearMax={}",
                        url,
                        parts[0].trim(),
                        parts[1].trim()
                    );
                }
            } else {
                url = format!("{}&yearMin={}&yearMax={}", url, year, year);
            }
        }

        // Clone values for retry closure
        let client = Arc::clone(&self.client);
        let url_for_retry = url.clone();

        let response = with_retry(api_retry_config(), || {
            let client = Arc::clone(&client);
            let url = url_for_retry.clone();
            async move {
                let response = client
                    .get(&format!("{}{}", DBLP_SEARCH_URL, url))
                    .header("Accept", "application/xml")
                    .send()
                    .await
                    .map_err(|e| SourceError::Network(format!("Failed to search DBLP: {}", e)))?;

                if response.status() == 204 {
                    return Err(SourceError::NotFound("No results".to_string()));
                }

                if !response.status().is_success() {
                    let status = response.status();
                    // DBLP may return 500 for server errors or rate limiting
                    // Return empty results gracefully
                    if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
                        tracing::debug!("DBLP API returned 500 - returning empty results");
                        return Err(SourceError::Api("DBLP server error".to_string()));
                    }
                    return Err(SourceError::Api(format!(
                        "DBLP API returned status: {}",
                        status
                    )));
                }

                Ok(response)
            }
        })
        .await;

        // Handle the case where 204 is returned (no results) or 500 server errors
        let response = match response {
            Ok(r) => r,
            Err(SourceError::NotFound(_)) => {
                return Ok(SearchResponse::new(vec![], "DBLP", &query.query));
            }
            Err(SourceError::Api(msg)) if msg.contains("server error") => {
                tracing::debug!("DBLP server error - returning empty results");
                return Ok(SearchResponse::new(vec![], "DBLP", &query.query));
            }
            Err(e) => return Err(e),
        };

        let xml_content = response
            .text()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to read XML: {}", e)))?;

        let papers = self.parse_xml(&xml_content)?;

        Ok(SearchResponse::new(papers, "DBLP", &query.query))
    }

    async fn get_by_doi(&self, doi: &str) -> Result<Paper, SourceError> {
        let clean_doi = doi
            .replace("https://doi.org/", "")
            .replace("doi:", "")
            .trim()
            .to_string();

        let query = SearchQuery::new(clean_doi.to_string()).max_results(1);

        let response = self.search(&query).await?;

        response
            .papers
            .first()
            .cloned()
            .ok_or_else(|| SourceError::NotFound("DOI not found".to_string()))
    }
}

// ========== Helper Functions ==========

/// Get attribute value from a BytesStart element
fn get_attr<'a>(e: &BytesStart<'a>, attr_name: &str) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == attr_name.as_bytes())
        .and_then(|a| {
            std::str::from_utf8(a.value.as_ref())
                .ok()
                .map(|s| s.to_string())
        })
}

/// Find all elements with given tag name, returns their content
fn find_elements<'a>(xml: &'a str, tag: &str) -> Result<Vec<&'a str>, SourceError> {
    let mut results = Vec::new();
    // Start tag can have attributes, so search for <tag followed by > or whitespace+>
    let start_pattern = format!(r#"<{}(?:\s|>)"#, tag);
    let end_tag = format!("</{}>", tag);

    // Use regex for start tag to handle attributes
    let start_re = regex::Regex::new(&start_pattern)
        .map_err(|e| SourceError::Parse(format!("Regex error: {}", e)))?;

    let mut search_start = 0;

    while let Some(start_match) = start_re.find(&xml[search_start..]) {
        // The match includes <tag followed by whitespace or >
        // Find the position of '>' in the matched portion to skip to after >
        let match_start = start_match.start();
        let matched_text = start_match.as_str();
        let gt_pos = matched_text.find('>').unwrap_or(matched_text.len() - 1);
        let abs_start = search_start + match_start + gt_pos + 1; // After >

        if let Some(end_pos) = xml[abs_start..].find(&end_tag) {
            let element_content = &xml[abs_start..abs_start + end_pos];
            results.push(element_content);
            search_start = abs_start + end_pos + end_tag.len();
        } else {
            break;
        }
    }

    Ok(results)
}

/// Extract text content of a single element
fn extract_element_text(xml: &str, tag: &str) -> Result<Option<String>, SourceError> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    if let Some(start) = xml.find(&start_tag) {
        let content_start = start + start_tag.len();
        if let Some(end) = xml[content_start..].find(&end_tag) {
            let text = xml[content_start..content_start + end].trim().to_string();
            if !text.is_empty() {
                return Ok(Some(text));
            }
        }
    }

    Ok(None)
}

/// Extract all text content of elements with given tag
fn extract_all_element_text(xml: &str, tag: &str) -> Vec<String> {
    let mut results = Vec::new();
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    let mut search_start = 0;

    while let Some(start) = xml[search_start..].find(&start_tag) {
        let abs_start = search_start + start;
        let content_start = abs_start + start_tag.len();

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

/// Extract attribute value from element
fn extract_attribute(xml: &str, attr_name: &str) -> Result<Option<String>, SourceError> {
    let pattern = format!(r#"{}=""#, attr_name);
    if let Some(pos) = xml.find(&pattern) {
        let start = pos + pattern.len(); // After key="
        if let Some(end) = xml[start..].find('"') {
            return Ok(Some(xml[start..start + end].to_string()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_element_text() {
        let xml = r#"<title>Test Paper</title>"#;
        let result = extract_element_text(xml, "title").unwrap();
        assert_eq!(result, Some("Test Paper".to_string()));

        let result = extract_element_text(xml, "author").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_all_element_text() {
        let xml = r#"<author>John Doe</author><author>Jane Smith</author>"#;
        let result = extract_all_element_text(xml, "author");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "John Doe");
        assert_eq!(result[1], "Jane Smith");
    }

    #[test]
    fn test_extract_attribute() {
        let xml = r#"<hit key="conf/chi/2024">"#;
        let result = extract_attribute(xml, "key").unwrap();
        assert_eq!(result, Some("conf/chi/2024".to_string()));
    }

    #[test]
    fn test_find_elements() {
        let xml = r#"<hit><title>Paper 1</title></hit><hit><title>Paper 2</title></hit>"#;
        let result = find_elements(xml, "hit").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("Paper 1"));
        assert!(result[1].contains("Paper 2"));
    }

    #[test]
    fn test_parse_hit_fallback() {
        let xml = r#"<hit key="conf/chi/2024">
            <title>Test Paper Title</title>
            <author>John Doe</author>
            <author>Jane Smith</author>
            <year>2024</year>
            <booktitle>CHI 2024</booktitle>
            <ee>https://doi.org/10.1145/1234567.1234568</ee>
        </hit>"#;

        let source = DblpSource::new().unwrap();
        let paper = source.parse_hit_fallback(xml);

        assert!(paper.is_some());
        let paper = paper.unwrap();
        assert_eq!(paper.paper_id, "conf/chi/2024");
        assert_eq!(paper.title, "Test Paper Title");
        assert!(paper.authors.contains("John Doe"));
        assert!(paper.authors.contains("Jane Smith"));
        assert_eq!(paper.published_date, Some("2024".to_string()));
        assert!(paper
            .categories
            .as_ref()
            .map(|c| c.contains("CHI 2024"))
            .unwrap_or(false));
        assert_eq!(paper.doi, Some("10.1145/1234567.1234568".to_string()));
    }

    #[test]
    fn test_parse_xml_fallback() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hits>
            <hit key="conf/chi/2024a">
                <title>Paper A</title>
                <author>Author A</author>
                <year>2024</year>
            </hit>
            <hit key="conf/chi/2024b">
                <title>Paper B</title>
                <author>Author B</author>
                <year>2024</year>
            </hit>
        </hits>"#;

        let source = DblpSource::new().unwrap();
        let papers = source.parse_xml_fallback(xml).unwrap();

        assert_eq!(papers.len(), 2);
        assert_eq!(papers[0].paper_id, "conf/chi/2024a");
        assert_eq!(papers[1].paper_id, "conf/chi/2024b");
    }
}
