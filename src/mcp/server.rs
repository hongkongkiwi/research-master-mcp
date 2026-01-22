//! MCP server implementation using stdio transport.

use std::sync::Arc;

use crate::mcp::tools::ToolRegistry;
use crate::sources::SourceRegistry;

/// The MCP server for Research Master
///
/// This server communicates over stdio and provides tools for searching
/// and downloading academic papers from multiple research sources.
#[derive(Debug, Clone)]
pub struct McpServer {
    tools: ToolRegistry,
}

impl McpServer {
    /// Create a new MCP server with the given source registry
    pub fn new(sources: SourceRegistry) -> Result<Self, anyhow::Error> {
        let tools = ToolRegistry::from_sources(&sources);

        Ok(Self { tools })
    }

    /// Get all available tools
    pub fn tools(&self) -> Vec<&crate::mcp::Tool> {
        self.tools.all()
    }

    /// Run the server in stdio mode
    ///
    /// This reads JSON-RPC messages from stdin and writes responses to stdout
    pub async fn run_stdio(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Starting MCP server in stdio mode");

        // Create stdin/stdout streams
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        // For now, we'll implement a simple JSON-RPC server
        // In a full implementation, this would use the mcp-sdk properly
        self.run_simple_stdio(stdin, stdout).await
    }

    /// Simple stdio implementation (placeholder for proper MCP SDK integration)
    async fn run_simple_stdio<R, W>(
        &self,
        _reader: R,
        _writer: W,
    ) -> Result<(), anyhow::Error>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        // This is a simplified implementation
        // A full implementation would use the mcp-sdk to handle:
        // - initialize
        // - tools/list
        // - tools/call
        // - resources/list
        // - prompts/list
        // etc.

        tracing::info!("MCP server running");
        tracing::info!("Available tools: {}", self.tools().len());

        for tool in self.tools() {
            tracing::debug!("  - {}", tool.name);
        }

        // Keep the server running
        // In the full implementation, this would be a loop reading JSON-RPC messages
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())?;
            sigterm.recv().await;
            tracing::info!("Received SIGTERM, shutting down");
        }

        #[cfg(not(unix))]
        {
            // For non-Unix systems, just sleep indefinitely
            // In production, you'd have proper shutdown handling
            std::future::pending::<()>().await;
        }

        Ok(())
    }

    /// Run the server in SSE (Server-Sent Events) mode
    ///
    /// This starts an HTTP server that streams events over SSE
    pub async fn run_sse(&self, _port: u16) -> Result<(), anyhow::Error> {
        tracing::info!("Starting MCP server in SSE mode");

        // SSE mode is typically used for web-based connections
        // For now, we'll just log and wait
        tracing::info!("SSE mode not yet fully implemented");
        tracing::info!("Available tools: {}", self.tools().len());

        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())?;
            sigterm.recv().await;
            tracing::info!("Received SIGTERM, shutting down");
        }

        #[cfg(not(unix))]
        {
            std::future::pending::<()>().await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let sources = SourceRegistry::new();
        let server = McpServer::new(sources);
        assert!(server.is_ok());

        let server = server.unwrap();
        // Should have at least the arxiv search tool
        assert!(!server.tools().is_empty());
    }
}
