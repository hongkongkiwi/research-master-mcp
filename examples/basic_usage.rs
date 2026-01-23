//! Basic usage example for Research Master MCP library.
//!
//! This example demonstrates how to use the library to search for papers
//! across multiple research sources.

use research_master_mcp::models::{SearchQuery, SortBy};
use research_master_mcp::sources::SourceRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the source registry with all available sources
    let registry = Arc::new(SourceRegistry::new());

    println!("Initialized {} research sources", registry.len());
    println!("Available sources: {}\n", registry.ids().collect::<Vec<_>>().join(", "));

    // Create a search query
    let query = SearchQuery::new("machine learning transformers")
        .max_results(5)
        .year("2020-")
        .sort_by(SortBy::Date);

    // Search across all available sources
    let mut all_papers = Vec::new();

    for source in registry.searchable() {
        println!("Searching {}...", source.name());

        match source.search(&query).await {
            Ok(response) => {
                println!("  Found {} papers", response.papers.len());
                all_papers.extend(response.papers);
            }
            Err(e) => {
                eprintln!("  Error: {}", e);
            }
        }
    }

    println!("\nTotal papers found: {}", all_papers.len());

    // Print the first few papers
    for (i, paper) in all_papers.iter().take(3).enumerate() {
        println!("\n{}. {}", i + 1, paper.title);
        println!("   Authors: {}", paper.authors);
        if let Some(year) = &paper.published_date {
            println!("   Year: {}", year);
        }
        if let Some(doi) = &paper.doi {
            println!("   DOI: {}", doi);
        }
        println!("   Source: {}", paper.source.name());
        println!("   URL: {}", paper.url);
    }

    Ok(())
}
