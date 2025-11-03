//! Configuration loading utilities
//!
//! Provides helper functions for loading configuration from various sources
//! with proper error handling and validation.

use crate::{Result, config::Settings};
use std::path::Path;
use tracing::{debug, info, warn};

/// Configuration loader with multiple source support
#[derive(Debug)]
pub struct ConfigLoader {
    /// Default settings
    defaults: Settings,
}

impl ConfigLoader {
    /// Create new configuration loader
    pub fn new() -> Self {
        Self {
            defaults: Settings::default(),
        }
    }

    /// Get the config file path from BGUTIL_CONFIG environment variable or default location
    ///
    /// Priority:
    /// 1. BGUTIL_CONFIG environment variable
    /// 2. ~/.config/bgutil-pot-provider/config.toml (or platform equivalent)
    pub fn get_config_path() -> Option<std::path::PathBuf> {
        // First try BGUTIL_CONFIG environment variable
        if let Ok(config_path) = std::env::var("BGUTIL_CONFIG") {
            let path = std::path::PathBuf::from(config_path);
            if path.exists() {
                debug!("Using config file from BGUTIL_CONFIG: {:?}", path);
                return Some(path);
            } else {
                warn!("BGUTIL_CONFIG points to non-existent file: {:?}", path);
            }
        }

        // Try default config location
        if let Some(config_dir) = dirs::config_dir() {
            let default_path = config_dir.join("bgutil-pot-provider").join("config.toml");
            if default_path.exists() {
                debug!("Using default config file: {:?}", default_path);
                return Some(default_path);
            }
        }

        debug!("No config file found");
        None
    }

    /// Load configuration with precedence order:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. Default values (lowest priority)
    pub fn load(&self, config_file: Option<&Path>) -> Result<Settings> {
        let mut settings = self.defaults.clone();

        // Load from config file if provided
        if let Some(path) = config_file {
            if path.exists() {
                info!("Loading configuration from file: {:?}", path);
                settings = Settings::from_file(path)?;
            } else {
                warn!("Configuration file not found: {:?}, using defaults", path);
            }
        }

        // Override with environment variables
        debug!("Applying environment variable overrides");
        settings = settings.merge_with_env()?;

        // Validate final configuration
        settings.validate()?;

        info!("Configuration loaded successfully");
        debug!("Final configuration: {:?}", settings);

        Ok(settings)
    }

    /// Load configuration from environment only
    pub fn from_env_only(&self) -> Result<Settings> {
        let settings = Settings::from_env()?;
        settings.validate()?;
        Ok(settings)
    }

    /// Get default configuration
    pub fn defaults(&self) -> &Settings {
        &self.defaults
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    // Static mutex to ensure environment variable tests don't interfere with each other
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_load_defaults() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();

        // Instead of modifying the global environment, test that Settings::default()
        // returns the expected default values
        let default_settings = Settings::default();
        assert_eq!(default_settings.server.port, 4416);
        assert_eq!(default_settings.token.ttl_hours, 6);

        // Test that ConfigLoader correctly uses these defaults
        let loader = ConfigLoader::new();
        let defaults = loader.defaults();
        assert_eq!(defaults.server.port, 4416);
        assert_eq!(defaults.token.ttl_hours, 6);
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

        let loader = ConfigLoader::new();
        let settings = loader.load(Some(temp_file.path())).unwrap();

        assert_eq!(settings.server.host, "localhost");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.token.ttl_hours, 12);
    }

    #[test]
    fn test_env_var_override() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();

        // Save current environment state
        let original_port = std::env::var("POT_SERVER_PORT").ok();
        let original_ttl = std::env::var("TOKEN_TTL").ok();

        // Set test environment variables (still need unsafe for global env modification)
        unsafe {
            std::env::set_var("TOKEN_TTL", "24");
            std::env::set_var("POT_SERVER_PORT", "9000");
        }

        let loader = ConfigLoader::new();
        let settings = loader.from_env_only().unwrap();

        assert_eq!(settings.token.ttl_hours, 24);
        assert_eq!(settings.server.port, 9000);

        // Restore original environment state
        unsafe {
            std::env::remove_var("TOKEN_TTL");
            std::env::remove_var("POT_SERVER_PORT");

            if let Some(port) = original_port {
                std::env::set_var("POT_SERVER_PORT", port);
            }
            if let Some(ttl) = original_ttl {
                std::env::set_var("TOKEN_TTL", ttl);
            }
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
}
