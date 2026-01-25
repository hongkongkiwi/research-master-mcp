# Usage

This guide covers CLI commands and MCP server usage.

## MCP Server Mode

### STDIO Mode (Recommended for most clients)

For integration with Claude Desktop or other MCP clients:

```bash
research-master serve --stdio
```

### HTTP/SSE Mode

For clients that support HTTP-based MCP connections:

```bash
research-master serve --port 3000 --host 0.0.0.0
```

Then configure your client to connect to `http://localhost:3000/sse`.

## CLI Commands

### Search Command (`search` or `s`)

Search for papers across all available research sources.

```bash
research-master search "transformer architecture" --year 2020-
```

**Options:**

| Option | Description |
|--------|-------------|
| `query` (required) | Search query string |
| `-s, --source <SOURCE>` | Source to search (default: all) |
| `-m, --max-results <N>` | Maximum results (default: 10) |
| `-y, --year <YEAR>` | Year filter (e.g., "2020", "2018-2022", "2010-", "-2015") |
| `--sort-by <FIELD>` | Sort by: relevance, date, citations, title, author |
| `--order <ORDER>` | Sort order: asc, desc |
| `-c, --category <CAT>` | Category/subject filter |
| `--author <NAME>` | Author name filter |
| `--dedup` | Deduplicate results |
| `--dedup-strategy <STRAT>` | Deduplication strategy: first, last, mark |
| `--fetch-details` | Fetch detailed information (slower but more complete, default: true) |

### Author Command (`author` or `a`)

Search for papers by a specific author.

```bash
research-master author "Geoffrey Hinton" --source semantic
```

**Options:**

| Option | Description |
|--------|-------------|
| `author` (required) | Author name |
| `-s, --source <SOURCE>` | Source to search (default: all with author search) |
| `-m, --max-results <N>` | Maximum results per source (default: 10) |
| `-y, --year <YEAR>` | Year filter |
| `--dedup` | Deduplicate results |
| `--dedup-strategy <STRAT>` | Deduplication strategy |

**Supported sources:** arxiv, semantic, openalex, pubmed, biorxiv, pmc, hal, iacr, ssrn

### Download Command (`download` or `d`)

Download a paper PDF to your local filesystem.

```bash
research-master download 2301.12345 --source arxiv --output ./papers
```

**Options:**

| Option | Description |
|--------|-------------|
| `paper_id` (required) | Paper identifier |
| `-s, --source <SOURCE>` | Paper source (required for CLI) |
| `-o, --output <PATH>` | Save path (default: ./downloads) |
| `--auto-filename` | Auto-generate filename from title (default: true) |
| `--create-dir` | Create parent directory if needed |
| `--doi <DOI>` | Paper DOI (optional, for verification) |

### Read Command (`read` or `r`)

Read and extract text from a paper's PDF.

```bash
research-master read 2301.12345 --source arxiv --output extracted.txt
```

**Options:**

| Option | Description |
|--------|-------------|
| `paper_id` (required) | Paper identifier |
| `-s, --source <SOURCE>` | Paper source |
| `-p, --path <PATH>` | Path to PDF or where to download (default: ./downloads) |
| `--download-if-missing` | Download PDF if not found locally (default: true) |
| `--pages <N>` | Number of pages to extract (0 = all) |
| `-o, --output <PATH>` | Write extracted text to file |

**Note:** Requires poppler to be installed for PDF text extraction.

### Citations Command (`citations` or `c`)

Get papers that cite a specific paper.

```bash
research-master citations 2301.12345 --source arxiv --max-results 20
```

**Options:**

| Option | Description |
|--------|-------------|
| `paper_id` (required) | Paper identifier |
| `-s, --source <SOURCE>` | Source to search (default: semantic) |
| `-m, --max-results <N>` | Maximum results (default: 20) |

### References Command (`references` or `ref`)

Get papers referenced by a specific paper.

```bash
research-master references 1706.03762 --source semantic
```

**Options:**

| Option | Description |
|--------|-------------|
| `paper_id` (required) | Paper identifier |
| `-s, --source <SOURCE>` | Source to search (default: semantic) |
| `-m, --max-results <N>` | Maximum results (default: 20) |

### Related Command (`related` or `rel`)

Get related/similar papers.

```bash
research-master related 1706.03762 --source connected_papers
```

**Options:**

| Option | Description |
|--------|-------------|
| `paper_id` (required) | Paper identifier |
| `-s, --source <SOURCE>` | Source to search (default: connected_papers) |
| `-m, --max-results <N>` | Maximum results (default: 20) |

### Lookup Command (`lookup` or `doi`)

Look up a paper by its DOI.

```bash
research-master lookup 10.48550/arXiv.2301.12345
```

**Options:**

| Option | Description |
|--------|-------------|
| `doi` (required) | Digital Object Identifier |
| `-s, --source <SOURCE>` | Source to search (default: all with DOI lookup) |
| `-j, --json` | Output as JSON |

**Supported sources:** semantic, openalex, crossref, hal, doaj, osf, springer, mdpi, acm, base, unpaywall

### Sources Command (`sources` or `ls`)

List available sources and their capabilities.

```bash
research-master sources --detailed
```

**Options:**

| Option | Description |
|--------|-------------|
| `-d, --detailed` | Show detailed information about each source |
| `--with-capability <CAP>` | Filter sources by capability (search, download, read, citations, doi_lookup, author_search) |

### Cache Command (`cache`)

Manage local cache.

```bash
# Show cache status
research-master cache status

# Clear all cached data
research-master cache clear

# Clear only search cache
research-master cache clear-searches

# Clear only citation cache
research-master cache clear-citations
```

### Doctor Command (`doctor` or `diag`)

Check configuration and source health.

```bash
# Basic check
research-master doctor

# Check connectivity to all sources
research-master doctor --check-connectivity

# Check API keys
research-master doctor --check-api-keys

# Verbose output
research-master doctor --check-connectivity --check-api-keys --verbose
```

### Update Command (`update`)

Update to the latest version.

```bash
# Check for updates
research-master update

# Force update even if already at latest
research-master update --force

# Preview what would be updated
research-master update --dry-run
```

### Dedupe Command (`dedupe`)

Remove duplicate papers from a JSON file.

```bash
research-master dedupe papers.json --strategy first --output deduplicated.json

# Just show duplicates without removing
research-master dedupe papers.json --show
```

**Options:**

| Option | Description |
|--------|-------------|
| `input` (required) | Input JSON file containing papers |
| `-o, --output <PATH>` | Output file (default: overwrite input) |
| `-s, --strategy <STRAT>` | Deduplication strategy: first, last, mark (default: first) |
| `--show` | Show duplicate groups without removing |

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose logging (can be repeated: -v, -vv, -vvv) |
| `-q, --quiet` | Suppress non-error output |
| `-o, --output <FORMAT>` | Output format: `auto`, `table`, `json`, `plain` (default: auto) |
| `--config <PATH>` | Path to configuration file |
| `--timeout <SECONDS>` | Request timeout in seconds (default: 30) |
| `--env` | Show all environment variables and exit |
| `--no-cache` | Disable caching for this command |

## Available Sources

| Source | ID | Search | Download | Read | Citations | DOI Lookup | Author Search |
|--------|-----|--------|----------|------|-----------|------------|---------------|
| arXiv | arxiv | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PubMed | pubmed | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| bioRxiv | biorxiv | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Semantic Scholar | semantic | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| OpenAlex | openalex | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| CrossRef | crossref | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |
| IACR ePrint | iacr | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| PMC | pmc | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| HAL | hal | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| DBLP | dblp | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| SSRN | ssrn | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Dimensions | dimensions | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| IEEE Xplore | ieee_xplore | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Europe PMC | europe_pmc | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ |
| CORE | core | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| Zenodo | zenodo | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| Unpaywall | unpaywall | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ |
| MDPI | mdpi | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| JSTOR | jstor | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| SciSpace | scispace | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| ACM DL | acm | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ |
| Connected Papers | connected_papers | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| DOAJ | doaj | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| WorldWideScience | worldwidescience | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| OSF Preprints | osf | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| BASE | base | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ |
| Springer | springer | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ |
| Google Scholar | google_scholar | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |

## Output Formats

### Auto Format (Default)

Automatically chooses between table and plain text based on terminal:

```bash
research-master search "transformer" --output auto
```

### Table Format

Tabular output for easy reading:

```bash
research-master search "transformer" --output table
```

### JSON Format

Machine-readable JSON output:

```bash
research-master search "transformer" --output json
```

### Plain Format

Simple line-by-line output:

```bash
research-master search "transformer" --output plain
```

## Quick Examples

```bash
# Search for papers
research-master search "transformer architecture" --year 2020-

# Search by author
research-master author "Geoffrey Hinton"

# Download a paper
research-master download 2301.12345 --source arxiv --output ./papers

# Read extracted text from a PDF
research-master read 2301.12345 --source arxiv --output extracted.txt

# Look up by DOI
research-master lookup 10.48550/arXiv.2301.12345

# Get citations
research-master citations 2301.12345 --source arxiv

# Get references
research-master references 1706.03762

# Get related papers
research-master related 1706.03762 --source connected_papers

# List all sources with capabilities
research-master sources --detailed --with-capability download

# Check system health
research-master doctor --check-connectivity

# Deduplicate papers
research-master dedupe papers.json --strategy first

# Show all environment variables
research-master --env

# Start MCP server
research-master serve --stdio
```

## Next Steps

- [Set up your MCP client](mcp-clients.md) for integration with AI assistants
- [Configure sources](sources.md) with API keys for better rate limits
- [Configure environment variables](configuration.md)

## Related Documentation

- [Sources](sources.md) - Supported research sources
- [MCP Clients](mcp-clients.md) - Configuration for MCP clients
- [Configuration](configuration.md) - Environment variables and config file
- [Tools](tools.md) - Available MCP tools reference
