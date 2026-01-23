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
- [MCP Client Integration](#mcp-client-integration)
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
| [EuropePMC](https://europepmc.org) | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |
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

## Source Pricing and API Requirements

Understanding which sources are free, which require API keys, and their access limitations is essential for effective research. Below is a comprehensive breakdown of all 26 supported research sources.

### Source Comparison Table

| Source | Pricing | API Key Required | Free Tier | Rate Limits (no key) | Notes |
|--------|---------|------------------|-----------|----------------------|-------|
| [arXiv](https://arxiv.org) | Free | No | Unlimited | 1 request/sec | Open access preprints |
| [Semantic Scholar](https://semanticscholar.org) | Freemium | Optional | Yes | 1 request/sec | 100 papers/day free; API key increases limits |
| [OpenAlex](https://openalex.org) | Free | Optional | Unlimited | 10 requests/sec | Email recommended for polite pool |
| [PubMed](https://pubmed.ncbi.nlm.nih.gov) | Free | No | Unlimited | 3 requests/sec | NIH funded, no API key needed |
| [PMC](https://www.ncbi.nlm.nih.gov/pmc) | Free | No | Unlimited | 3 requests/sec | Full-text biomedical literature |
| [bioRxiv](https://biorxiv.org) | Free | No | Unlimited | 1 request/sec | Biology preprints |
| [HAL](https://hal.science) | Free | No | Unlimited | 1 request/sec | French open access repository |
| [DBLP](https://dblp.org) | Free | No | Unlimited | 1 request/sec | Computer science bibliography only |
| [CrossRef](https://www.crossref.org) | Free | No | Unlimited | 1 request/sec | DOI lookup only, no full search |
| [IACR ePrint](https://eprint.iacr.org) | Free | No | Unlimited | 1 request/sec | Cryptology research only |
| [SSRN](https://www.ssrn.com) | Free | No | Limited | 1 request/sec | Social sciences preprints |
| [CORE](https://core.ac.uk) | Freemium | Optional | Yes | 1 request/sec | Aggregates open access papers |
| [EuropePMC](https://europepmc.org) | Free | No | Unlimited | 1 request/sec | Biomedical literature, indexes PubMed/PMC |
| [Dimensions](https://app.dimensions.ai) | Free | No | Unlimited | 1 request/sec | Research metrics and discovery |
| [IEEE Xplore](https://ieeexplore.ieee.org) | Paid | Required* | Yes | 3 requests/sec | Free API key from developer.ieee.org |
| [Zenodo](https://zenodo.org) | Free | No | Unlimited | 1 request/sec | CERN-hosted open science repository |
| [Unpaywall](https://unpaywall.org) | Free | No | Unlimited | 1 request/sec | Find open access versions of papers |
| [MDPI](https://mdpi.com) | Free | No | Unlimited | 1 request/sec | Open access publisher |
| [JSTOR](https://www.jstor.org) | Paid | Required* | Limited | 1 request/sec | Historical journals, requires institutional access |
| [SciSpace](https://scispace.net) | Freemium | Optional | Yes | 1 request/sec | Research discovery platform |
| [ACM DL](https://dl.acm.org) | Paid | Required* | Limited | 3 requests/sec | Computer science, free API key available |
| [Connected Papers](https://www.connectedpapers.com) | Free | No | Unlimited | 1 request/sec | Related paper discovery |
| [DOAJ](https://doaj.org) | Free | No | Unlimited | 1 request/sec | Quality open access journals |
| [WorldWideScience](https://www.worldwidescience.org) | Free | No | Unlimited | 1 request/sec | Global science gateway |
| [OSF Preprints](https://osf.io/preprints) | Free | No | Unlimited | 1 request/sec | Open Science Framework |
| [BASE](https://www.base-search.net) | Free | No | Unlimited | 1 request/sec | Bielefeld Academic Search Engine |
| [Springer](https://link.springer.com) | Paid | Required* | Limited | 1 request/sec | Academic publisher, some open access |
| [Google Scholar](https://scholar.google.com) | Free | No | Limited | 1 request/sec | Disabled by default, scraping-based |

*Requires API key for full access. Free API keys are available but have rate limits.

### Detailed Source Information

#### Completely Free Sources (No API Key Needed)

The following sources are completely free to use without any API key requirements. They are funded by academic institutions, governments, or operate as open access repositories.

**arXiv** is the premier open access preprint repository for physics, mathematics, computer science, and related fields. Operated by Cornell University, it provides unlimited access to millions of preprints without any API key or registration requirements.

**PubMed** and **PMC** are NIH-funded databases providing free access to biomedical and life sciences literature. PubMed contains abstracts and citations, while PMC provides full-text articles. Both are completely free and require no API key.

**OpenAlex** is an open infrastructure funded by various academic organizations, providing a comprehensive catalog of scholarly papers, authors, institutions, and more. While completely free, providing an email address via `OPENALEX_EMAIL` gives access to the "polite pool" with better rate limits.

**HAL** is France's national open access repository, providing free access to academic papers from French institutions. It requires no API key and offers unlimited queries.

**DBLP** specializes in computer science bibliography, providing free access to citations for conference proceedings, journals, and books in CS. While comprehensive for its niche, it only supports search and does not provide full paper downloads.

**CrossRef** provides DOI (Digital Object Identifier) lookup services for free. It does not support full-text search but is invaluable for looking up paper metadata by DOI.

**IACR ePrint** focuses exclusively on cryptology research from the International Association for Cryptologic Research. All papers are freely available, and no API key is required.

**bioRxiv** is the primary preprint server for biology and life sciences, operated by Cold Spring Harbor Laboratory. All preprints are free to access.

**Zenodo** is CERN-hosted open science repository, providing free access to research data and papers from researchers worldwide. No API key is required for basic access.

**Unpaywall** helps find open access versions of papers by checking thousands of open access repositories. It's completely free and requires no API key.

**DOAJ** (Directory of Open Access Journals) indexes only peer-reviewed, open access journals. It provides free access to journal metadata and paper information.

**Connected Papers** helps researchers discover related papers using a visual graph approach. It offers a free tier with reasonable rate limits.

**WorldWideScience** is a global science gateway providing access to scientific databases from multiple countries. It's completely free and operated by the scientific community.

**OSF Preprints** provides access to preprints from the Open Science Framework, covering many disciplines. No API key is required.

**BASE** (Bielefeld Academic Search Engine) aggregates content from over 4,000 data sources. It provides free access to academic resources.

**Dimensions** is a research discovery platform funded by Digital Science, offering free access to papers, grants, patents, and clinical trials.

**MDPI** is an open access publisher that provides free access to all its journal articles. The website is free to use.

**Google Scholar** is completely free but disabled by default because it uses scraping techniques rather than an official API. Enabling it requires both compile-time feature flags and runtime configuration (see below).

#### Freemium Sources (Optional API Key for Higher Limits)

**Semantic Scholar** offers a free tier with approximately 100 papers per day and 1 request per second. Providing a `SEMANTIC_SCHOLAR_API_KEY` significantly increases these limits. The API key is free to obtain from the Semantic Scholar website.

**CORE** aggregates millions of open access papers from repositories worldwide. A free API key is available at core.ac.uk and increases rate limits. Without a key, functionality is limited.

**SciSpace** provides a research discovery platform with both free and paid tiers. An optional API key can be configured for enhanced access.

#### API Key Required (Free Registration Available)

**IEEE Xplore** requires an API key for programmatic access. Free API keys are available by registering at developer.ieee.org. The free tier provides limited requests per second, and full access requires institutional subscriptions.

**ACM Digital Library** requires an API key for its programmatic search API. Free API keys can be obtained from the ACM Developer Portal (developers.acm.org). Without an API key, functionality is severely limited.

**Springer** may require an API key for full programmatic access. Check the Springer Nature Developer Portal for registration and access details.

**JSTOR** is primarily a subscription-based service. Institutional access is typically required, and API access may need additional licensing.

### Source-Specific Configuration

#### Google Scholar Setup

Google Scholar is disabled by default both at compile-time and runtime due to its scraping-based approach:

1. **Compile with the feature flag:**
   ```bash
   cargo build --features google_scholar
   ```

2. **Enable at runtime:**
   ```bash
   export GOOGLE_SCHOLAR_ENABLED=true
   ```

#### Semantic Scholar API Key

```bash
export SEMANTIC_SCHOLAR_API_KEY="your-api-key-here"
```

Get your free API key at: https://www.semanticscholar.org/product/api

#### CORE API Key

```bash
export CORE_API_KEY="your-core-api-key"
```

Register for a free API key at: https://core.ac.uk/services/api

#### IEEE Xplore API Key

```bash
export IEEE_XPLORE_API_KEY="your-ieee-api-key"
```

Register for a free API key at: https://developer.ieee.org/

#### ACM API Key

```bash
export ACM_API_KEY="your-acm-api-key"
```

Register for a free API key at: https://developers.acm.org/

#### OpenAlex Email (Recommended)

```bash
export OPENALEX_EMAIL="your-email@example.com"
```

Providing an email gives you access to OpenAlex's "polite pool" with better rate limits.

### Rate Limit Configuration

Without API keys, sources enforce rate limits to prevent abuse. You can customize these limits:

```bash
# Semantic Scholar rate limit (requests per second)
export SEMANTIC_SCHOLAR_RATE_LIMIT=5

# Global rate limit
export RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND=10

# Disable rate limiting entirely
export RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND=0
```

**Note:** Disabling rate limits or setting them too high may result in your IP being blocked by the source APIs. Use with caution.

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
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp_x.x.x_amd64.deb

# Install the package
sudo dpkg -i research-master-mcp_x.x.x_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

### Linux (Alpine) - APK Package

```bash
# Download .apk package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp-x.x.x-x86_64.apk

# Install the package
sudo apk add --allow-untrusted research-master-mcp-x.x.x-x86_64.apk
```

### Linux (RedHat/Fedora) - RPM Package

```bash
# Download .rpm package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp-x.x.x-1.x86_64.rpm

# Install the package
sudo dnf install research-master-mcp-x.x.x-1.x86_64.rpm
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
| `biorxiv` | Enable bioRxiv/medRxiv source |
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
| `europe_pmc` | Enable EuropePMC source |
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

## MCP Client Integration

Research Master MCP is compatible with any Model Context Protocol client. Below are setup instructions for popular MCP-compatible applications.

<details>
<summary><b>Claude Desktop</b></summary>

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows:** `%APPDATA%/Claude/claude_desktop_config.json`

**Linux:** `~/.config/Claude/claude_desktop_config.json`

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
</details>

<details>
<summary><b>Zed Editor</b></summary>

Add to **project settings** (`.zed/settings.json`) or **global settings** (`~/.config/zed/settings.json`):

```json
{
  "model_context_provider": {
    "servers": {
      "research-master": {
        "command": "research-master-mcp",
        "args": ["serve"]
      }
    }
  }
}
```

For per-project configuration, create `.zed/settings.json` in your project root.
</details>

<details>
<summary><b>Continue (VS Code / JetBrains)</b></summary>

Add to **Continue config** (`~/.continue/config.json` or project `.continue/config.json`):

```json
{
  "models": [
    {
      "name": "claude",
      "provider": "anthropic"
    }
  ],
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"]
    }
  }
}
```

The `~` prefix expands to your home directory. Config file location:
- **VS Code:** `~/.continue/config.json`
- **JetBrains:** `~/.continue/config.json` or project `.continue/config.json`
</details>

<details>
<summary><b>Cursor</b></summary>

Cursor is compatible with Claude Desktop config. Edit `~/.cursor/mcp.json`:

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

Or use **Settings > Features > MCP** to configure via UI.
</details>

<details>
<summary><b>Goose</b></summary>

Goose uses the same config format as Claude Desktop. Edit `~/.config/goose/mcp_config.json`:

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

Ensure the binary is in your PATH or use absolute path:
```json
{
  "mcpServers": {
    "research-master": {
      "command": "/usr/local/bin/research-master-mcp",
      "args": ["serve"]
    }
  }
}
```
</details>

<details>
<summary><b>Tabby</b></summary>

Tabby supports MCP servers via its configuration. Edit `~/.tabby/mcp/servers.json`:

```json
{
  "servers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"]
    }
  }
}
```

Restart Tabby after configuration changes.
</details>

<details>
<summary><b>CLI with MCP Proxy</b></summary>

Use with any MCP proxy tool (e.g., `mcp-cli`, `glama-cli`):

```bash
# Install proxy
pip install mcp-cli

# Run with proxy
mcp-cli run --command "research-master-mcp" --args "serve"
```

Or use with Smithery for easy MCP server discovery:

```bash
# Install Smithery CLI
npm install -g @smithery/cli

# Add Research Master MCP
smithery add research-master-mcp
```
</details>

<details>
<summary><b>Homebrew Cask (macOS)</b></summary>

If installed via Homebrew cask, the binary is already in your PATH:

```bash
# Using Homebrew to install
brew tap hongkongkiwi/research-master-mcp
brew install --cask research-master-mcp

# Verify installation
research-master-mcp --version
```

The MCP server command in configs can simply be `"research-master-mcp"`.
</details>

<details>
<summary><b>Docker</b></summary>

Run via Docker for isolated execution:

```bash
# Build image
docker build -t research-master-mcp .

# Run with stdio mode
docker run --rm -i research-master-mcp serve --stdio

# Or use pre-built image (includes Poppler for PDF text extraction)
docker run --rm -i ghcr.io/hongkongkiwi/research-master-mcp serve --stdio

# OCR variant (adds Tesseract for scanned PDFs)
docker run --rm -i ghcr.io/hongkongkiwi/research-master-mcp-ocr serve --stdio

# Build OCR image with extra languages (e.g., English + German)
docker build -f Dockerfile.ocr -t research-master-mcp-ocr --build-arg OCR_LANGS="eng deu" .
```

For persistent configuration, mount volumes:
```bash
docker run --rm -i \
  -v ~/.config/research-master:/root/.config/research-master \
  -v ./downloads:/downloads \
  ghcr.io/hongkongkiwi/research-master-mcp serve --stdio
```
</details>

<details>
<summary><b>Cline (VS Code / JetBrains)</b></summary>

Cline supports MCP servers via `~/.cline/mcp_servers.json` or project `.cline/mcp_servers.json`:

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

**Or use absolute path:**

```json
{
  "mcpServers": {
    "research-master": {
      "command": "/usr/local/bin/research-master-mcp",
      "args": ["serve"]
    }
  }
}
```
</details>

<details>
<summary><b>Roo Code</b></summary>

Roo Code (formerly Rui) uses the same MCP config format as Claude Desktop. Edit `~/.config/roo/mcp_config.json`:

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

**Or use Settings UI:**
- Open Roo Code Settings
- Navigate to **Extensions > MCP**
- Add server configuration manually

**With environment variables:**

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "RESEARCH_MASTER_ENABLED_SOURCES": "arxiv,semantic"
      }
    }
  }
}
```
</details>

<details>
<summary><b>Kilo Code</b></summary>

Kilo Code supports MCP in `~/.config/kilo/mcp.json`:

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

**Or per-project:** Create `.kilo/mcp.json` in your project root.
</details>

<details>
<summary><b>VS Code (Direct MCP)</b></summary>

VS Code requires the **MCP for VS Code** extension or use with Continue extension (see above).

**Using MCP for VS Code extension:**
1. Install "MCP" extension from marketplace
2. Open Settings (Ctrl+,)
3. Search for "MCP Servers"
4. Add configuration:

```json
{
  "mcp.servers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"]
    }
  }
}
```
</details>

<details>
<summary><b>1MCP</b></summary>

1MCP is an MCP proxy/aggregator. Configure in `~/.config/1mcp/servers.json`:

```json
{
  "servers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"]
    }
  }
}
```

Run 1MCP with your preferred client:

```bash
# Start 1MCP proxy
1mcp serve --port 3000

# Or with custom config
1mcp serve --config ~/.config/1mcp/config.json
```
</details>

<details>
<summary><b>OpenAI Codex CLI</b></summary>

Codex CLI uses MCP configuration via `~/.config/codex/mcp.json`:

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

**Or use environment variable:**

```bash
export MCP_SERVERS='{"research-master": {"command": "research-master-mcp", "args": ["serve"]}}'
```
</details>

<details>
<summary><b>Gemini CLI</b></summary>

Gemini CLI supports MCP via `~/.config/gemini/mcp.json`:

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

**With API keys in environment:**

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "SEMANTIC_SCHOLAR_API_KEY": "${SEMANTIC_SCHOLAR_API_KEY}",
        "RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND": "10"
      }
    }
  }
}
```
</details>

<details>
<summary><b>OpenCode</b></summary>

OpenCode supports MCP servers. Configure via **Settings > MCP**:

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

**Or via config file:** `~/.opencode/mcp.json`

```json
{
  "servers": {
    "research-master": {
      "type": "stdio",
      "command": "research-master-mcp",
      "args": ["serve"]
    }
  }
}
```
</details>

<details>
<summary><b>Other MCP Clients</b></summary>

Research Master MCP works with any MCP-compatible client. General configuration pattern:

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

**Key clients known to work:**
- **Aider**: `~/.aider.mcp.json`
- **CopilotKit**: Uses environment variables
- **AgentOps**: Configure via dashboard
- **LangChain agents**: Pass via MCPConfig

For HTTP/SSE mode (alternative to stdio):

```bash
research-master-mcp serve --port 3000 --host 0.0.0.0
```

Then configure with HTTP endpoint:
```json
{
  "mcpServers": {
    "research-master": {
      "url": "http://localhost:3000/sse"
    }
  }
}
```
</details>

---

**Common Configuration Options:**

| Option | Description |
|--------|-------------|
| `command` | Binary name or full path to `research-master-mcp` |
| `args` | `["serve"]` for stdio mode, `["serve", "--port", "3000"]` for SSE |
| `env` | Optional environment variables (API keys, rate limits) |

**Example with environment variables:**

```json
{
  "mcpServers": {
    "research-master": {
      "command": "research-master-mcp",
      "args": ["serve"],
      "env": {
        "RESEARCH_MASTER_ENABLED_SOURCES": "arxiv,semantic,openalex",
        "RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND": "10"
      }
    }
  }
}
```
</details>

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
│   │   ├── dimensions.rs        # Dimensions
│   │   ├── ieee_xplore.rs       # IEEE Xplore
│   │   ├── europe_pmc.rs        # EuropePMC
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
| `ACM_RATE_LIMIT` | ACM Digital Library requests per second | `3` |

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
