use anyhow::Result;
use clap::Parser;
use research_master_mcp::mcp::server::McpServer;
use research_master_mcp::sources::SourceRegistry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "research-master-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "MCP server for searching and downloading academic papers", long_about = None)]
struct Args {
    /// Run in stdio mode (for MCP clients like Claude Desktop)
    #[arg(long, default_value_t = false)]
    stdio: bool,

    /// Port for SSE mode (if not using stdio)
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let env_filter = if args.verbose {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::INFO
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| format!("research_master_mcp={}", env_filter)),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Research Master MCP v{}", env!("CARGO_PKG_VERSION"));

    // Create source registry with all available sources
    let registry = SourceRegistry::new();

    // Create and run MCP server
    let server = McpServer::new(registry)?;

    if args.stdio {
        tracing::info!("Running in stdio mode");
        server.run_stdio().await?;
    } else {
        tracing::info!("Running in SSE mode on port {}", args.port);
        server.run_sse(args.port).await?;
    }

    Ok(())
}
