//! Configuration management.

mod file_config;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const TEST_MODE_ENV_VAR: &str = "RESEARCH_MASTER_TEST_MODE";

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Cache directory (defaults to platform-specific cache dir)
    #[serde(default)]
    pub directory: Option<PathBuf>,

    /// TTL for search results in seconds (default: 30 minutes)
    #[serde(default = "default_search_ttl")]
    pub search_ttl_seconds: u64,

    /// TTL for citation/reference results in seconds (default: 15 minutes)
    #[serde(default = "default_citation_ttl")]
    pub citation_ttl_seconds: u64,

    /// Maximum cache size in MB (default: 500MB)
    #[serde(default = "default_max_cache_size")]
    pub max_size_mb: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("RESEARCH_MASTER_CACHE_ENABLED").is_ok(),
            directory: None,
            search_ttl_seconds: default_search_ttl(),
            citation_ttl_seconds: default_citation_ttl(),
            max_size_mb: default_max_cache_size(),
        }
    }
}

fn default_search_ttl() -> u64 {
    1800 // 30 minutes
}

fn default_citation_ttl() -> u64 {
    900 // 15 minutes
}

fn default_max_cache_size() -> usize {
    500
}

/// Get the default cache directory for the platform
pub fn default_cache_dir() -> PathBuf {
    // Try platform-specific cache directories first
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("research-master");
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            return PathBuf::from(xdg_cache).join("research-master");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".cache").join("research-master");
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(appdata).join("research-master").join("cache");
        }
    }

    // Fallback to current directory
    PathBuf::from(".research-master-cache")
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,
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

    /// Per-source HTTP proxy configuration
    /// Format: source_id:proxy_url (e.g., "arxiv:http://proxy:8080")
    #[serde(default)]
    pub proxy_http: Option<String>,

    /// Per-source HTTPS proxy configuration
    /// Format: source_id:proxy_url (e.g., "semantic:https://proxy:8080")
    #[serde(default)]
    pub proxy_https: Option<String>,

    /// Per-source rate limits (requests per second)
    /// Format: source_id:rate (e.g., "semantic:0.5,arxiv:5")
    /// Environment variable: RESEARCH_MASTER_RATE_LIMITS
    #[serde(default)]
    pub rate_limits: Option<String>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SourceConfig {
    fn from_env() -> Self {
        Self {
            enabled_sources: std::env::var("RESEARCH_MASTER_ENABLED_SOURCES").ok(),
            disabled_sources: std::env::var("RESEARCH_MASTER_DISABLED_SOURCES").ok(),
            proxy_http: std::env::var("RESEARCH_MASTER_PROXY_HTTP").ok(),
            proxy_https: std::env::var("RESEARCH_MASTER_PROXY_HTTPS").ok(),
            rate_limits: std::env::var("RESEARCH_MASTER_RATE_LIMITS").ok(),
        }
    }

    fn without_env() -> Self {
        Self {
            enabled_sources: None,
            disabled_sources: None,
            proxy_http: None,
            proxy_https: None,
            rate_limits: None,
        }
    }

    /// Parse per-source rate limits from config string
    /// Format: "source1:rate1,source2:rate2"
    /// Example: "semantic:0.5,arxiv:5,openalex:2"
    pub fn parse_rate_limits(&self) -> std::collections::HashMap<String, f32> {
        let mut limits = std::collections::HashMap::new();

        if let Some(ref limits_str) = self.rate_limits {
            for part in limits_str.split(',') {
                let parts: Vec<&str> = part.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(rate) = parts[1].parse::<f32>() {
                        limits.insert(parts[0].trim().to_string(), rate);
                    }
                }
            }
        }

        limits
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
        Self::from_env()
    }
}

impl ApiKeys {
    fn from_env() -> Self {
        Self {
            semantic_scholar: std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok(),
            core: std::env::var("CORE_API_KEY").ok(),
        }
    }

    fn without_env() -> Self {
        Self {
            semantic_scholar: None,
            core: None,
        }
    }
}

impl Config {
    fn from_env() -> Self {
        Self {
            api_keys: ApiKeys::from_env(),
            downloads: DownloadConfig::default(),
            rate_limits: RateLimitConfig::default(),
            sources: SourceConfig::from_env(),
            cache: CacheConfig::default(),
        }
    }

    fn without_env() -> Self {
        Self {
            api_keys: ApiKeys::without_env(),
            downloads: DownloadConfig::default(),
            rate_limits: RateLimitConfig::default(),
            sources: SourceConfig::without_env(),
            cache: CacheConfig::default(),
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
pub fn load_config(path: &Path) -> Result<Config, config::ConfigError> {
    let test_mode = std::env::var(TEST_MODE_ENV_VAR)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if test_mode {
        return Ok(Config::without_env());
    }

    let settings = config::Config::builder()
        .add_source(config::File::from(path))
        .add_source(config::Environment::with_prefix("RESEARCH_MASTER"))
        .build()?;

    settings.try_deserialize()
}

/// Get the configuration (from env vars or defaults)
pub fn get_config() -> Config {
    let test_mode = std::env::var(TEST_MODE_ENV_VAR)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if test_mode {
        Config::without_env()
    } else {
        Config::from_env()
    }
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

pub use file_config::ConfigFile;
pub use file_config::ConfigFileError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.downloads.organize_by_source);
        assert_eq!(config.rate_limits.default_requests_per_second, 5.0);
    }

    #[test]
    fn test_parse_rate_limits() {
        let source_config = SourceConfig {
            rate_limits: Some("semantic:0.5,arxiv:5,openalex:2.5".to_string()),
            ..Default::default()
        };

        let limits = source_config.parse_rate_limits();
        assert_eq!(limits.get("semantic").copied(), Some(0.5));
        assert_eq!(limits.get("arxiv").copied(), Some(5.0));
        assert_eq!(limits.get("openalex").copied(), Some(2.5));
        assert_eq!(limits.get("nonexistent"), None);
    }

    #[test]
    fn test_parse_rate_limits_empty() {
        let source_config = SourceConfig {
            rate_limits: None,
            ..Default::default()
        };

        let limits = source_config.parse_rate_limits();
        assert!(limits.is_empty());
    }

    #[test]
    fn test_parse_rate_limits_invalid_format() {
        let source_config = SourceConfig {
            rate_limits: Some("semantic:0.5,invalidformat,arxiv:5".to_string()),
            ..Default::default()
        };

        let limits = source_config.parse_rate_limits();
        assert_eq!(limits.get("semantic").copied(), Some(0.5));
        assert_eq!(limits.get("arxiv").copied(), Some(5.0));
        // invalidformat should be ignored (no colon)
        assert_eq!(limits.len(), 2);
    }
}
