//! PubMed Central (PMC) research source implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;

use crate::models::{Paper, PaperBuilder, ReadRequest, ReadResult, SearchQuery, SearchResponse, SourceType};
use crate::sources::{DownloadRequest, DownloadResult, Source, SourceCapabilities, SourceError};

const PMC_EUTILS_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const PMC_BASE_URL: &str = "https://www.ncbi.nlm.nih.gov/pmc";

/// PMC research source
///
/// Uses NCBI E-utilities API for PubMed Central full-text papers.
#[derive(Debug, Clone)]
pub struct PmcSource {
    client: Arc<Client>,
}

impl PmcSource {
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

    /// Clean PMCID (remove PMC prefix if present)
    fn clean_pmcid(&self, pmcid: &str) -> String {
        pmcid.replace("PMC", "").trim().to_string()
    }

    /// Parse PMC XML response into Paper
    fn parse_pmc_xml(&self, xml_content: &str, pmcid: &str) -> Result<Paper, SourceError> {
        // For simplicity, we'll extract key data using regex patterns
        // A production implementation would use a proper XML parser

        let title = self
            .extract_xml_text(xml_content, "article-title")
            .unwrap_or_default();

        let abstract_text = self
            .extract_xml_text(xml_content, "abstract")
            .unwrap_or_default();

        let authors = self.extract_authors(xml_content);

        let published_date = self.extract_date(xml_content);

        let doi = self
            .extract_attribute(xml_content, "article-id", "pub-id-type", "doi")
            .unwrap_or_default();

        let journal = self
            .extract_xml_text(xml_content, "journal-title")
            .unwrap_or_default();

        let full_pmcid = format!("PMC{}", pmcid);
        let url = format!("{}/{}", PMC_BASE_URL, full_pmcid);
        let pdf_url = format!("{}/articles/{}/pdf/", PMC_BASE_URL, full_pmcid);

        Ok(PaperBuilder::new(full_pmcid.clone(), title, url, SourceType::PMC)
            .authors(authors)
            .abstract_text(abstract_text)
            .doi(doi)
            .published_date(published_date)
            .categories(journal)
            .pdf_url(pdf_url)
            .build())
    }

    /// Extract text content from an XML element
    fn extract_xml_text(&self, xml: &str, tag_name: &str) -> Option<String> {
        let start_pattern = format!("<{}>", tag_name);
        let end_pattern = format!("</{}>", tag_name);

        let start = xml.find(&start_pattern)?;
        let end = xml.find(&end_pattern)?;

        if end > start {
            let content = &xml[start + start_pattern.len()..end];
            // Strip inner tags
            let text = regex::Regex::new(r"<[^>]+>")
                .ok()?
                .replace_all(content, " ")
                .to_string();
            Some(text.trim().to_string())
        } else {
            None
        }
    }

    /// Extract authors from XML
    fn extract_authors(&self, xml: &str) -> String {
        let mut authors = Vec::new();

        // Find all contrib elements with contrib-type="author"
        let pattern = r#"<contrib[^>]*contrib-type\s*=\s*["']author["'][^>]*>"#;
        if let Ok(re) = regex::Regex::new(pattern) {
            for mut match_cap in re.find_iter(xml) {
                let start = match_cap.end();
                // Find closing </contrib>
                if let Some(end) = xml[start..].find("</contrib>") {
                    let contrib_xml = &xml[start..start + end];

                    // Extract given-names and surname
                    let given = self
                        .extract_xml_text(contrib_xml, "given-names")
                        .unwrap_or_default();
                    let surname = self
                        .extract_xml_text(contrib_xml, "surname")
                        .unwrap_or_default();

                    if !given.is_empty() && !surname.is_empty() {
                        authors.push(format!("{} {}", given, surname));
                    } else if !surname.is_empty() {
                        authors.push(surname);
                    }
                }
            }
        }

        authors.join("; ")
    }

    /// Extract publication date from XML
    fn extract_date(&self, xml: &str) -> String {
        let year = self.extract_xml_text(xml, "year").unwrap_or_default();
        let month = self.extract_xml_text(xml, "month").unwrap_or_default();
        let day = self.extract_xml_text(xml, "day").unwrap_or_default();

        if !year.is_empty() {
            if !month.is_empty() && !day.is_empty() {
                format!("{}-{}-{}", year, month, day)
            } else {
                year
            }
        } else {
            String::new()
        }
    }

    /// Extract attribute value from an element
    fn extract_attribute(&self, xml: &str, tag: &str, attr: &str, value: &str) -> Option<String> {
        let pattern = format!(r#"<{}[^>]*{}\s*=\s*["']([^"']*{})[^"']*["'][^>]*>([^<]*)</{}>"#, tag, attr, value, tag);
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(caps) = re.captures(xml) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }
}

impl Default for PmcSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Source for PmcSource {
    fn id(&self) -> &str {
        "pmc"
    }

    fn name(&self) -> &str {
        "PubMed Central"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD | SourceCapabilities::READ
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        let mut url = format!(
            "{}/esearch.fcgi?db=pmc&term={}&retmax={}&retmode=json",
            PMC_EUTILS_BASE,
            urlencoding::encode(&query.query),
            query.max_results
        );

        // Add year filter if specified
        if let Some(year) = &query.year {
            if year.contains('-') {
                // Year range - for simplicity, just use the first year
                let start_year = year.split('-').next().unwrap_or(year);
                url = format!(
                    "{}&datetype=pubmed&mindate={}/01/01&maxdate={}/12/31",
                    url, start_year, start_year
                );
            } else {
                url = format!(
                    "{}&datetype=pubmed&mindate={}/01/01&maxdate={}/12/31",
                    url, year, year
                );
            }
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to search PMC: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::Api(format!(
                "PMC API returned status: {}",
                response.status()
            )));
        }

        let data: ESearchResponse = response
            .json()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to parse JSON: {}", e)))?;

        let pmcids = data.esearchresult.idlist;

        if pmcids.is_empty() {
            return Ok(SearchResponse::new(vec![], "PMC", &query.query));
        }

        // Fetch details for each paper
        let mut papers = Vec::new();
        for pmcid in pmcids.iter().take(query.max_results) {
            match self.fetch_paper_details(pmcid).await {
                Ok(Some(paper)) => papers.push(paper),
                Ok(None) => {}
                Err(e) => {
                    // Log error but continue with other papers
                    eprintln!("Error fetching paper {}: {}", pmcid, e);
                }
            }
        }

        Ok(SearchResponse::new(papers, "PMC", &query.query))
    }

    async fn download(&self, request: &DownloadRequest) -> Result<DownloadResult, SourceError> {
        let pmcid = self.clean_pmcid(&request.paper_id);
        let full_pmcid = format!("PMC{}", pmcid);

        let pdf_url = format!("{}/articles/{}/pdf/", PMC_BASE_URL, full_pmcid);

        let response = self
            .client
            .get(&pdf_url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to download PDF: {}", e)))?;

        if !response.status().is_success() {
            return Err(SourceError::NotFound(format!(
                "Paper not found: {}",
                full_pmcid
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to read PDF: {}", e)))?;

        std::fs::create_dir_all(&request.save_path).map_err(|e| {
            SourceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory: {}", e),
            ))
        })?;

        let filename = format!("{}.pdf", full_pmcid);
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
}

impl PmcSource {
    async fn fetch_paper_details(&self, pmcid: &str) -> Result<Option<Paper>, SourceError> {
        let url = format!(
            "{}/efetch.fcgi?db=pmc&id={}&retmode=xml",
            PMC_EUTILS_BASE, pmcid
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SourceError::Network(format!("Failed to fetch paper: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let xml_content = response
            .text()
            .await
            .map_err(|e| SourceError::Parse(format!("Failed to read XML: {}", e)))?;

        Ok(Some(self.parse_pmc_xml(&xml_content, pmcid)?))
    }
}

// ===== PMC API Types =====

#[derive(Debug, Deserialize)]
struct ESearchResponse {
    esearchresult: ESearchResult,
}

#[derive(Debug, Deserialize)]
struct ESearchResult {
    idlist: Vec<String>,
}
