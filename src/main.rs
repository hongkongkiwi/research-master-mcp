use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use research_master_mcp::config::{find_config_file, load_config};
use research_master_mcp::mcp::server::McpServer;
use research_master_mcp::models::{
    CitationRequest, DownloadRequest, ReadRequest, SearchQuery, SortBy, SortOrder,
};
use research_master_mcp::sources::{SourceCapabilities, SourceRegistry};
use research_master_mcp::utils::{
    deduplicate_papers, find_duplicates, CacheService, DuplicateStrategy,
};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Research Master MCP - Search and download academic papers from multiple research sources
#[derive(Parser, Debug)]
#[command(name = "research-master-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "hongkongkiwi")]
#[command(about = "Search and download academic papers from multiple research sources", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose logging (can be used multiple times for more verbosity: -v, -vv, -vvv)
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress non-error output
    #[arg(long, short)]
    quiet: bool,

    /// Output format
    #[arg(long, short, value_enum, global = true, default_value_t = OutputFormat::Auto)]
    output: OutputFormat,

    /// Configuration file path
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Request timeout in seconds
    #[arg(long, global = true, default_value_t = 30)]
    timeout: u64,

    /// Show all environment variables
    #[arg(long, global = true)]
    env: bool,

    /// Disable caching for this command (useful for testing fresh results)
    #[arg(long, global = true, default_value_t = false)]
    no_cache: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Output format for results
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    /// Automatic based on terminal (table if TTY, JSON otherwise)
    Auto,
    /// Table format (human-readable)
    Table,
    /// JSON format (machine-readable)
    Json,
    /// Plain text format
    Plain,
}

/// Available research sources
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    #[value(name = "arxiv")]
    Arxiv,
    #[value(name = "pubmed")]
    Pubmed,
    #[value(name = "biorxiv")]
    Biorxiv,
    #[value(name = "semantic")]
    Semantic,
    #[value(name = "openalex")]
    OpenAlex,
    #[value(name = "crossref")]
    CrossRef,
    #[value(name = "iacr")]
    Iacr,
    #[value(name = "pmc")]
    Pmc,
    #[value(name = "hal")]
    Hal,
    #[value(name = "dblp")]
    Dblp,
    #[value(name = "ssrn")]
    Ssrn,
    #[value(name = "all")]
    All,
}

/// Sort field for results
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum SortField {
    /// Sort by relevance
    Relevance,
    /// Sort by publication date
    Date,
    /// Sort by citation count
    Citations,
    /// Sort by title
    Title,
    /// Sort by author
    Author,
}

/// Sort order
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Strategy for handling duplicates
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum DedupStrategy {
    /// Keep the first occurrence of each duplicate group
    First,
    /// Keep the last occurrence of each duplicate group
    Last,
    /// Keep all papers but mark duplicates
    Mark,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for papers by query string
    #[command(alias = "s")]
    Search {
        /// Search query string
        query: String,

        /// Source to search (default: all)
        #[arg(long, short, value_enum, default_value_t = Source::All)]
        source: Source,

        /// Maximum number of results
        #[arg(long, short, default_value_t = 10)]
        max_results: usize,

        /// Year filter (e.g., "2020", "2018-2022", "2010-", "-2015")
        #[arg(long)]
        year: Option<String>,

        /// Sort by field
        #[arg(long, value_enum)]
        sort_by: Option<SortField>,

        /// Sort order
        #[arg(long, value_enum)]
        order: Option<Order>,

        /// Category/subject filter
        #[arg(long, short)]
        category: Option<String>,

        /// Author filter
        #[arg(long, short)]
        author: Option<String>,

        /// Deduplicate results across sources
        #[arg(long)]
        dedup: bool,

        /// Deduplication strategy (default: first)
        #[arg(long, value_enum, requires = "dedup")]
        dedup_strategy: Option<DedupStrategy>,

        /// Fetch detailed information (slower but more complete)
        #[arg(long, default_value_t = true)]
        fetch_details: bool,
    },

    /// Search for papers by author
    #[command(alias = "a")]
    Author {
        /// Author name
        author: String,

        /// Source to search (default: all sources that support author search)
        #[arg(long, short, value_enum, default_value_t = Source::All)]
        source: Source,

        /// Maximum number of results per source
        #[arg(long, short, default_value_t = 10)]
        max_results: usize,

        /// Year filter (e.g., "2020", "2018-2022", "2010-", "-2015")
        #[arg(long)]
        year: Option<String>,

        /// Deduplicate results across sources
        #[arg(long)]
        dedup: bool,

        /// Deduplication strategy (default: first)
        #[arg(long, value_enum, requires = "dedup")]
        dedup_strategy: Option<DedupStrategy>,
    },

    /// Download a paper's PDF
    #[command(alias = "d")]
    Download {
        /// Paper ID (source-specific identifier)
        paper_id: String,

        /// Source of the paper
        #[arg(long, short, value_enum)]
        source: Source,

        /// Path where to save the PDF
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Auto-generate filename from paper title
        #[arg(long)]
        auto_filename: bool,

        /// Create directory if it doesn't exist
        #[arg(long)]
        create_dir: bool,

        /// Paper DOI (optional, for verification)
        #[arg(long)]
        doi: Option<String>,
    },

    /// Read and extract text from a paper's PDF
    #[command(alias = "r")]
    Read {
        /// Paper ID (source-specific identifier)
        paper_id: String,

        /// Source of the paper
        #[arg(long, short, value_enum)]
        source: Source,

        /// Path where PDF is saved (or will be downloaded)
        #[arg(long, short)]
        path: PathBuf,

        /// Download PDF if not found locally
        #[arg(long, default_value_t = true)]
        download_if_missing: bool,

        /// Number of pages to extract (0 = all)
        #[arg(long)]
        pages: Option<usize>,

        /// Extract text to file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Get papers that cite a given paper
    #[command(alias = "c")]
    Citations {
        /// Paper ID (source-specific identifier)
        paper_id: String,

        /// Source of the paper
        #[arg(long, short, value_enum)]
        source: Source,

        /// Maximum number of results
        #[arg(long, short, default_value_t = 20)]
        max_results: usize,
    },

    /// Get papers referenced by a given paper
    #[command(alias = "ref")]
    References {
        /// Paper ID (source-specific identifier)
        paper_id: String,

        /// Source of the paper
        #[arg(long, short, value_enum)]
        source: Source,

        /// Maximum number of results
        #[arg(long, short, default_value_t = 20)]
        max_results: usize,
    },

    /// Get related/similar papers
    #[command(alias = "rel")]
    Related {
        /// Paper ID (source-specific identifier)
        paper_id: String,

        /// Source of the paper
        #[arg(long, short, value_enum)]
        source: Source,

        /// Maximum number of results
        #[arg(long, short, default_value_t = 20)]
        max_results: usize,
    },

    /// Look up a paper by DOI
    #[command(alias = "doi")]
    LookupByDoi {
        /// Digital Object Identifier
        doi: String,

        /// Source to use for lookup (default: all that support DOI lookup)
        #[arg(long, short, value_enum, default_value_t = Source::All)]
        source: Source,

        /// Return JSON output even in terminal
        #[arg(long, short)]
        json: bool,
    },

    /// List available sources and their capabilities
    #[command(alias = "ls")]
    Sources {
        /// Show detailed information about each source
        #[arg(long, short)]
        detailed: bool,

        /// Filter sources by capability
        #[arg(long, value_enum)]
        with_capability: Option<CapabilityFilter>,
    },

    /// Run the MCP server (for Claude Desktop and other MCP clients)
    Serve {
        /// Run in stdio mode (for MCP clients like Claude Desktop)
        #[arg(long, default_value_t = true)]
        stdio: bool,

        /// Run in HTTP/SSE mode (overrides --stdio)
        #[arg(long)]
        http: bool,

        /// Port for SSE mode (if not using stdio)
        #[arg(long, short, default_value_t = 3000)]
        port: u16,

        /// Host to bind to for SSE mode
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Deduplicate a JSON file containing papers
    #[command(alias = "dedup")]
    Dedupe {
        /// Input JSON file containing papers
        input: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Deduplication strategy
        #[arg(long, short, value_enum, default_value_t = DedupStrategy::First)]
        strategy: DedupStrategy,

        /// Show duplicate groups without removing
        #[arg(long, short)]
        show: bool,
    },

    /// Manage local cache
    #[command(alias = "c")]
    Cache {
        /// Subcommand
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CacheCommands {
    /// Show cache status and statistics
    Status,

    /// Clear all cached data
    Clear,

    /// Clear only search cache
    ClearSearches,

    /// Clear only citation cache
    ClearCitations,
}

/// Capability filter for listing sources
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum CapabilityFilter {
    Search,
    Download,
    Read,
    Citations,
    DoiLookup,
    AuthorSearch,
}

/// Print all available environment variables
fn print_env_vars() {
    println!("Research Master MCP - Environment Variables");
    println!();
    println!("API Keys:");
    println!("  SEMANTIC_SCHOLAR_API_KEY    API key for Semantic Scholar (higher rate limits)");
    println!("  CORE_API_KEY                API key for CORE service");
    println!("  OPENALEX_EMAIL              Email for OpenAlex 'polite pool' access");
    println!();
    println!("Source-Specific Rate Limits:");
    println!("  SEMANTIC_SCHOLAR_RATE_LIMIT  Semantic Scholar requests per second (default: 1)");
    println!();
    println!("Download Settings:");
    println!("  RESEARCH_MASTER_DOWNLOADS_DEFAULT_PATH     Default directory for PDF downloads (default: ./downloads)");
    println!("  RESEARCH_MASTER_DOWNLOADS_ORGANIZE_BY_SOURCE  Create subdirectories per source (default: true)");
    println!("  RESEARCH_MASTER_DOWNLOADS_MAX_FILE_SIZE_MB    Maximum file size for downloads in MB (default: 100)");
    println!();
    println!("Rate Limiting:");
    println!("  RESEARCH_MASTER_RATE_LIMITS_DEFAULT_REQUESTS_PER_SECOND  Default requests per second (default: 5.0)");
    println!("  RESEARCH_MASTER_RATE_LIMITS_MAX_CONCURRENT_REQUESTS     Max concurrent requests (default: 10)");
    println!();
    println!("Cache Settings:");
    println!(
        "  RESEARCH_MASTER_CACHE_ENABLED                Enable local caching (default: disabled)"
    );
    println!("  RESEARCH_MASTER_CACHE_DIRECTORY              Custom cache directory");
    println!("  RESEARCH_MASTER_CACHE_SEARCH_TTL_SECONDS     TTL for search results (default: 1800 = 30 min)");
    println!("  RESEARCH_MASTER_CACHE_CITATION_TTL_SECONDS   TTL for citation results (default: 900 = 15 min)");
    println!();
    println!("Other Settings:");
    println!("  RUST_LOG                    Rust logging level (e.g., debug, info, warn, error)");
    println!();
    println!("Example:");
    println!("  export SEMANTIC_SCHOLAR_API_KEY=\"your-key-here\"");
    println!("  export SEMANTIC_SCHOLAR_RATE_LIMIT=\"5\"");
    println!("  export RESEARCH_MASTER_DOWNLOADS_DEFAULT_PATH=\"./papers\"");
    std::process::exit(0);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Show environment variables and exit if requested
    if cli.env {
        print_env_vars();
    }

    // Initialize tracing based on verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let env_filter = if cli.quiet { "error" } else { log_level };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| format!("research_master_mcp={}", env_filter)),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set timeout
    tokio::time::sleep(Duration::from_secs(0)).await; // Just to ensure runtime is initialized

    // Load configuration from file if specified or found in default locations
    let _config = if let Some(config_path) = &cli.config {
        Some(load_config(config_path)?)
    } else if let Some(config_path) = find_config_file() {
        tracing::info!("Using config file: {}", config_path.display());
        Some(load_config(&config_path)?)
    } else {
        None
    };

    // Create source registry
    let registry = SourceRegistry::new();

    // Execute command
    match cli.command {
        Some(Commands::Search {
            query,
            source,
            max_results,
            year,
            sort_by,
            order,
            category,
            author,
            dedup,
            dedup_strategy,
            fetch_details,
        }) => {
            let mut search_query = SearchQuery::new(&query);
            search_query.max_results = max_results;
            search_query.year = year;
            search_query.sort_by = sort_by.map(|s| match s {
                SortField::Relevance => SortBy::Relevance,
                SortField::Date => SortBy::Date,
                SortField::Citations => SortBy::CitationCount,
                SortField::Title => SortBy::Title,
                SortField::Author => SortBy::Author,
            });
            search_query.sort_order = order.map(|o| match o {
                Order::Asc => SortOrder::Ascending,
                Order::Desc => SortOrder::Descending,
            });
            search_query.category = category;
            search_query.author = author;
            search_query.fetch_details = fetch_details;

            let sources = get_sources(&registry, source, SourceCapabilities::SEARCH);
            let mut all_papers = Vec::new();

            // Initialize cache if not disabled
            let cache = if cli.no_cache {
                None
            } else {
                let c = CacheService::new();
                let _ = c.initialize();
                Some(c)
            };

            for src in sources {
                let source_id = src.id();

                // Check cache first
                if let Some(ref cache_service) = cache {
                    match cache_service.get_search(&search_query, source_id) {
                        research_master_mcp::utils::CacheResult::Hit(response) => {
                            if !cli.quiet {
                                eprintln!(
                                    "[CACHE HIT] Found {} papers from {}",
                                    response.papers.len(),
                                    source_id
                                );
                            }
                            all_papers.extend(response.papers);
                            continue;
                        }
                        research_master_mcp::utils::CacheResult::Expired => {
                            if !cli.quiet {
                                eprintln!(
                                    "[CACHE EXPIRED] Fetching fresh results from {}",
                                    source_id
                                );
                            }
                        }
                        research_master_mcp::utils::CacheResult::Miss => {
                            if !cli.quiet {
                                eprintln!("[CACHE MISS] Fetching from {}", source_id);
                            }
                        }
                    }
                }

                // Fetch from API
                match src.search(&search_query).await {
                    Ok(response) => {
                        if !cli.quiet {
                            eprintln!("Found {} papers from {}", response.papers.len(), source_id);
                        }
                        // Cache the result
                        if let Some(ref cache_service) = cache {
                            cache_service.set_search(source_id, &search_query, &response);
                        }
                        all_papers.extend(response.papers);
                    }
                    Err(e) => {
                        if !cli.quiet {
                            eprintln!("Error searching {}: {}", source_id, e);
                        }
                    }
                }
            }

            if dedup {
                let strategy = match dedup_strategy.unwrap_or(DedupStrategy::First) {
                    DedupStrategy::First => DuplicateStrategy::First,
                    DedupStrategy::Last => DuplicateStrategy::Last,
                    DedupStrategy::Mark => DuplicateStrategy::Mark,
                };
                all_papers = deduplicate_papers(all_papers, strategy);
            }

            output_papers(&all_papers, cli.output);
        }

        Some(Commands::Author {
            author,
            source,
            max_results,
            year: _,
            dedup,
            dedup_strategy,
        }) => {
            let sources = get_sources(&registry, source, SourceCapabilities::AUTHOR_SEARCH);
            let mut all_papers = Vec::new();

            for src in sources {
                match src.search_by_author(&author, max_results).await {
                    Ok(response) => {
                        if !cli.quiet {
                            eprintln!("Found {} papers from {}", response.papers.len(), src.id());
                        }
                        all_papers.extend(response.papers);
                    }
                    Err(e) => {
                        if !cli.quiet {
                            eprintln!("Error searching author in {}: {}", src.id(), e);
                        }
                    }
                }
            }

            if dedup {
                let strategy = match dedup_strategy.unwrap_or(DedupStrategy::First) {
                    DedupStrategy::First => DuplicateStrategy::First,
                    DedupStrategy::Last => DuplicateStrategy::Last,
                    DedupStrategy::Mark => DuplicateStrategy::Mark,
                };
                all_papers = deduplicate_papers(all_papers, strategy);
            }

            output_papers(&all_papers, cli.output);
        }

        Some(Commands::Download {
            paper_id,
            source,
            output,
            auto_filename: _,
            create_dir,
            doi,
        }) => {
            let src = get_source(&registry, source)?;
            let save_path = output.unwrap_or_else(|| PathBuf::from("."));

            if create_dir {
                if let Some(parent) = save_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let mut request = DownloadRequest::new(&paper_id, save_path.to_string_lossy());
            if let Some(doi_val) = doi {
                request = request.doi(&doi_val);
            }

            let result = src.download(&request).await?;

            if result.success {
                if !cli.quiet {
                    eprintln!("Downloaded {} bytes to {}", result.bytes, result.path);
                }
            } else {
                anyhow::bail!("Download failed: {:?}", result.error);
            }
        }

        Some(Commands::Read {
            paper_id,
            source,
            path,
            download_if_missing,
            pages: _,
            output: output_file,
        }) => {
            let src = get_source(&registry, source)?;
            let request = ReadRequest::new(&paper_id, path.to_string_lossy())
                .download_if_missing(download_if_missing);

            let result = src.read(&request).await?;

            if result.success {
                let text = result.text;
                if let Some(output_path) = output_file {
                    std::fs::write(&output_path, text)?;
                    if !cli.quiet {
                        eprintln!("Text written to {}", output_path.display());
                    }
                } else {
                    println!("{}", text);
                }
            } else {
                anyhow::bail!("Read failed: {:?}", result.error);
            }
        }

        Some(Commands::Citations {
            paper_id,
            source,
            max_results,
        }) => {
            let src = get_source(&registry, source)?;
            let request = CitationRequest::new(&paper_id).max_results(max_results);

            let response = src.get_citations(&request).await?;
            output_papers(&response.papers, cli.output);
        }

        Some(Commands::References {
            paper_id,
            source,
            max_results,
        }) => {
            let src = get_source(&registry, source)?;
            let request = CitationRequest::new(&paper_id).max_results(max_results);

            let response = src.get_references(&request).await?;
            output_papers(&response.papers, cli.output);
        }

        Some(Commands::Related {
            paper_id,
            source,
            max_results,
        }) => {
            let src = get_source(&registry, source)?;
            let request = CitationRequest::new(&paper_id).max_results(max_results);

            let response = src.get_related(&request).await?;
            output_papers(&response.papers, cli.output);
        }

        Some(Commands::LookupByDoi { doi, source, json }) => {
            let sources = get_sources(&registry, source, SourceCapabilities::DOI_LOOKUP);
            let output_fmt = if json { OutputFormat::Json } else { cli.output };

            for src in sources {
                match src.get_by_doi(&doi).await {
                    Ok(paper) => {
                        output_papers(&[paper], output_fmt);
                        return Ok(());
                    }
                    Err(e) => {
                        if !cli.quiet {
                            eprintln!("Not found in {}: {}", src.id(), e);
                        }
                    }
                }
            }
            anyhow::bail!("Paper not found in any source");
        }

        Some(Commands::Sources {
            detailed,
            with_capability,
        }) => {
            let sources: Vec<_> = match with_capability {
                Some(CapabilityFilter::Search) => {
                    registry.with_capability(SourceCapabilities::SEARCH)
                }
                Some(CapabilityFilter::Download) => {
                    registry.with_capability(SourceCapabilities::DOWNLOAD)
                }
                Some(CapabilityFilter::Read) => registry.with_capability(SourceCapabilities::READ),
                Some(CapabilityFilter::Citations) => {
                    registry.with_capability(SourceCapabilities::CITATIONS)
                }
                Some(CapabilityFilter::DoiLookup) => {
                    registry.with_capability(SourceCapabilities::DOI_LOOKUP)
                }
                Some(CapabilityFilter::AuthorSearch) => {
                    registry.with_capability(SourceCapabilities::AUTHOR_SEARCH)
                }
                None => registry.all().collect(),
            };

            for src in sources {
                if detailed {
                    println!("{} ({})", src.name(), src.id());
                    println!("  Capabilities: {:?}", src.capabilities());
                } else {
                    println!("{} - {}", src.id(), src.name());
                }
            }
        }

        Some(Commands::Serve {
            stdio,
            http,
            port,
            host,
        }) => {
            let server = McpServer::new(Arc::new(registry))?;

            // Use HTTP mode if --http flag is provided, otherwise use --stdio flag
            let use_http = http || !stdio;

            if use_http {
                let addr = format!("{}:{}", host, port);
                tracing::info!("Running MCP server in HTTP/SSE mode on {}", addr);
                let (bound_addr, handle) = server.run_http(&addr).await?;
                tracing::info!("MCP server listening on {}", bound_addr);

                // Wait for the server to finish
                handle
                    .await
                    .map_err(|e| anyhow::anyhow!("Server task failed: {}", e))?;
            } else {
                tracing::info!("Running MCP server in stdio mode");
                server.run().await?;
            }
        }

        Some(Commands::Dedupe {
            input,
            output,
            strategy,
            show,
        }) => {
            let json_str = std::fs::read_to_string(&input)?;
            let papers: Vec<research_master_mcp::models::Paper> = serde_json::from_str(&json_str)?;

            let dup_strategy = match strategy {
                DedupStrategy::First => DuplicateStrategy::First,
                DedupStrategy::Last => DuplicateStrategy::Last,
                DedupStrategy::Mark => DuplicateStrategy::Mark,
            };

            if show {
                let groups = find_duplicates(&papers);
                if groups.is_empty() {
                    println!("No duplicates found");
                } else {
                    println!("Found {} duplicate groups:", groups.len());
                    for (i, group) in groups.iter().enumerate() {
                        println!("  Group {}: {} papers", i + 1, group.len());
                        for idx in group {
                            println!("    - {} ({})", papers[*idx].title, papers[*idx].source);
                        }
                    }
                }
            } else {
                let deduped = deduplicate_papers(papers, dup_strategy);
                let output_json = serde_json::to_string_pretty(&deduped)?;
                let output_path = output.as_ref().unwrap_or(&input);
                std::fs::write(output_path, output_json)?;
                if !cli.quiet {
                    eprintln!(
                        "Deduplicated: {} -> {} papers",
                        input.display(),
                        deduped.len()
                    );
                }
            }
        }

        Some(Commands::Cache { command }) => {
            let cache = CacheService::new();
            cache.initialize()?;

            match command {
                CacheCommands::Status => {
                    let stats = cache.stats();
                    if !stats.enabled {
                        println!("Cache: disabled");
                        println!("To enable, set RESEARCH_MASTER_CACHE_ENABLED=true");
                    } else {
                        println!("Cache: enabled");
                        println!("Directory: {}", stats.cache_dir.display());
                        println!(
                            "Search cache: {} items ({} KB)",
                            stats.search_count, stats.search_size_kb
                        );
                        println!(
                            "Citation cache: {} items ({} KB)",
                            stats.citation_count, stats.citation_size_kb
                        );
                        println!("Total size: {} KB", stats.total_size_kb);
                        println!("Search TTL: {} seconds", stats.ttl_search.as_secs());
                        println!("Citation TTL: {} seconds", stats.ttl_citations.as_secs());
                    }
                }
                CacheCommands::Clear => {
                    if !cli.quiet {
                        eprintln!("Clearing all cached data...");
                    }
                    cache.clear_all()?;
                    if !cli.quiet {
                        eprintln!("Cache cleared successfully.");
                    }
                }
                CacheCommands::ClearSearches => {
                    if !cli.quiet {
                        eprintln!("Clearing search cache...");
                    }
                    cache.clear_searches()?;
                    if !cli.quiet {
                        eprintln!("Search cache cleared successfully.");
                    }
                }
                CacheCommands::ClearCitations => {
                    if !cli.quiet {
                        eprintln!("Clearing citation cache...");
                    }
                    cache.clear_citations()?;
                    if !cli.quiet {
                        eprintln!("Citation cache cleared successfully.");
                    }
                }
            }
        }

        None => {
            // No command provided - show help
            println!("No command provided. Use --help for usage information.");
            println!("Common commands:");
            println!("  search <query>   - Search for papers");
            println!("  author <name>    - Search by author");
            println!("  download <id>    - Download a paper");
            println!("  sources          - List available sources");
            println!("  serve            - Run MCP server");
        }
    }

    Ok(())
}

fn get_source(
    registry: &SourceRegistry,
    source: Source,
) -> Result<&std::sync::Arc<dyn research_master_mcp::sources::Source>> {
    let source_id = match source {
        Source::All => anyhow::bail!("Please specify a specific source"),
        s => source_to_id(s),
    };
    registry
        .get_required(source_id)
        .map_err(|e| anyhow::anyhow!(e))
}

fn get_sources(
    registry: &SourceRegistry,
    source: Source,
    capability: SourceCapabilities,
) -> Vec<&std::sync::Arc<dyn research_master_mcp::sources::Source>> {
    match source {
        Source::All => registry.with_capability(capability),
        s => {
            let id = source_to_id(s);
            registry.get(id).into_iter().collect()
        }
    }
}

fn source_to_id(source: Source) -> &'static str {
    match source {
        Source::Arxiv => "arxiv",
        Source::Pubmed => "pubmed",
        Source::Biorxiv => "biorxiv",
        Source::Semantic => "semantic",
        Source::OpenAlex => "openalex",
        Source::CrossRef => "crossref",
        Source::Iacr => "iacr",
        Source::Pmc => "pmc",
        Source::Hal => "hal",
        Source::Dblp => "dblp",
        Source::Ssrn => "ssrn",
        Source::All => unreachable!(),
    }
}

fn output_papers(papers: &[research_master_mcp::models::Paper], format: OutputFormat) {
    let actual_format = if format == OutputFormat::Auto {
        if std::io::stdout().is_terminal() {
            OutputFormat::Table
        } else {
            OutputFormat::Json
        }
    } else {
        format
    };

    match actual_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(papers).unwrap());
        }
        OutputFormat::Plain => {
            for paper in papers {
                println!("{} - {} ({})", paper.title, paper.authors, paper.source);
                println!("  URL: {}", paper.url);
                if let Some(ref doi) = paper.doi {
                    println!("  DOI: {}", doi);
                }
                if let Some(ref pdf_url) = paper.pdf_url {
                    println!("  PDF: {}", pdf_url);
                }
                println!();
            }
        }
        OutputFormat::Table => {
            use comfy_table::{Attribute, Cell, Table};
            let mut table = Table::new();
            table.load_preset(comfy_table::presets::UTF8_FULL);
            table.set_header(vec!["Title", "Authors", "Source", "Year"]);

            for paper in papers {
                let year = paper
                    .published_date
                    .as_ref()
                    .map(|d| d.chars().take(4).collect::<String>())
                    .unwrap_or_default();

                let title = if paper.title.len() > 50 {
                    format!("{}...", &paper.title[..47])
                } else {
                    paper.title.clone()
                };

                let authors = if paper.authors.len() > 30 {
                    format!("{}...", &paper.authors[..27])
                } else {
                    paper.authors.clone()
                };

                table.add_row(vec![
                    Cell::new(title).add_attribute(Attribute::Bold),
                    Cell::new(authors),
                    Cell::new(paper.source.to_string()),
                    Cell::new(year),
                ]);
            }
            println!("{table}");
        }
        OutputFormat::Auto => unreachable!(),
    }
}
