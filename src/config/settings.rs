//! Enhanced configuration management system
//!
//! Provides comprehensive configuration loading from environment variables,
//! configuration files, and command-line overrides.
//!
//! Based on TypeScript environment variable usage throughout the project.

use serde::{Deserialize, Serialize};
use std::time::Duration;

// Helper functions for serde defaults
fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_true() -> bool {
    true
}

fn default_max_body_size() -> usize {
    1024 * 1024
}

fn default_max_cache_entries() -> usize {
    1000
}

fn default_cache_cleanup_interval() -> u64 {
    60
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_request_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_interval() -> u64 {
    5000
}

fn default_user_agent() -> String {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
}

fn default_request_key() -> String {
    "O43z0dpjhgX20SCx4KAo".to_string()
}

fn default_vm_timeout() -> u64 {
    30
}

fn default_memory_cache_size() -> usize {
    100
}

// Duration serialization module
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Main configuration settings for the POT provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Server configuration
    #[serde(default)]
    pub server: ServerSettings,
    /// Token configuration
    #[serde(default)]
    pub token: TokenSettings,
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingSettings,
    /// Network configuration
    #[serde(default)]
    pub network: NetworkSettings,
    /// BotGuard configuration
    #[serde(default)]
    pub botguard: BotGuardSettings,
    /// Cache configuration
    #[serde(default)]
    pub cache: CacheSettings,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Request timeout duration
    #[serde(with = "duration_secs", default = "default_timeout")]
    pub timeout: Duration,
    /// Enable CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,
    /// Maximum request body size
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

/// Token generation and caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSettings {
    /// Token TTL in hours (corresponds to TypeScript TOKEN_TTL env var)
    pub ttl_hours: u64,
    /// Enable token caching
    #[serde(default = "default_true")]
    pub enable_cache: bool,
    /// Maximum cache entries
    #[serde(default = "default_max_cache_entries")]
    pub max_cache_entries: usize,
    /// Cache cleanup interval in minutes
    #[serde(default = "default_cache_cleanup_interval")]
    pub cache_cleanup_interval: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
    /// Log format (text, json)
    #[serde(default = "default_log_format")]
    pub format: String,
    /// Enable request/response logging
    #[serde(default = "default_true")]
    pub log_requests: bool,
}

/// Network and proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// HTTPS proxy URL (corresponds to TypeScript HTTPS_PROXY)
    #[serde(default)]
    pub https_proxy: Option<String>,
    /// HTTP proxy URL (corresponds to TypeScript HTTP_PROXY)
    #[serde(default)]
    pub http_proxy: Option<String>,
    /// All protocols proxy URL (corresponds to TypeScript ALL_PROXY)
    #[serde(default)]
    pub all_proxy: Option<String>,
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
    /// Number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Retry interval in milliseconds
    #[serde(default = "default_retry_interval")]
    pub retry_interval: u64,
    /// User agent string
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

/// BotGuard specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotGuardSettings {
    /// Request key for BotGuard API (hardcoded in TypeScript as O43z0dpjhgX20SCx4KAo)
    #[serde(default = "default_request_key")]
    pub request_key: String,
    /// Enable JavaScript VM execution
    #[serde(default = "default_true")]
    pub enable_vm: bool,
    /// VM execution timeout in seconds
    #[serde(default = "default_vm_timeout")]
    pub vm_timeout: u64,
    /// Force disable Innertube API usage
    #[serde(default)]
    pub disable_innertube: bool,
    /// Custom challenge endpoint URL
    #[serde(default)]
    pub challenge_endpoint: Option<String>,
    /// BotGuard snapshot file path for caching
    #[serde(default)]
    pub snapshot_path: Option<std::path::PathBuf>,
    /// Custom User Agent for BotGuard
    #[serde(default)]
    pub user_agent: Option<String>,
    /// Disable snapshot functionality
    #[serde(default)]
    pub disable_snapshot: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Cache directory path (for script mode)
    #[serde(default)]
    pub cache_dir: Option<String>,
    /// Enable file-based caching
    #[serde(default = "default_true")]
    pub enable_file_cache: bool,
    /// Memory cache size limit
    #[serde(default = "default_memory_cache_size")]
    pub memory_cache_size: usize,
    /// Enable cache compression
    #[serde(default)]
    pub enable_compression: bool,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "::".to_string(),
            port: 4416,
            timeout: default_timeout(),
            enable_cors: default_true(),
            max_body_size: default_max_body_size(),
        }
    }
}

impl Default for TokenSettings {
    fn default() -> Self {
        Self {
            ttl_hours: 6,
            enable_cache: default_true(),
            max_cache_entries: default_max_cache_entries(),
            cache_cleanup_interval: default_cache_cleanup_interval(),
        }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            verbose: false,
            format: default_log_format(),
            log_requests: default_true(),
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            https_proxy: None,
            http_proxy: None,
            all_proxy: None,
            connect_timeout: default_connect_timeout(),
            request_timeout: default_request_timeout(),
            max_retries: default_max_retries(),
            retry_interval: default_retry_interval(),
            user_agent: default_user_agent(),
        }
    }
}

impl Default for BotGuardSettings {
    fn default() -> Self {
        Self {
            request_key: default_request_key(),
            enable_vm: default_true(),
            vm_timeout: default_vm_timeout(),
            disable_innertube: false,
            challenge_endpoint: None,
            snapshot_path: Some(
                dirs::data_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("bgutil-ytdlp-pot-provider")
                    .join("botguard_snapshot.bin")
            ),
            user_agent: None, // Use rustypipe-botguard default
            disable_snapshot: false,
        }
    }
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            cache_dir: None,
            enable_file_cache: default_true(),
            memory_cache_size: default_memory_cache_size(),
            enable_compression: false,
        }
    }
}

impl Settings {
    /// Create new settings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from environment variables
    ///
    /// Corresponds to TypeScript environment variable usage throughout the project
    pub fn from_env() -> crate::Result<Self> {
        let mut settings = Self::default();

        // Load server settings
        if let Ok(host) = std::env::var("POT_SERVER_HOST") {
            settings.server.host = host;
        }

        if let Ok(port) = std::env::var("POT_SERVER_PORT") {
            settings.server.port = port
                .parse()
                .map_err(|e| crate::Error::config("port", &format!("Invalid port: {}", e)))?;
        }

        if let Ok(timeout) = std::env::var("POT_SERVER_TIMEOUT") {
            let timeout_secs: u64 = timeout
                .parse()
                .map_err(|e| crate::Error::config("timeout", &format!("Invalid timeout: {}", e)))?;
            settings.server.timeout = Duration::from_secs(timeout_secs);
        }

        // Load token settings (TOKEN_TTL from TypeScript)
        if let Ok(ttl) = std::env::var("TOKEN_TTL") {
            settings.token.ttl_hours = ttl
                .parse()
                .map_err(|e| crate::Error::config("TOKEN_TTL", &format!("Invalid TTL: {}", e)))?;
        }

        // Load network/proxy settings (from TypeScript)
        settings.network.https_proxy = std::env::var("HTTPS_PROXY").ok();
        settings.network.http_proxy = std::env::var("HTTP_PROXY").ok();
        settings.network.all_proxy = std::env::var("ALL_PROXY").ok();

        // Load logging settings
        if let Ok(level) = std::env::var("LOG_LEVEL") {
            settings.logging.level = level;
        }

        if let Ok(verbose) = std::env::var("VERBOSE") {
            settings.logging.verbose = verbose.parse().unwrap_or(false);
        }

        // Load BotGuard settings
        if let Ok(disable_innertube) = std::env::var("DISABLE_INNERTUBE") {
            settings.botguard.disable_innertube = disable_innertube.parse().unwrap_or(false);
        }

        // Load cache settings
        settings.cache.cache_dir = std::env::var("CACHE_DIR").ok();

        Ok(settings)
    }

    /// Load settings from configuration file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::Error::config("file", &format!("Failed to read config file: {}", e))
        })?;

        let settings: Settings = toml::from_str(&content).map_err(|e| {
            crate::Error::config("file", &format!("Failed to parse config file: {}", e))
        })?;

        Ok(settings)
    }

    /// Merge settings with environment variable overrides
    pub fn merge_with_env(mut self) -> crate::Result<Self> {
        let env_settings = Self::from_env()?;

        // Merge only non-default values from environment
        if env_settings.server.host != Self::default().server.host {
            self.server.host = env_settings.server.host;
        }

        if env_settings.server.port != Self::default().server.port {
            self.server.port = env_settings.server.port;
        }

        if env_settings.token.ttl_hours != Self::default().token.ttl_hours {
            self.token.ttl_hours = env_settings.token.ttl_hours;
        }

        // Merge proxy settings (always override if present)
        if env_settings.network.https_proxy.is_some() {
            self.network.https_proxy = env_settings.network.https_proxy;
        }
        if env_settings.network.http_proxy.is_some() {
            self.network.http_proxy = env_settings.network.http_proxy;
        }
        if env_settings.network.all_proxy.is_some() {
            self.network.all_proxy = env_settings.network.all_proxy;
        }

        Ok(self)
    }

    /// Get effective proxy URL based on priority
    ///
    /// Corresponds to TypeScript proxy selection logic in session_manager.ts
    pub fn get_proxy_url(&self) -> Option<String> {
        self.network
            .https_proxy
            .as_ref()
            .or(self.network.http_proxy.as_ref())
            .or(self.network.all_proxy.as_ref())
            .cloned()
    }

    /// Validate configuration settings
    pub fn validate(&self) -> crate::Result<()> {
        // Validate server settings
        if self.server.port == 0 {
            return Err(crate::Error::config(
                "port",
                "Invalid server port: cannot be 0",
            ));
        }

        // Validate token settings
        if self.token.ttl_hours == 0 {
            return Err(crate::Error::config(
                "ttl_hours",
                "Invalid token TTL: cannot be 0",
            ));
        }

        // Validate log level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(crate::Error::config(
                    "log_level",
                    &format!("Invalid log level: {}", self.logging.level),
                ));
            }
        }

        // Validate proxy URLs if present
        for (name, proxy_url) in [
            ("https_proxy", &self.network.https_proxy),
            ("http_proxy", &self.network.http_proxy),
            ("all_proxy", &self.network.all_proxy),
        ]
        .iter()
        {
            if let Some(url_str) = proxy_url
                && let Err(e) = url::Url::parse(url_str)
            {
                return Err(crate::Error::config(
                    *name,
                    &format!("Invalid proxy URL '{}': {}", url_str, e),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.host, "::");
        assert_eq!(settings.server.port, 4416);
        assert_eq!(settings.token.ttl_hours, 6);
        assert!(settings.token.enable_cache);
        assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo");
    }

    #[test]
    fn test_settings_creation() {
        let settings = Settings::new();
        assert_eq!(settings.server.port, 4416);
        assert_eq!(settings.network.max_retries, 3);
    }

    #[test]
    fn test_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[server]
host = "localhost"
port = 8080

[token]
ttl_hours = 12
        "#
        )
        .unwrap();

        let settings = Settings::from_file(temp_file.path()).unwrap();
        assert_eq!(settings.server.host, "localhost");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.token.ttl_hours, 12);
    }

    #[test]
    fn test_env_var_override() {
        unsafe {
            std::env::set_var("TOKEN_TTL", "24");
            std::env::set_var("POT_SERVER_PORT", "9000");
        }

        let settings = Settings::from_env().unwrap();
        assert_eq!(settings.token.ttl_hours, 24);
        assert_eq!(settings.server.port, 9000);

        unsafe {
            std::env::remove_var("TOKEN_TTL");
            std::env::remove_var("POT_SERVER_PORT");
        }
    }

    #[test]
    fn test_proxy_priority() {
        let mut settings = Settings::default();
        settings.network.https_proxy = Some("https://proxy1:8080".to_string());
        settings.network.http_proxy = Some("http://proxy2:8080".to_string());
        settings.network.all_proxy = Some("socks5://proxy3:1080".to_string());

        // HTTPS proxy should have highest priority
        assert_eq!(settings.get_proxy_url().unwrap(), "https://proxy1:8080");

        // Remove HTTPS proxy, HTTP should be next
        settings.network.https_proxy = None;
        assert_eq!(settings.get_proxy_url().unwrap(), "http://proxy2:8080");

        // Remove HTTP proxy, ALL_PROXY should be last
        settings.network.http_proxy = None;
        assert_eq!(settings.get_proxy_url().unwrap(), "socks5://proxy3:1080");
    }

    #[test]
    fn test_validation_success() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_port() {
        let mut settings = Settings::default();
        settings.server.port = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_proxy_url() {
        let mut settings = Settings::default();
        settings.network.https_proxy = Some("invalid-url".to_string());
        assert!(settings.validate().is_err());
    }
}
