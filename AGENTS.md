# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Research Master MCP is a Model Context Protocol (MCP) server that provides unified access to 28+ academic research sources (arXiv, Semantic Scholar, PubMed, OpenAlex, Europe PMC, etc.) for searching, downloading, and analyzing academic papers.

## Build & Test Commands

```bash
# Build the project
cargo build

# Build with specific features (e.g., only core sources)
cargo build --no-default-features --features core

# Build with specific sources
cargo build --no-default-features --features arxiv,semantic,openalex

# Build with all sources except one
cargo build --features -dblp

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run doc tests
cargo test --doc

# Release build
cargo build --release

# Check for clippy warnings
cargo clippy

# Format code
cargo fmt
```

## Architecture

### Core Modules

- **`src/lib.rs`** - Library root, exports public API
- **`src/main.rs`** - CLI entry point using clap for argument parsing
- **`src/sources/`** - Plugin-based architecture with 28 research sources
- **`src/mcp/`** - MCP protocol server implementation
- **`src/models/`** - Core data structures (Paper, SearchQuery, etc.)
- **`src/utils/`** - HTTP client, deduplication, PDF extraction

### Source Plugin System

All sources implement the [`Source`](src/sources/mod.rs:138) trait from `src/sources/mod.rs`. Key aspects:

1. **Capability-based design**: Each source declares capabilities via `SourceCapabilities` bitflags (SEARCH, DOWNLOAD, READ, CITATIONS, DOI_LOOKUP, AUTHOR_SEARCH)

2. **Feature-gated modules**: Sources are conditionally compiled via `#[cfg(feature = "source-xxx")]` in `src/sources/mod.rs`

3. **Registration in registry**: Sources are registered in `src/sources/registry.rs` via `try_register!()` macro

4. **Unified tool handlers**: `src/mcp/unified_tools.rs` provides smart source auto-detection for paper IDs (e.g., arXiv IDs, DOIs, PMC IDs)

### Data Flow

1. **CLI/MCP Request** → `main.rs` parses args or MCP receives JSON-RPC
2. **Unified Tool Handler** → `src/mcp/unified_tools.rs` determines appropriate source(s)
3. **Source Implementation** → Specific source (e.g., `arxiv.rs`) makes HTTP API calls
4. **Response Parsing** → Deserialize JSON/XML to `Paper` models
5. **Deduplication** → Optional Jaro-Winkler similarity matching in `src/utils/dedup.rs`

### HTTP Client

`src/utils/http.rs` provides a rate-limited HTTP client using the `governor` crate for GCRA (Generic Cell Rate Limiting). Each source has configurable rate limits via environment variables.

## Key Patterns

### Source Implementation Pattern

```rust
#[derive(Debug)]
pub struct MySource;

#[async_trait]
impl Source for MySource {
    fn id(&self) -> &str { "mysource" }
    fn name(&self) -> &str { "My Source" }
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities::SEARCH | SourceCapabilities::DOWNLOAD
    }

    async fn search(&self, query: &SearchQuery) -> Result<SearchResponse, SourceError> {
        // Implementation
    }
}
```

### Error Handling

Sources use `SourceError` enum with conversion implementations from `reqwest::Error`, `serde_json::Error`, and `quick_xml::DeError`.

## Important Configuration

- **Google Scholar**: Disabled by default, requires both `--features google_scholar` at compile time AND `GOOGLE_SCHOLAR_ENABLED=true` at runtime
- **API Keys**: Optional for most sources (rate limits apply). Keys via environment variables (e.g., `SEMANTIC_SCHOLAR_API_KEY`)
- **Poppler**: Required for PDF text extraction (`brew install poppler` on macOS, `apt-get install libpoppler-cpp-dev` on Ubuntu)

## Source Auto-Detection

Paper ID formats are auto-detected in `unified_tools.rs`:
- `arXiv:1234.5678` or numeric → arXiv
- `PMC12345678` → PMC
- `hal-xxxxxxx` → HAL
- `xxxx/xxxx` (single slash) → IACR
- `10.xxxx/xxxxxx` → DOI lookup (Semantic Scholar preferred)

## Research Sources (28 total)

| Source | Capabilities | Description |
|--------|-------------|-------------|
| arXiv | Search, Download | Preprint papers in physics, math, CS, and more |
| PubMed | Search, Citations | Biomedical literature from NIH/NLM |
| bioRxiv | Search, Download | Biology preprints |
| Semantic Scholar | Search, Citations, Download | AI-powered academic search |
| OpenAlex | Search, Citations | Open scholarly knowledge graph |
| CrossRef | Search, DOI Lookup | DOI registration metadata |
| IACR | Search, Download | Cryptology research papers |
| PMC | Search, Download | PubMed Central open access papers |
| HAL | Search, Download | French open archive |
| DBLP | Search | Computer science bibliography |
| SSRN | Search, Download | Social science research network |
| Dimensions | Search | Research discovery platform |
| IEEE Xplore | Search, Download | Engineering and technology papers |
| Europe PMC | Search, Citations | European PMC mirror |
| CORE | Search | Open access research papers |
| Zenodo | Search, Download | CERN-backed research repository |
| Unpaywall | DOI Lookup | Find open access versions of papers |
| MDPI | Search, Download | Open access publisher |
| JSTOR | Search | Humanities and social sciences |
| SciSpace | Search, Citations | Scientific research platform |
| ACM | Search, Download | Computing literature |
| Connected Papers | Citations | Find related papers |
| DOAJ | Search | Directory of open access journals |
| WorldWideScience | Search | Global science portal |
| OSF | Search, Download | Open Science Framework preprints |
| BASE | Search | Bielefeld Academic Search Engine |
| Springer | Search, Download | Springer/Nature publications |
| Google Scholar | Search | Web search for scholarly literature |

### Convenience Feature Groups

- `core` - arxiv, pubmed, semantic
- `preprints` - arxiv, biorxiv
- `full` - All sources (default, excludes google_scholar)
