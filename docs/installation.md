# Installation

This guide covers all methods for installing Research Master MCP.

## Quick Install (macOS with Homebrew)

```bash
# Add the custom tap
brew tap hongkongkiwi/research-master-mcp

# Install research-master-mcp
brew install research-master-mcp

# Start the MCP server
research-master-mcp serve --stdio
```

## Packages & Images Summary

| Package/Image | Where | Architectures |
|---|---|---|
| Release binaries (`.gz`, `.zip`) | GitHub Releases | Linux (x86_64, arm64), macOS (x86_64, arm64), Windows (x86_64) |
| Homebrew | Homebrew tap | macOS (x86_64, arm64) |
| `.deb` | GitHub Releases | Linux (amd64, arm64) |
| `.rpm` | GitHub Releases | Linux (x86_64, aarch64) |
| `.apk` | GitHub Releases | Alpine (x86_64, aarch64) |
| `cargo install` | crates.io | Any supported Rust target |
| Docker image | GHCR | linux/amd64, linux/arm64 |
| Docker image (OCR) | GHCR (`-ocr`) | linux/amd64, linux/arm64 |

## GitHub Releases (Binaries)

Download prebuilt binaries from the [GitHub Releases page](https://github.com/hongkongkiwi/research-master-mcp/releases). Assets include:

- **Linux (glibc)**: `research-master-mcp-x86_64-unknown-linux-gnu.gz`, `research-master-mcp-aarch64-unknown-linux-gnu.gz`
- **macOS**: `research-master-mcp-x86_64-apple-darwin.gz`, `research-master-mcp-aarch64-apple-darwin.gz`
- **Windows**: `research-master-mcp-windows.zip`

After downloading:

```bash
# Linux/macOS example
gunzip research-master-mcp-<target>.gz
chmod +x research-master-mcp-<target>
./research-master-mcp-<target> --version
```

## Homebrew (macOS)

```bash
# Add the custom tap
brew tap hongkongkiwi/research-master-mcp

# Install research-master-mcp
brew install research-master-mcp
```

## Linux (Debian/Ubuntu) - DEB Package

```bash
# Download .deb package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp_x.x.x_amd64.deb
# Or arm64:
# wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp_x.x.x_arm64.deb

# Install the package
sudo dpkg -i research-master-mcp_x.x.x_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

## Linux (Alpine) - APK Package

```bash
# Download .apk package from GitHub Releases (see asset names/paths)
# Example (x86_64):
# wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/x86_64/research-master-mcp-x.x.x-r0.apk

# Install the package
sudo apk add --allow-untrusted research-master-mcp-x.x.x-r0.apk
```

## Linux (RedHat/Fedora) - RPM Package

```bash
# Download .rpm package from GitHub Releases
wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp-x.x.x-1.x86_64.rpm
# Or arm64:
# wget https://github.com/hongkongkiwi/research-master-mcp/releases/download/vx.x.x/research-master-mcp-x.x.x-1.aarch64.rpm

# Install the package
sudo dnf install research-master-mcp-x.x.x-1.x86_64.rpm
```

## Crates.io

```bash
cargo install research-master-mcp
```

## From Source

```bash
git clone https://github.com/hongkongkiwi/research-master-mcp
cd research-master-mcp
cargo install --path .
```

## Compile-Time Feature Flags

Individual research sources can be disabled at compile time using Cargo features. By default, all sources are included except Google Scholar (requires `GOOGLE_SCHOLAR_ENABLED=true`).

### Available Features

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

### Feature Groups

| Group | Description |
|-------|-------------|
| `core` | arxiv, pubmed, semantic |
| `preprints` | arxiv, biorxiv |
| `full` | All sources (default) |

### Build Examples

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

## Dependencies for PDF Extraction

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

## Docker

### Basic Usage

```bash
# Build image
docker build -t research-master-mcp .

# Run with stdio mode
docker run --rm -i research-master-mcp serve --stdio

# Or use pre-built image (includes Poppler for PDF text extraction)
docker run --rm -i ghcr.io/hongkongkiwi/research-master-mcp serve --stdio
```

### OCR Variant (for scanned PDFs)

```bash
# OCR variant (adds Tesseract for scanned PDFs)
docker run --rm -i ghcr.io/hongkongkiwi/research-master-mcp-ocr serve --stdio

# Build OCR image with extra languages (e.g., English + German)
docker build -f Dockerfile.ocr -t research-master-mcp-ocr --build-arg OCR_LANGS="eng deu" .
```

### With Persistent Configuration

```bash
docker run --rm -i \
  -v ~/.config/research-master:/root/.config/research-master \
  -v ./downloads:/downloads \
  ghcr.io/hongkongkiwi/research-master-mcp serve --stdio
```

## Verifying Installation

```bash
# Check version
research-master-mcp --version

# Show all environment variables
research-master-mcp --env

# Search for papers (test)
research-master-mcp search "transformer architecture" --max-results 1
```

## Next Steps

- [Configure API Keys](sources.md#source-specific-configuration) for rate-limited sources
- [Set up your MCP client](mcp-clients.md) (Claude Desktop, Zed, etc.)
- [Configure environment variables](configuration.md)

## Related Documentation

- [Sources](sources.md) - Supported research sources and API requirements
- [Usage](usage.md) - CLI commands and options
- [Configuration](configuration.md) - Environment variables and config file
- [MCP Clients](mcp-clients.md) - Configuration for MCP clients
