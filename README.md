# Research Master MCP

A Model Context Protocol (MCP) server for searching and downloading academic papers from multiple research sources.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/research-master-mcp)](https://crates.io/crates/research-master-mcp)

## Overview

Research Master MCP is a comprehensive academic research server that provides unified access to 11 major research repositories and databases. It implements the Model Context Protocol (MCP) to integrate seamlessly with AI assistants like Claude Desktop, enabling powerful literature search, paper discovery, and citation analysis capabilities.

## Features

### Multi-Source Search

Search across **11 academic research sources** simultaneously:

| Source | Search | Download | Citations | DOI Lookup | Author Search |
|--------|--------|----------|-----------|------------|---------------|
| [arXiv](https://arxiv.org) | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Semantic Scholar](https://semanticscholar.org) | ✅ | ✅ | ✅ | ✅ | ✅ |
| [OpenAlex](https://openalex.org) | ✅ | ✅ | ✅ | ✅ | ✅ |
| [PubMed](https://pubmed.ncbi.nlm.nih.gov) | ✅ | ✅ | ❌ | ❌ | ❌ |
| [PMC](https://www.ncbi.nlm.nih.gov/pmc) | ✅ | ✅ | ❌ | ❌ | ❌ |
| [bioRxiv](https://biorxiv.org) | ✅ | ✅ | ❌ | ❌ | ❌ |
| [HAL](https://hal.science) | ✅ | ✅ | ✅ | ✅ | ❌ |
| [DBLP](https://dblp.org) | ✅ | ❌ | ❌ | ❌ | ❌ |
| [CrossRef](https://www.crossref.org) | ❌ | ❌ | ❌ | ✅ | ❌ |
| [IACR ePrint](https://eprint.iacr.org) | ✅ | ✅ | ❌ | ❌ | ❌ |
| [SSRN](https://www.ssrn.com) | ✅ | ✅ | ❌ | ❌ | ❌ |

### Advanced Search Capabilities

- **Keyword search** with relevance ranking
- **Year range filtering**: `"2020"`, `"2018-2022"`, `"2010-"`, `"-2015"`
- **Category/subject filtering** for categorized sources
- **Author-specific search** (supported sources)
- **DOI-based lookup** across multiple sources

### PDF Management

- Download papers directly to your local filesystem
- Automatic directory creation
- Configurable download paths
- Optional organization by source

### Citation Analysis

- **Forward citations**: Find papers that cite a given paper
- **References**: Discover papers referenced by a given paper
- **Related papers**: Explore similar research
- **Citation count tracking**

### Deduplication

Intelligent duplicate detection across sources:
- DOI matching (exact)
- Title similarity (Jaro-Winkler algorithm, 0.95+ threshold)
- Author verification
- Multiple strategies: keep first, keep last, or mark duplicates

## Installation

### From Crates.io

```bash
cargo install research-master-mcp
```

### From Source

```bash
git clone https://github.com/hongkongkiwi/research-master-mcp
cd research-master-mcp
cargo install --path .
```

## Configuration

### Environment Variables (Optional)

```bash
# API Keys for higher rate limits (optional)
export SEMANTIC_SCHOLAR_API_KEY="your-key-here"
export CORE_API_KEY="your-key-here"
export OPENALEX_EMAIL="your@email.com"  # For polite pool access

# Download settings
export RESEARCH_MASTER_DOWNLOADS_DEFAULT_PATH="./downloads"
export RESEARCH_MASTER_DOWNLOADS_ORGANIZE_BY_SOURCE=true
export RESEARCH_MASTER_DOWNLOADS_MAX_FILE_SIZE_MB=100

# Rate limiting
export RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND=5.0
export RESEARCH_MASTER_RATE_LIMITS_MAX_CONCURRENT_REQUESTS=10
```

### Configuration File (Optional)

Create a `research-master.toml` file:

```toml
[downloads]
default_path = "./downloads"
organize_by_source = true
max_file_size_mb = 100

[rate_limits]
default_requests_per_second = 5.0
max_concurrent_requests = 10

[sources.semantic_scholar]
api_key = "your-key-here"

[sources.openalex]
email = "your@email.com"
```

## Usage

### Standalone (stdio mode)

For integration with Claude Desktop or other MCP clients:

```bash
research-master-mcp --stdio
```

### SSE Mode (Web)

For web-based connections:

```bash
research-master-mcp --port 3000
```

### Verbose Logging

```bash
research-master-mcp --stdio --verbose
```

## Claude Desktop Integration

Add to your Claude Desktop MCP configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["--stdio"]
    }
  }
}
```

## Available Tools

Once connected, the MCP server exposes the following tools:

### Search Tools

- `search_arxiv` - Search arXiv preprints
- `search_semantic` - Search Semantic Scholar
- `search_openalex` - Search OpenAlex
- `search_pubmed` - Search PubMed
- `search_pmc` - Search PubMed Central
- `search_biorxiv` - Search bioRxiv
- `search_hal` - Search HAL archive
- `search_dblp` - Search DBLP
- `search_iacr` - Search IACR ePrint
- `search_ssrn` - Search SSRN

### Download Tools

- `download_arxiv` - Download from arXiv
- `download_semantic` - Download from Semantic Scholar
- `download_openalex` - Download from OpenAlex
- `download_pubmed` - Download from PubMed
- `download_pmc` - Download from PMC
- `download_biorxiv` - Download from bioRxiv
- `download_hal` - Download from HAL
- `download_iacr` - Download from IACR

### Citation Tools

- `get_semantic_citations` - Get papers that cite a paper
- `get_semantic_references` - Get references from a paper
- `get_semantic_related` - Get related papers
- `get_arxiv_citations` - Get citations from arXiv
- `get_openalex_citations` - Get citations from OpenAlex
- `get_hal_citations` - Get citations from HAL

### Lookup Tools

- `get_semantic_by_doi` - Lookup paper by DOI (Semantic Scholar)
- `get_openalex_by_doi` - Lookup paper by DOI (OpenAlex)
- `get_crossref_by_doi` - Lookup paper by DOI (CrossRef)
- `get_hal_by_doi` - Lookup paper by DOI (HAL)

### Author Search Tools

- `search_semantic_by_author` - Search papers by author
- `search_arxiv_by_author` - Search arXiv by author
- `search_openalex_by_author` - Search OpenAlex by author

### Utility Tools

- `deduplicate_papers` - Remove duplicate papers from a list

## Example Usage with Claude

Once configured with Claude Desktop, you can interact with the research sources naturally:

```
User: Search for papers about "transformer architecture" from 2020 onwards
Claude: [Uses search_semantic tool]

User: Find papers by Geoffrey Hinton on deep learning
Claude: [Uses search_semantic_by_author tool]

User: Download the paper "Attention Is All You Need" and find what papers cite it
Claude: [Uses download_arxiv and get_semantic_citations tools]
```

## Development

### Project Structure

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── mcp/              # MCP protocol implementation
│   ├── server.rs     # MCP server (stdio/SSE)
│   ├── tools.rs      # Tool registry and handlers
│   └── mod.rs
├── sources/          # Research source implementations
│   ├── mod.rs        # Source trait definition
│   ├── registry.rs   # Source registry
│   ├── arxiv.rs      # arXiv
│   ├── dblp.rs       # DBLP
│   ├── biorxiv.rs    # bioRxiv/medRxiv
│   ├── hal.rs        # HAL
│   ├── iacr.rs       # IACR
│   ├── semantic.rs   # Semantic Scholar
│   ├── openalex.rs   # OpenAlex
│   ├── crossref.rs   # CrossRef
│   ├── pubmed.rs     # PubMed
│   ├── pmc.rs        # PMC
│   └── ssrn.rs       # SSRN
├── models/           # Data models
│   ├── paper.rs      # Paper model
│   ├── search.rs     # Search request/response
│   └── mod.rs
├── utils/            # Utilities
│   ├── http.rs       # HTTP client
│   ├── dedup.rs      # Deduplication logic
│   └── mod.rs
└── config/           # Configuration management
    └── mod.rs
```

### Adding a New Source

1. Create a new file in `src/sources/` implementing the `Source` trait:

```rust
use crate::sources::Source;
use async_trait::async_trait;

#[derive(Debug)]
pub struct MySource;

#[async_trait]
impl Source for MySource {
    fn id(&self) -> &str {
        "mysource"
    }

    fn name(&self) -> &str {
        "My Research Source"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        // Implementation here
    }
}
```

2. Register the source in `src/sources/registry.rs`

3. Rebuild and the MCP tools will be auto-generated

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

## Dependencies

- **Async runtime**: tokio, async-trait
- **HTTP client**: reqwest with JSON and SOCKS support
- **Serialization**: serde, serde_json
- **XML parsing**: quick-xml
- **Feed parsing**: feed-rs
- **HTML parsing**: scraper
- **Text similarity**: strsim (Jaro-Winkler for deduplication)
- **Date/time**: chrono, time
- **Error handling**: thiserror, anyhow
- **Logging**: tracing, tracing-subscriber
- **Configuration**: config
- **CLI**: clap

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Implements the [Model Context Protocol](https://modelcontextprotocol.io/)
- Integrates with numerous academic research APIs and services

## Contact

- GitHub: [@hongkongkiwi](https://github.com/hongkongkiwi)
- Repository: [https://github.com/hongkongkiwi/research-master-mcp](https://github.com/hongkongkiwi/research-master-mcp)
