//! # Research Master MCP
//!
//! A Model Context Protocol (MCP) server for searching and downloading academic papers
//! from multiple research sources.
//!
//! ## Architecture
//!
//! The library is organized into several modules:
//!
//! - [`models`]: Core data structures (Paper, SearchRequest, etc.)
//! - [`sources`]: Research source plugins with extensible trait-based architecture
//! - [`mcp`]: MCP protocol implementation and server
//! - [`utils`]: HTTP client, deduplication, and other utilities
//! - [`config`]: Configuration management

pub mod config;
pub mod mcp;
pub mod models;
pub mod sources;
pub mod ui;
pub mod utils;

// Re-export commonly used types
pub use models::Paper;
pub use sources::{Source, SourceRegistry};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
