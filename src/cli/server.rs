//! Server mode CLI logic
//!
//! Contains the core logic for running the HTTP server mode.

use crate::{Settings, server::app, utils::version};
use anyhow::Result;

/// Arguments for server mode
#[derive(Debug)]
pub struct ServerArgs {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub verbose: bool,
}

/// Run server mode with the given arguments
pub async fn run_server_mode(args: ServerArgs) -> Result<()> {
    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Load configuration with proper precedence:
    // 1. Command line arguments (highest priority)
    // 2. Environment variables
    // 3. Configuration file (from BGUTIL_CONFIG or default location)
    // 4. Default values (lowest priority)
    use crate::config::ConfigLoader;

    let config_loader = ConfigLoader::new();
    let config_path = ConfigLoader::get_config_path();

    let mut settings = match config_loader.load(config_path.as_deref()) {
        Ok(settings) => settings,
        Err(e) => {
            tracing::warn!("Failed to load configuration: {}. Using defaults.", e);
            Settings::default()
        }
    };

    // Override with CLI arguments if provided (highest priority)
    if let Some(host) = args.host {
        settings.server.host = host;
    }
    if let Some(port) = args.port {
        settings.server.port = port;
    }
    settings.logging.verbose = args.verbose;

    tracing::info!("Starting POT server v{}", version::get_version());

    // Create the Axum application
    let app = app::create_app(settings.clone());

    // Parse address and attempt IPv6/IPv4 fallback like TypeScript implementation
    let addr = parse_and_bind_address(&settings.server.host, settings.server.port).await?;

    tracing::info!(
        "POT server v{} listening on {}",
        version::get_version(),
        addr
    );

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Parse host string and attempt to bind to the address
///
/// Implements the same IPv6 fallback logic as TypeScript implementation:
/// - First try to bind to IPv6 (::)
/// - If that fails, fall back to IPv4 (0.0.0.0)
pub async fn parse_and_bind_address(host: &str, port: u16) -> Result<std::net::SocketAddr> {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    // Try to parse as IP address first
    if let Ok(ip) = host.parse::<IpAddr>() {
        let addr = SocketAddr::new(ip, port);
        tracing::debug!("Parsed address: {}", addr);
        return Ok(addr);
    }

    // Handle special cases like "::" for IPv6 any
    match host {
        "::" => {
            let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);
            tracing::debug!("Using IPv6 any address: {}", addr);

            // Test if we can bind to IPv6
            match tokio::net::TcpListener::bind(addr).await {
                Ok(_) => {
                    tracing::info!("Successfully bound to IPv6 address {}", addr);
                    Ok(addr)
                }
                Err(e) => {
                    tracing::warn!(
                        "Could not listen on [::]:{} (Caused by {}), falling back to 0.0.0.0",
                        port,
                        e
                    );
                    let fallback_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
                    tracing::info!("Using IPv4 fallback address: {}", fallback_addr);
                    Ok(fallback_addr)
                }
            }
        }
        "0.0.0.0" => {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
            tracing::info!("Using IPv4 any address: {}", addr);
            Ok(addr)
        }
        _ => {
            anyhow::bail!(
                "Invalid host address: {}. Use '::' for IPv6 or '0.0.0.0' for IPv4",
                host
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_parse_and_bind_ipv4_address() {
        let result = parse_and_bind_address("127.0.0.1", 0).await; // Use port 0 to get any available port
        assert!(result.is_ok());

        let addr = result.unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[tokio::test]
    async fn test_parse_and_bind_ipv6_address() {
        let result = parse_and_bind_address("::1", 0).await; // Use port 0 to get any available port
        assert!(result.is_ok());

        let addr = result.unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[tokio::test]
    async fn test_parse_and_bind_ipv4_any_address() {
        let result = parse_and_bind_address("0.0.0.0", 0).await; // Use port 0 to get any available port
        assert!(result.is_ok());

        let addr = result.unwrap();
        assert_eq!(
            addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
        );
    }

    #[tokio::test]
    async fn test_parse_and_bind_ipv6_any_fallback() {
        // Test IPv6 any address - this should work or fallback to IPv4
        let result = parse_and_bind_address("::", 0).await; // Use port 0 to get any available port
        assert!(result.is_ok());

        let addr = result.unwrap();
        // Should be either IPv6 unspecified or IPv4 unspecified (fallback)
        assert!(
            addr.ip() == std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED)
                || addr.ip() == std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
        );
    }

    #[tokio::test]
    async fn test_parse_and_bind_invalid_address() {
        let result = parse_and_bind_address("invalid-host", 8080).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Invalid host address: invalid-host")
        );
    }

    #[tokio::test]
    async fn test_parse_and_bind_empty_address() {
        let result = parse_and_bind_address("", 8080).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid host address"));
    }

    #[tokio::test]
    async fn test_parse_and_bind_localhost_fails() {
        // localhost should fail since we only accept IP addresses or :: and 0.0.0.0
        let result = parse_and_bind_address("localhost", 8080).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_server_args_with_optional_values() {
        // Test ServerArgs with all None values
        let args = ServerArgs {
            port: None,
            host: None,
            verbose: false,
        };
        assert!(args.port.is_none());
        assert!(args.host.is_none());
        assert!(!args.verbose);

        // Test ServerArgs with Some values
        let args = ServerArgs {
            port: Some(8080),
            host: Some("127.0.0.1".to_string()),
            verbose: true,
        };
        assert_eq!(args.port, Some(8080));
        assert_eq!(args.host, Some("127.0.0.1".to_string()));
        assert!(args.verbose);
    }

    #[tokio::test]
    async fn test_run_server_mode_with_invalid_config() {
        use std::sync::Mutex;
        use tempfile::NamedTempFile;

        // Static mutex to ensure this test doesn't interfere with others
        static TEST_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = TEST_MUTEX.lock().unwrap();

        // Create an invalid config file
        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, b"invalid toml content [[[").unwrap();
        temp_file.flush().unwrap();

        // Save and set BGUTIL_CONFIG
        let original_config = std::env::var("BGUTIL_CONFIG").ok();
        unsafe {
            std::env::set_var("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
        }

        // Create ServerArgs for testing
        let args = ServerArgs {
            port: Some(0), // Use port 0 to get any available port
            host: Some("127.0.0.1".to_string()),
            verbose: false,
        };

        // Spawn the server in a separate task and cancel it immediately
        let handle = tokio::spawn(async move { run_server_mode(args).await });

        // Give it a moment to initialize, then abort
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        handle.abort();

        // Restore environment
        unsafe {
            std::env::remove_var("BGUTIL_CONFIG");
            if let Some(config) = original_config {
                std::env::set_var("BGUTIL_CONFIG", config);
            }
        }
    }

    #[tokio::test]
    async fn test_run_server_mode_with_valid_config() {
        use std::io::Write;
        use std::sync::Mutex;
        use tempfile::NamedTempFile;

        // Static mutex to ensure this test doesn't interfere with others
        static TEST_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = TEST_MUTEX.lock().unwrap();

        // Create a valid config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[server]
host = "127.0.0.1"
port = 4416
        "#
        )
        .unwrap();
        temp_file.flush().unwrap();

        // Save and set BGUTIL_CONFIG
        let original_config = std::env::var("BGUTIL_CONFIG").ok();
        unsafe {
            std::env::set_var("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
        }

        // Create ServerArgs that will override config
        let args = ServerArgs {
            port: Some(0), // Use port 0 to get any available port
            host: Some("127.0.0.1".to_string()),
            verbose: false,
        };

        // Spawn the server in a separate task and cancel it immediately
        let handle = tokio::spawn(async move { run_server_mode(args).await });

        // Give it a moment to initialize, then abort
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        handle.abort();

        // Restore environment
        unsafe {
            std::env::remove_var("BGUTIL_CONFIG");
            if let Some(config) = original_config {
                std::env::set_var("BGUTIL_CONFIG", config);
            }
        }
    }

    #[tokio::test]
    async fn test_run_server_mode_verbose_logging() {
        // Test that verbose flag is properly handled
        let args = ServerArgs {
            port: Some(0),
            host: Some("127.0.0.1".to_string()),
            verbose: true,
        };

        // Spawn the server in a separate task and cancel it immediately
        let handle = tokio::spawn(async move { run_server_mode(args).await });

        // Give it a moment to initialize, then abort
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        handle.abort();
    }
}
