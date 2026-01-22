//! MCP (Model Context Protocol) implementation.

pub mod server;
mod tools;
mod unified_tools;

pub use server::McpServer;
pub use tools::{Tool, ToolRegistry};
