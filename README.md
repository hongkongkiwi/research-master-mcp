# Research Master MCP

A Model Context Protocol (MCP) server for searching and downloading academic papers from multiple research sources.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/research-master-mcp)](https://crates.io/crates/research-master-mcp)
[![GitHub Release](https://img.shields.io/github/v/release/hongkongkiwi/research-master-mcp)](https://github.com/hongkongkiwi/research-master-mcp/releases)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Claude Desktop Integration](#claude-desktop-integration)
- [Development](#development)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Overview

Research Master MCP is a comprehensive academic research server that provides unified access to **26 major research repositories and databases**. It implements the Model Context Protocol (MCP) to integrate seamlessly with AI assistants like Claude Desktop, enabling powerful literature search, paper discovery, and citation analysis capabilities.

## Features

### Multi-Source Search

Search across **26 academic research sources** simultaneously:

| Source | Search | Download | Read* | Citations | DOI Lookup | Author Search |
|--------|--------|----------|-------|-----------|------------|---------------|
| [arXiv](https://arxiv.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [Semantic Scholar](https://semanticscholar.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [OpenAlex](https://openalex.org) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [PubMed](https://pubmed.ncbi.nlm.nih.gov) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [PMC](https://www.ncbi.nlm.nih.gov/pmc) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [bioRxiv](https://biorxiv.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [medRxiv](https://medrxiv.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [HAL](https://hal.science) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [DBLP](https://dblp.org) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [CrossRef](https://www.crossref.org) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| [IACR ePrint](https://eprint.iacr.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [SSRN](https://www.ssrn.com) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [CORE](https://core.ac.uk) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [EuropePMC](https://europepmc.org) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [Dimensions](https://app.dimensions.ai) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| [IEEE Xplore](https://ieeexplore.ieee.org) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [Zenodo](https://zenodo.org) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [Unpaywall](https://unpaywall.org) | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ |
| [MDPI](https://mdpi.com) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [JSTOR](https://www.jstor.org) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [SciSpace](https://scispace.net) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| [ACM DL](https://dl.acm.org) | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ |
| [Connected Papers](https://www.connectedpapers.com) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| [DOAJ](https://doaj.org) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [WorldWideScience](https://www.worldwidescience.org) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [OSF Preprints](https://osf.io/preprints) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [BASE](https://www.base-search.net) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| [Springer](https://link.springer.com) | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ |
| [Google Scholar](https://scholar.google.com) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |

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
- **Google Scholar** is disabled by default and requires `GOOGLE_SCHOLAR_ENABLED=true` to activate

## Quick Start

### Using Homebrew (macOS)

```bash
# Add the custom tap
brew tap hongkongkiwi/research-master-mcp

# Install research-master-mcp
brew install research-master-mcp

# Start the MCP server
research-master-mcp serve --stdio
```

### Using Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json`):

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

### Using the CLI

```bash
# Search for papers
research-master-mcp search "transformer architecture" --year 2020-

# Search by author
research-master-mcp author "Geoffrey Hinton"

# Download a paper
research-master-mcp download 2301.12345 --source arxiv

# Look up by DOI
research-master-mcp lookup 10.48550/arXiv.2301.12345
```

## Installation

### Homebrew (macOS)

```bash
# Add the custom tap
brew tap hongkongkiwi/research-master-mcp

# Install research-master-mcp
brew install research-master-mcp
```

### Linux (Debian/Ubuntu) - DEB Package

```bash
# Download .deb package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/v0.1.1/research-master-mcp_0.1.1_amd64.deb

# Install the package
sudo dpkg -i research-master-mcp_0.1.1_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

### Linux (Alpine) - APK Package

```bash
# Download .apk package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/v0.1.1/research-master-mcp-0.1.1-x86_64.apk

# Install the package
sudo apk add --allow-untrusted research-master-mcp-0.1.1-x86_64.apk
```

### Linux (RedHat/Fedora) - RPM Package

```bash
# Download .rpm package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/v0.1.1/research-master-mcp-0.1.1-1.x86_64.rpm

# Install the package
sudo dnf install research-master-mcp-0.1.1-1.x86_64.rpm
```

### Crates.io

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

Individual research sources can be disabled at compile time using Cargo features. By default, all sources are included except Google Scholar (requires `GOOGLE_SCHOLAR_ENABLED=true`).

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
| `dimensions` | Enable Dimensions source |
| `ieee_xplore` | Enable IEEE Xplore source |
| `core_repo` | Enable CORE source |
| `zenodo` | Enable Zenodo source |
| `unpaywall` | Enable Unpaywall source |
| `mdpi` | Enable MDPI source |
| `jstor` | Enable JSTOR source |
| `scispace` | Enable SciSpace source |
| `acm` | Enable ACM Digital Library source |
| `connected_papers` | Enable Connected Papers source |
| `doaj` | Enable DOAJ source |
| `worldwidescience` | Enable WorldWideScience source |
| `osf` | Enable OSF Preprints source |
| `base` | Enable BASE source |
| `springer` | Enable Springer source |
| `google_scholar` | Enable Google Scholar source (disabled by default) |

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

# Build with Google Scholar enabled (requires environment variable at runtime)
cargo build --release --features google_scholar
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

## Usage

### MCP Server Mode (stdio)

For integration with Claude Desktop or other MCP clients:

```bash
research-master-mcp serve --stdio
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

**Linux**: `~/.config/Claude/claude_desktop_config.json`

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

You can configure sources using a config file or environment variables. Environment variables always take precedence over config file settings.

**Using a config file:**

Create `~/.config/research-master/config.toml`:
```toml
[sources]
enabled_sources = "arxiv,pubmed,semantic"
disabled_sources = "dblp,jstor"
```

**Using environment variables:**

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

To enable Google Scholar (requires compile-time feature):

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "GOOGLE_SCHOLAR_ENABLED": "true"
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

Once connected via MCP, the following **unified tools** are available. These tools automatically select the best source based on your input, or you can specify a source explicitly.

### Core Research Tools

#### `search_papers`
Search for papers across all available research sources or a specific source.

**Parameters:**
- `query` (required): Search query string
- `source` (optional): Specific source to search (e.g., "arxiv", "semantic", "pubmed")
- `max_results` (optional): Maximum number of results (default: 10)
- `year` (optional): Year filter (e.g., "2020", "2018-2022", "2010-", "-2015")
- `category` (optional): Category/subject filter

**Example:**
```json
{
  "query": "transformer architecture",
  "year": "2020-",
  "max_results": 20
}
```

#### `search_by_author`
Search for papers by a specific author across sources that support author search.

**Parameters:**
- `author` (required): Author name
- `source` (optional): Specific source to search
- `max_results` (optional): Maximum results per source (default: 10)

**Supported sources:** arxiv, semantic, openalex, pubmed, biorxiv, pmc, hal, iacr, ssrn

#### `get_paper`
Get detailed metadata for a specific paper. The source is auto-detected from the paper ID format.

**Parameters:**
- `paper_id` (required): Paper identifier (e.g., "2301.12345", "arXiv:2301.12345", "PMC12345678")
- `source` (optional): Override auto-detection and use specific source

**Auto-detection patterns:**
- `arXiv:xxxx` or `xxxx.xxxx` (numeric) → arXiv
- `PMCxxxxxxx` → PMC
- `10.xxxx/xxxxxx` (DOI) → Source with DOI lookup capability
- `CORX:xxxxx` → CORE
- `hal-xxxxxxx` → HAL
- `xxxx/xxxx` (IACR format) → IACR
- `DOI:10.xxxx` → DOI-based lookup
- etc.

#### `download_paper`
Download a paper PDF to your local filesystem.

**Parameters:**
- `paper_id` (required): Paper identifier
- `source` (optional): Override auto-detection
- `output_path` (optional): Save path (default: ./downloads)
- `auto_filename` (optional): Auto-generate filename from title (default: true)

#### `read_paper`
Extract and return the full text content from a paper PDF.

**Parameters:**
- `paper_id` (required): Paper identifier
- `source` (optional): Override auto-detection

**Note:** Requires poppler to be installed. Returns an error if PDF extraction fails.

### Citation & Reference Tools

#### `get_citations`
Get papers that cite a specific paper. Prefers Semantic Scholar for best results.

**Parameters:**
- `paper_id` (required): Paper identifier
- `source` (optional): Specific source (default: "semantic")
- `max_results` (optional): Maximum results (default: 20)

#### `get_references`
Get papers referenced by a specific paper. Prefers Semantic Scholar.

**Parameters:**
- `paper_id` (required): Paper identifier
- `source` (optional): Specific source (default: "semantic")
- `max_results` (optional): Maximum results (default: 20)

### Lookup Tools

#### `lookup_by_doi`
Look up a paper by its DOI across all sources that support DOI lookup.

**Parameters:**
- `doi` (required): Digital Object Identifier (e.g., "10.48550/arXiv.2301.12345")
- `source` (optional): Specific source to query (default: all)

**Supported sources:** semantic, openalex, crossref, hal, doaj, osf, springer, mdpi, zenodo, acm, base, unpaywall

### Utility Tools

#### `deduplicate_papers`
Remove duplicate papers from a list using DOI matching and title similarity.

**Parameters:**
- `papers` (required): Array of paper objects
- `strategy` (optional): Deduplication strategy - "first" (keep first), "last" (keep last), or "mark" (add `is_duplicate` flag)

**Deduplication criteria:**
- Exact DOI match
- Title similarity > 0.95 (Jaro-Winkler algorithm)
- Author verification

### Smart Source Selection

The unified tools use intelligent source auto-detection:

| Paper ID Format | Detected Source |
|-----------------|-----------------|
| `arXiv:1234.5678` or `1234.5678` | arXiv |
| `PMC12345678` | PMC |
| `10.xxxx/xxxxxx` | Source with DOI lookup |
| `CORX:xxxxx` | CORE |
| `hal-xxxxxxx` | HAL |
| `xxxx/xxxx` (IACR format) | IACR |
| `doi:10.xxxx` or `https://doi.org/10.xxxx` | DOI lookup |
| `ZENODO:xxxxx` | Zenodo |
| etc. |

You can always override auto-detection by specifying the `source` parameter explicitly.

## Example Usage with Claude

Once configured with Claude Desktop, you can interact with the research sources naturally using the unified tools:

```
User: Search for papers about "transformer architecture" from 2020 onwards
Claude: [Uses search_papers tool with query="transformer architecture", year="2020-"]

User: Find papers by Geoffrey Hinton on deep learning
Claude: [Uses search_by_author tool with author="Geoffrey Hinton"]

User: Download the paper "Attention Is All You Need" and find what papers cite it
Claude: [Uses download_paper with paper_id="1706.03762" (arXiv),
         then get_citations to find citing papers]

User: Read the abstract and introduction from this paper
Claude: [Uses read_paper tool with paper_id="1706.03762" to extract PDF text]

User: Look up this paper by its DOI
Claude: [Uses lookup_by_doi with doi="10.48550/arXiv.1706.03762"]
```

The unified tools automatically detect the appropriate source, so you don't need to remember which source has which paper. You can simply provide the paper ID, DOI, or search query, and the tool handles the rest.

## Development

### Quick Commands (using just)

This project uses a `justfile` for common development tasks:

```bash
# Build
just build              # Debug build
just build-release      # Release build
just build-features     # Build with specific features

# Development
just dev                # Start MCP server in stdio mode
just watch              # Watch and rebuild (requires cargo-watch)

# Testing
just test               # Run all tests
just test-verbose       # Run tests with output
just test-name <name>   # Run specific test

# Code Quality
just fmt                # Format code
just fmt-check          # Check formatting
just clippy             # Run clippy lints
just clippy-fix         # Auto-fix clippy issues
just doc                # Generate documentation
just check              # Run all checks (fmt, clippy, doc, test)

# Security
just audit              # Check for vulnerabilities

# Release
just version            # Show current version
just release patch      # Create patch release
just release minor      # Create minor release
just release major      # Create major release

# Cleanup
just clean              # Clean build artifacts
just clean-all          # Clean everything

# Show all commands
just help
```

### Project Structure

```
research-master-mcp/
├── Cargo.toml
├── justfile                     # Development commands
├── README.md
├── LICENSE
├── src/
│   ├── main.rs                  # CLI entry point
│   ├── lib.rs                   # Library exports
│   ├── mcp/                     # MCP protocol implementation
│   │   ├── server.rs            # MCP server (stdio/SSE)
│   │   ├── tools.rs             # Tool registry
│   │   ├── unified_tools.rs     # Unified tool handlers with smart source selection
│   │   └── mod.rs
│   ├── sources/                 # Research source implementations
│   │   ├── mod.rs               # Source trait definition
│   │   ├── registry.rs          # Source registry with filtering
│   │   ├── arxiv.rs             # arXiv
│   │   ├── pubmed.rs            # PubMed
│   │   ├── biorxiv.rs           # bioRxiv/medRxiv
│   │   ├── semantic.rs          # Semantic Scholar
│   │   ├── openalex.rs          # OpenAlex
│   │   ├── crossref.rs          # CrossRef
│   │   ├── iacr.rs              # IACR
│   │   ├── pmc.rs               # PMC
│   │   ├── hal.rs               # HAL
│   │   ├── dblp.rs              # DBLP
│   │   ├── ssrn.rs              # SSRN
│   │   ├── core.rs              # CORE
│   │   ├── europepmc.rs         # EuropePMC
│   │   ├── dimensions.rs        # Dimensions
│   │   ├── ieee_xplore.rs       # IEEE Xplore
│   │   ├── zenodo.rs            # Zenodo
│   │   ├── unpaywall.rs         # Unpaywall
│   │   ├── mdpi.rs              # MDPI
│   │   ├── jstor.rs             # JSTOR
│   │   ├── scispace.rs          # SciSpace
│   │   ├── acm.rs               # ACM Digital Library
│   │   ├── connected_papers.rs  # Connected Papers
│   │   ├── doaj.rs              # DOAJ
│   │   ├── worldwidescience.rs  # WorldWideScience
│   │   ├── osf.rs               # OSF Preprints
│   │   ├── base.rs              # BASE
│   │   ├── springer.rs          # Springer
│   │   └── google_scholar.rs    # Google Scholar
│   ├── models/                  # Data models
│   │   ├── paper.rs             # Paper model
│   │   ├── search.rs            # Search request/response
│   │   └── mod.rs
│   └── utils/                   # Utilities
│       ├── http.rs              # HTTP client with rate limiting
│       ├── dedup.rs             # Deduplication logic
│       ├── pdf.rs               # PDF text extraction
│       └── mod.rs
└── .github/
    └── workflows/
        ├── ci.yml               # CI pipeline
        ├── release.yml          # Release automation
        └── update-homebrew.yml  # Homebrew formula update
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

2. Add a feature flag in `Cargo.toml`:

```toml
[features]
default = ["source-mysource"]
source-mysource = []
```

3. Conditionally compile the module in `src/sources/mod.rs`:

```rust
#[cfg(feature = "source-mysource")]
mod mysource;
```

4. Register the source in `src/sources/registry.rs`:

```rust
#[cfg(feature = "source-mysource")]
use super::mysource::MySource;

// In try_new():
#[cfg(feature = "source-mysource")]
try_register!(MySource::new());
```

5. Add the SourceType variant in `src/models/paper.rs`:

```rust
pub enum SourceType {
    // ... existing variants
    MySource,
    // ...
}
```

6. Rebuild - the unified MCP tools will automatically include your new source

### Building and Testing

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets -- -D warnings

# Generate documentation
cargo doc --no-deps --all-features
```

## Configuration

Research Master MCP supports configuration via environment variables and/or a configuration file. Environment variables take precedence over file-based settings.

### Configuration File

The application searches for a configuration file in the following locations (in order):

| Priority | Location | Platform |
|----------|----------|----------|
| 1 | `./research-master.toml` | All |
| 2 | `./.research-master.toml` | All |
| 3 | `$XDG_CONFIG_HOME/research-master/config.toml` | Linux/BSD (if XDG_CONFIG_HOME set) |
| 4 | `~/Library/Application Support/research-master/config.toml` | macOS |
| 5 | `~/.config/research-master/config.toml` | Linux/BSD |
| 6 | `%APPDATA%\research-master\config.toml` | Windows |

You can also specify a custom config file path using the `--config` CLI option:

```bash
research-master-mcp --config /path/to/config.toml search "machine learning"
```

#### Configuration File Format (TOML)

```toml
# Download settings
[downloads]
default_path = "./downloads"
organize_by_source = true
max_file_size_mb = 100

# Rate limiting settings
[rate_limits]
default_requests_per_second = 5
max_concurrent_requests = 10

# API Keys
[api_keys]
semantic_scholar = "your-semantic-scholar-api-key"
core = "your-core-api-key"
openalex_email = "your@email.com"

# Source filtering (same as environment variables)
[sources]
enabled_sources = "arxiv,pubmed,semantic"
disabled_sources = "dblp,jstor"
```

### Environment Variables

All settings can be overridden using environment variables with the `RESEARCH_MASTER_` prefix.

#### Source Filtering

| Variable | Description | Default |
|----------|-------------|---------|
| `RESEARCH_MASTER_ENABLED_SOURCES` | Comma-separated list of sources to enable | (all enabled) |
| `RESEARCH_MASTER_DISABLED_SOURCES` | Comma-separated list of sources to disable | (none disabled) |

**Logic:**
- If only `ENABLED` is set: only those sources are enabled
- If only `DISABLED` is set: all sources except those are enabled
- If both are set: enabled sources **minus** disabled sources (authoritative list)
- If neither is set: all sources enabled

**Example:**
```bash
# Only enable arXiv, PubMed, and Semantic Scholar
export RESEARCH_MASTER_ENABLED_SOURCES="arxiv,pubmed,semantic"

# Enable all sources except DBLP and JSTOR
export RESEARCH_MASTER_DISABLED_SOURCES="dblp,jstor"

# Enable only arxiv and semantic, but disable semantic (result: only arxiv)
export RESEARCH_MASTER_ENABLED_SOURCES="arxiv,semantic"
export RESEARCH_MASTER_DISABLED_SOURCES="semantic"
```

**Available source IDs:**
`arxiv`, `pubmed`, `biorxiv`, `semantic`, `openalex`, `crossref`, `iacr`, `pmc`, `hal`, `dblp`, `ssrn`, `core`, `europe_pmc`, `dimensions`, `ieee_xplore`, `zenodo`, `unpaywall`, `mdpi`, `jstor`, `scispace`, `acm`, `connected_papers`, `doaj`, `worldwidescience`, `osf`, `base`, `springer`, `google_scholar`

#### API Keys (Optional)

| Variable | Description |
|----------|-------------|
| `SEMANTIC_SCHOLAR_API_KEY` | API key for Semantic Scholar (higher rate limits) |
| `CORE_API_KEY` | API key for CORE service |
| `OPENALEX_EMAIL` | Email for OpenAlex "polite pool" access |
| `IEEEXPLORE_API_KEY` | API key for IEEE Xplore |
| `JSTOR_API_KEY` | API key for JSTOR |
| `ACM_API_KEY` | API key for ACM Digital Library |
| `SPRINGER_API_KEY` | API key for Springer |

**Note:** Sources work without API keys but may have lower rate limits. If a source requires an API key that isn't provided, it will be automatically disabled during initialization.

#### Source-Specific Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_SCHOLAR_ENABLED` | Enable Google Scholar (requires compile-time feature) | `false` |

**Note:** Google Scholar is disabled by default both at compile-time and runtime. To enable it:
1. Build with `--features google_scholar`
2. Set `GOOGLE_SCHOLAR_ENABLED=true` at runtime

#### Source-Specific Rate Limits

| Variable | Description | Default |
|----------|-------------|---------|
| `SEMANTIC_SCHOLAR_RATE_LIMIT` | Semantic Scholar requests per second | `1` |
| `IEEEXPLORE_RATE_LIMIT` | IEEE Xplore requests per second | `3` |
| `ACMRATE_LIMIT` | ACM Digital Library requests per second | `3` |

**Note:** Without an API key, Semantic Scholar limits you to 1 request per second. Set to a higher value if you have an API key.

#### Global Rate Limiting

| Variable | Description | Default |
|----------|-------------|---------|
| `RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND` | Global requests per second for all HTTP requests | `5` |
| `RESEARCH_MASTER_RATE_LIMITS_MAX_CONCURRENT_REQUESTS` | Maximum concurrent requests | `10` |

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

### Configuration Precedence

Configuration values are loaded in this order (later values override earlier ones):

1. Configuration file in default search locations
2. Configuration file specified via `--config` option
3. Environment variables (always take final precedence)

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Ways to Contribute

- Report bugs and issues
- Suggest new features
- Add new research sources
- Improve documentation
- Submit pull requests

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Implements the [Model Context Protocol](https://modelcontextprotocol.io/)
- Integrates with numerous academic research APIs and services

## Contact

- GitHub: [@hongkongkiwi](https://github.com/hongkongkiwi)
- Repository: [https://github.com/hongkongkiwi/research-master-mcp](https://github.com/hongkongkiwi/research-master-mcp)
