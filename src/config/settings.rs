//! Configuration settings structure
//!
//! Defines the main settings structure and loading logic for the POT provider.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main configuration settings for the POT provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerSettings,
    /// Token configuration
    pub token: TokenSettings,
    /// Logging configuration
    pub logging: LoggingSettings,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Request timeout duration
    pub timeout: Duration,
}

/// Token generation and caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSettings {
    /// Token TTL in hours
    pub ttl_hours: u64,
    /// Enable token caching
    pub enable_cache: bool,
    /// Maximum cache entries
    pub max_cache_entries: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level
    pub level: String,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                host: "::".to_string(),
                port: 4416,
                timeout: Duration::from_secs(30),
            },
            token: TokenSettings {
                ttl_hours: 6,
                enable_cache: true,
                max_cache_entries: 1000,
            },
            logging: LoggingSettings {
                level: "info".to_string(),
                verbose: false,
            },
        }
    }
}

impl Settings {
    /// Create new settings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from environment variables
    pub fn from_env() -> crate::Result<Self> {
        let mut settings = Self::default();

        // Load server settings from environment
        if let Ok(host) = std::env::var("POT_SERVER_HOST") {
            settings.server.host = host;
        }

        if let Ok(port) = std::env::var("POT_SERVER_PORT") {
            settings.server.port = port
                .parse()
                .map_err(|e| crate::Error::Config(format!("Invalid port: {}", e)))?;
        }

        // Load token settings from environment
        if let Ok(ttl) = std::env::var("TOKEN_TTL") {
            settings.token.ttl_hours = ttl
                .parse()
                .map_err(|e| crate::Error::Config(format!("Invalid TTL: {}", e)))?;
        }

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.server.host, "::");
        assert_eq!(settings.server.port, 4416);
        assert_eq!(settings.token.ttl_hours, 6);
        assert!(settings.token.enable_cache);
    }

    #[test]
    fn test_settings_creation() {
        let settings = Settings::new();
        assert_eq!(settings.server.port, 4416);
    }
}
