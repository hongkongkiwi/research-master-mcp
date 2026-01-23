//! Configuration management.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// API keys for various services
    #[serde(default)]
    pub api_keys: ApiKeys,

    /// Download settings
    #[serde(default)]
    pub downloads: DownloadConfig,

    /// Rate limiting settings
    #[serde(default)]
    pub rate_limits: RateLimitConfig,

    /// Source filtering settings
    #[serde(default)]
    pub sources: SourceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_keys: ApiKeys::default(),
            downloads: DownloadConfig::default(),
            rate_limits: RateLimitConfig::default(),
            sources: SourceConfig::default(),
        }
    }
}

/// Source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Comma-separated list of source IDs to enable (e.g., "arxiv,pubmed,semantic")
    /// Maps to RESEARCH_MASTER_ENABLED_SOURCES environment variable
    #[serde(default)]
    pub enabled_sources: Option<String>,

    /// Comma-separated list of source IDs to disable (e.g., "dblp,jstor")
    /// Maps to RESEARCH_MASTER_DISABLED_SOURCES environment variable
    #[serde(default)]
    pub disabled_sources: Option<String>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            enabled_sources: std::env::var("RESEARCH_MASTER_ENABLED_SOURCES").ok(),
            disabled_sources: std::env::var("RESEARCH_MASTER_DISABLED_SOURCES").ok(),
        }
    }
}

/// API keys for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
    /// Semantic Scholar API key (optional, for higher rate limits)
    #[serde(default)]
    pub semantic_scholar: Option<String>,

    /// CORE API key (optional)
    #[serde(default)]
    pub core: Option<String>,
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self {
            semantic_scholar: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
            core: std::env::var("CORE_API_KEY").ok(),
        }
    }
}

/// Download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Default download directory
    #[serde(default = "default_download_dir")]
    pub default_path: PathBuf,

    /// Whether to create subdirectories per source
    #[serde(default = "default_true")]
    pub organize_by_source: bool,

    /// Maximum file size for downloads (in MB)
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            default_path: default_download_dir(),
            organize_by_source: true,
            max_file_size_mb: 100,
        }
    }
}

fn default_download_dir() -> PathBuf {
    PathBuf::from("./downloads")
}

fn default_true() -> bool {
    true
}

fn default_max_file_size() -> usize {
    100
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Default requests per second for APIs
    #[serde(default = "default_rps")]
    pub default_requests_per_second: f32,

    /// Maximum concurrent requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            default_requests_per_second: default_rps(),
            max_concurrent_requests: default_max_concurrent(),
        }
    }
}

fn default_rps() -> f32 {
    5.0
}

fn default_max_concurrent() -> usize {
    10
}

/// Load configuration from a file
pub fn load_config(path: &PathBuf) -> Result<Config, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::from(path.as_path()))
        .add_source(config::Environment::with_prefix("RESEARCH_MASTER"))
        .build()?;

    settings.try_deserialize()
}

/// Get the default configuration (from env vars or defaults)
pub fn get_config() -> Config {
    Config::default()
}

/// Search for configuration file in default locations
///
/// Searches in the following order:
/// 1. Current directory: `./research-master.toml`
/// 2. Current directory: `./.research-master.toml`
/// 3. XDG config dir: `$XDG_CONFIG_HOME/research-master/config.toml` (or `~/.config/research-master/config.toml`)
/// 4. macOS: `~/Library/Application Support/research-master/config.toml`
/// 5. Unix: `~/.config/research-master/config.toml`
/// 6. Windows: `%APPDATA%\research-master\config.toml`
pub fn find_config_file() -> Option<PathBuf> {
    // 1. Current directory - research-master.toml
    let path = PathBuf::from("research-master.toml");
    if path.exists() {
        return Some(path);
    }

    // 2. Current directory - .research-master.toml
    let path = PathBuf::from(".research-master.toml");
    if path.exists() {
        return Some(path);
    }

    // 3. XDG Config Home
    if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_home)
            .join("research-master")
            .join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // 4. macOS Application Support
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(&home);
        let path = home_path
            .join("Library")
            .join("Application Support")
            .join("research-master")
            .join("config.toml");
        if path.exists() {
            return Some(path);
        }

        // 5. Unix fallback (~/.config/research-master/config.toml)
        let path = home_path
            .join(".config")
            .join("research-master")
            .join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // 6. Windows APPDATA
    if let Ok(appdata) = std::env::var("APPDATA") {
        let path = PathBuf::from(appdata)
            .join("research-master")
            .join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.downloads.organize_by_source);
        assert_eq!(config.rate_limits.default_requests_per_second, 5.0);
    }
}
