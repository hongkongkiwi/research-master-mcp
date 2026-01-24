# Development Guide

This guide covers setting up the development environment, understanding the project structure, and adding new research sources.

## Quick Commands (using just)

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

## Project Structure

```
research-master-mcp/
├── Cargo.toml
├── justfile                     # Development commands
├── README.md
├── LICENSE
├── docs/                        # Documentation
│   ├── sources.md
│   ├── installation.md
│   ├── usage.md
│   ├── mcp-clients.md
│   ├── tools.md
│   ├── development.md
│   └── configuration.md
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

## Adding a New Source

### Step 1: Create the Source Module

Create a new file in `src/sources/` implementing the `Source` trait:

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

### Step 2: Add Feature Flag in Cargo.toml

```toml
[features]
default = ["source-mysource"]
source-mysource = []
```

### Step 3: Conditionally Compile the Module

In `src/sources/mod.rs`:

```rust
#[cfg(feature = "source-mysource")]
mod mysource;
```

### Step 4: Register the Source

In `src/sources/registry.rs`:

```rust
#[cfg(feature = "source-mysource")]
use super::mysource::MySource;

// In try_new():
#[cfg(feature = "source-mysource")]
try_register!(MySource::new());
```

### Step 5: Add SourceType Variant

In `src/models/paper.rs`:

```rust
pub enum SourceType {
    // ... existing variants
    MySource,
    // ...
}
```

### Step 6: Rebuild

The unified MCP tools will automatically include your new source.

## Building and Testing

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

## Related Documentation

- [Sources](sources.md) - Supported research sources
- [Configuration](configuration.md) - Environment variables and config file
- [Installation](installation.md) - Building from source
