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

| Source | Search | Download | Read* | Citations | DOI Lookup | Author Search |
|--------|--------|----------|-------|-----------|------------|---------------|
| [arXiv](https://arxiv.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Semantic Scholar](https://semanticscholar.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [OpenAlex](https://openalex.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [PubMed](https://pubmed.ncbi.nlm.nih.gov) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [PMC](https://www.ncbi.nlm.nih.gov/pmc) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [bioRxiv](https://biorxiv.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [HAL](https://hal.science) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [DBLP](https://dblp.org) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [CrossRef](https://www.crossref.org) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| [IACR ePrint](https://eprint.iacr.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [SSRN](https://www.ssrn.com) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |

*PDF text extraction requires poppler/libpoppler to be installed on your system.

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
- **PDF text extraction** for reading paper contents (requires poppler)

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

### Intelligent Source Management

- **Auto-disable sources** that fail to initialize (e.g., missing API keys)
- **Enable specific sources** via environment variable
- Graceful degradation when some sources are unavailable

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

### Compile-Time Feature Flags

Individual research sources can be disabled at compile time using Cargo features. By default, all 11 sources are included.

#### Available Features

| Feature | Description |
|---------|-------------|
| `arxiv` | Enable arXiv source |
| `pubmed` | Enable PubMed source |
| `biorxiv` | Enable bioRxiv source |
| `semantic` | Enable Semantic Scholar source |
| `openalex` | Enable OpenAlex source |
| `crossref` | Enable CrossRef source |
| `iacr` | Enable IACR ePrint source |
| `pmc` | Enable PMC source |
| `hal` | Enable HAL source |
| `dblp` | Enable DBLP source |
| `ssrn` | Enable SSRN source |

#### Feature Groups

| Group | Description |
|-------|-------------|
| `core` | arxiv, pubmed, semantic |
| `preprints` | arxiv, biorxiv |
| `full` | All sources (default) |

#### Build Examples

```bash
# Build with all sources (default)
cargo build --release

# Build with only core sources (smaller binary)
cargo build --release --no-default-features --features core

# Build with specific sources
cargo build --release --no-default-features --features arxiv,semantic,openalex

# Build with all sources except dblp
cargo build --release --features -dblp

# Install from source with only core sources
cargo install --path . --no-default-features --features core
```

**Note:** Feature flags affect which sources are **compiled into the binary**. This is different from the `RESEARCH_MASTER_ENABLED_SOURCES` environment variable, which filters sources at **runtime**. You can use both together for maximum control.

### Dependencies for PDF Extraction

PDF text extraction requires native poppler libraries:

**macOS:**
```bash
brew install poppler
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libpoppler-cpp-dev
```

**Arch Linux:**
```bash
sudo pacman -S poppler
```

**Windows:**
Download and install from [poppler-windows](https://github.com/oschwartz10612/poppler-windows)

## Configuration

All configuration can be set via environment variables using the `RESEARCH_MASTER_` prefix.

### Environment Variables

#### Source Filtering

| Variable | Description | Default |
|----------|-------------|---------|
| `RESEARCH_MASTER_ENABLED_SOURCES` | Comma-separated list of sources to enable | (all enabled) |

**Example:**
```bash
# Only enable arXiv, PubMed, and Semantic Scholar
export RESEARCH_MASTER_ENABLED_SOURCES="arxiv,pubmed,semantic"

# All sources enabled (default)
# unset RESEARCH_MASTER_ENABLED_SOURCES
```

**Available source IDs:** `arxiv`, `pubmed`, `biorxiv`, `semantic`, `openalex`, `crossref`, `iacr`, `pmc`, `hal`, `dblp`, `ssrn`

#### API Keys (Optional)

| Variable | Description |
|----------|-------------|
| `SEMANTIC_SCHOLAR_API_KEY` | API key for Semantic Scholar (higher rate limits) |
| `OPENALEX_EMAIL` | Email for OpenAlex "polite pool" access |

**Note:** Sources work without API keys but may have lower rate limits. If a source requires an API key that isn't provided, it will be automatically disabled during initialization.

#### Source-Specific Rate Limits

| Variable | Description | Default |
|----------|-------------|---------|
| `SEMANTIC_SCHOLAR_RATE_LIMIT` | Semantic Scholar requests per second | `1` |

**Note:** Without an API key, Semantic Scholar limits you to 1 request per second. Set to a higher value if you have an API key.

#### Global Rate Limiting

| Variable | Description | Default |
|----------|-------------|---------|
| `RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND` | Global requests per second for all HTTP requests | `5` |

**Disable rate limiting entirely:**
```bash
export RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND=0
```

#### Download Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `RESEARCH_MASTER_DOWNLOADS_DEFAULT_PATH` | Default directory for PDF downloads | `./downloads` |
| `RESEARCH_MASTER_DOWNLOADS_ORGANIZE_BY_SOURCE` | Create subdirectories per source | `true` |
| `RESEARCH_MASTER_DOWNLOADS_MAX_FILE_SIZE_MB` | Maximum file size for downloads (MB) | `100` |

#### Logging

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Logging level (e.g., `debug`, `info`, `warn`, `error`) |

### Configuration File (Alternative)

You can also create a `research-master.toml` file:

```toml
[downloads]
default_path = "./downloads"
organize_by_source = true
max_file_size_mb = 100

[rate_limits]
default_requests_per_second = 5

[sources.semantic_scholar]
api_key = "your-key-here"

[sources.openalex]
email = "your@email.com"
```

## Usage

### MCP Server Mode (stdio)

For integration with Claude Desktop or other MCP clients:

```bash
research-master-mcp serve
```

### CLI Usage

```bash
# Search for papers
research-master-mcp search "transformer architecture" --year 2020-

# Search by author
research-master-mcp author "Geoffrey Hinton" --source semantic

# Download a paper
research-master-mcp download 2301.12345 --source arxiv --output ./papers

# Look up by DOI
research-master-mcp lookup 10.48550/arXiv.2301.12345

# Get citations
research-master-mcp citations 2301.12345 --source arxiv

# Deduplicate papers
research-master-mcp dedupe papers.json --strategy first

# Show all environment variables
research-master-mcp --env
```

### Command Options

#### Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose logging (can be repeated) |
| `-q, --quiet` | Suppress non-error output |
| `-o, --output` | Output format: `auto`, `table`, `json`, `plain` |
| `--config` | Path to configuration file |
| `--timeout` | Request timeout in seconds (default: 30) |
| `--env` | Show all environment variables and exit |

#### Search Command

```bash
research-master-mcp search <QUERY> [OPTIONS]

Options:
  -s, --source <SOURCE>    Source to search (default: all)
  -m, --max-results <N>    Maximum results (default: 10)
  -y, --year <YEAR>        Year filter (e.g., "2020", "2018-2022", "2010-", "-2015")
  --sort-by <FIELD>        Sort by: relevance, date, citations, title, author
  --order <ORDER>          Sort order: asc, desc
  -c, --category <CAT>     Category/subject filter
  --dedup                   Deduplicate results
  --dedup-strategy <STRAT>  Deduplication strategy: first, last, mark
```

#### Author Command

```bash
research-master-mcp author <AUTHOR> [OPTIONS]

Options:
  -s, --source <SOURCE>  Source to search (default: all with author search)
  -m, --max-results <N>  Maximum results per source (default: 10)
  -y, --year <YEAR>      Year filter
  --dedup                 Deduplicate results
  --dedup-strategy <STRAT>  Deduplication strategy
```

#### Download Command

```bash
research-master-mcp download <PAPER_ID> --source <SOURCE> [OPTIONS]

Options:
  -s, --source <SOURCE>  Paper source (required)
  -o, --output <PATH>    Save path
  --auto-filename        Auto-generate filename from title
  --create-dir           Create parent directory if needed
  --doi <DOI>            Paper DOI (optional, for verification)
```

#### Serve Command

```bash
research-master-mcp serve [OPTIONS]

Options:
  --stdio   Run in stdio mode (for MCP clients like Claude Desktop)
  -p, --port <PORT>  Port for HTTP/SSE mode (default: 3000)
  --host <HOST>      Host to bind to (default: 127.0.0.1)
```

#### Lookup Command

```bash
research-master-mcp lookup <DOI> [OPTIONS]

Options:
  -s, --source <SOURCE>  Source to search (default: all with DOI lookup)
  --json                 Output as JSON
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
      "args": ["serve"]
    }
  }
}
```

### Advanced Configuration

To enable only specific sources:

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "RESEARCH_MASTER_ENABLED_SOURCES": "arxiv,pubmed,semantic"
      }
    }
  }
}
```

To set custom rate limits:

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND": "10"
      }
    }
  }
}
```

## Available MCP Tools

Once connected via MCP, the following tools are available:

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

### Read Tools (PDF Text Extraction)

- `read_arxiv_paper` - Extract text from arXiv PDF
- `read_semantic_paper` - Extract text from Semantic Scholar PDF
- `read_openalex_paper` - Extract text from OpenAlex PDF
- `read_pubmed_paper` - Extract text from PubMed PDF
- `read_pmc_paper` - Extract text from PMC PDF
- `read_biorxiv_paper` - Extract text from bioRxiv PDF
- `read_hal_paper` - Extract text from HAL PDF
- `read_iacr_paper` - Extract text from IACR PDF

**Note:** PDF text extraction requires poppler to be installed. If extraction fails, the tool will return an error message indicating the issue.

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

User: Read the abstract and introduction from this paper
Claude: [Uses read_arxiv_paper tool to extract PDF text]
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
│   ├── registry.rs   # Source registry with filtering
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
└── utils/            # Utilities
    ├── http.rs       # HTTP client with rate limiting
    ├── dedup.rs      # Deduplication logic
    ├── pdf.rs        # PDF text extraction
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
- **Rate limiting**: governor (GCRA algorithm)
- **Serialization**: serde, serde_json
- **XML parsing**: quick-xml
- **Feed parsing**: feed-rs
- **HTML parsing**: scraper
- **Text similarity**: strsim (Jaro-Winkler for deduplication)
- **PDF extraction**: pdf-extract (requires poppler)
- **Date/time**: chrono, time
- **Error handling**: thiserror, anyhow
- **Logging**: tracing, tracing-subscriber
- **Configuration**: config
- **CLI**: clap
- **MCP Protocol**: pmcp (Model Context Protocol SDK)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Implements the [Model Context Protocol](https://modelcontextprotocol.io/)
- Integrates with numerous academic research APIs and services

## Contact

- GitHub: [@hongkongkiwi](https://github.com/hongkongkiwi)
- Repository: [https://github.com/hongkongkiwi/research-master-mcp](https://github.com/hongkongkiwi/research-master-mcp)
