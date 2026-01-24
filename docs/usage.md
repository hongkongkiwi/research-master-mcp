# Usage

This guide covers CLI commands and MCP server usage.

## MCP Server Mode

### STDIO Mode (Recommended for most clients)

For integration with Claude Desktop or other MCP clients:

```bash
research-master-mcp serve --stdio
```

### HTTP/SSE Mode

For clients that support HTTP-based MCP connections:

```bash
research-master-mcp serve --port 3000 --host 0.0.0.0
```

Then configure your client to connect to `http://localhost:3000/sse`.

## CLI Commands

### Search Command

Search for papers across all available research sources.

```bash
research-master-mcp search "transformer architecture" --year 2020-
```

**Options:**

| Option | Description |
|--------|-------------|
| `-s, --source <SOURCE>` | Source to search (default: all) |
| `-m, --max-results <N>` | Maximum results (default: 10) |
| `-y, --year <YEAR>` | Year filter (e.g., "2020", "2018-2022", "2010-", "-2015") |
| `--sort-by <FIELD>` | Sort by: relevance, date, citations, title, author |
| `--order <ORDER>` | Sort order: asc, desc |
| `-c, --category <CAT>` | Category/subject filter |
| `--dedup` | Deduplicate results |
| `--dedup-strategy <STRAT>` | Deduplication strategy: first, last, mark |

### Author Command

Search for papers by a specific author.

```bash
research-master-mcp author "Geoffrey Hinton" --source semantic
```

**Options:**

| Option | Description |
|--------|-------------|
| `-s, --source <SOURCE>` | Source to search (default: all with author search) |
| `-m, --max-results <N>` | Maximum results per source (default: 10) |
| `-y, --year <YEAR>` | Year filter |
| `--dedup` | Deduplicate results |
| `--dedup-strategy <STRAT>` | Deduplication strategy |

### Download Command

Download a paper PDF to your local filesystem.

```bash
research-master-mcp download 2301.12345 --source arxiv --output ./papers
```

**Options:**

| Option | Description |
|--------|-------------|
| `-s, --source <SOURCE>` | Paper source (required for CLI) |
| `-o, --output <PATH>` | Save path (default: ./downloads) |
| `--auto-filename` | Auto-generate filename from title (default: true) |
| `--create-dir` | Create parent directory if needed |
| `--doi <DOI>` | Paper DOI (optional, for verification) |

### Lookup Command

Look up a paper by its DOI.

```bash
research-master-mcp lookup 10.48550/arXiv.2301.12345
```

**Options:**

| Option | Description |
|--------|-------------|
| `-s, --source <SOURCE>` | Source to search (default: all with DOI lookup) |
| `--json` | Output as JSON |

### Citations Command

Get papers that cite a specific paper.

```bash
research-master-mcp citations 2301.12345 --source arxiv
```

### Deduplicate Command

Remove duplicate papers from a JSON file.

```bash
research-master-mcp dedupe papers.json --strategy first
```

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose logging (can be repeated) |
| `-q, --quiet` | Suppress non-error output |
| `-o, --output` | Output format: `auto`, `table`, `json`, `plain` |
| `--config` | Path to configuration file |
| `--timeout` | Request timeout in seconds (default: 30) |
| `--env` | Show all environment variables and exit |

## Quick Examples

```bash
# Search for papers
research-master-mcp search "transformer architecture" --year 2020-

# Search by author
research-master-mcp author "Geoffrey Hinton"

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

## Output Formats

### Auto Format (Default)

Automatically chooses between table and plain text based on terminal:

```bash
research-master-mcp search "transformer" --output auto
```

### Table Format

Tabular output for easy reading:

```bash
research-master-mcp search "transformer" --output table
```

### JSON Format

Machine-readable JSON output:

```bash
research-master-mcp search "transformer" --output json
```

### Plain Format

Simple line-by-line output:

```bash
research-master-mcp search "transformer" --output plain
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
