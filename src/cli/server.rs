//! Server mode CLI logic
//!
//! Contains the core logic for running the HTTP server mode.

use crate::{Settings, server::app, utils::version};
use anyhow::Result;

/// Arguments for server mode
#[derive(Debug)]
pub struct ServerArgs {
    pub port: u16,
    pub host: String,
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

    // Load configuration
    let settings = match Settings::from_env() {
        Ok(mut settings) => {
            // Override with CLI arguments
            settings.server.host = args.host.clone();
            settings.server.port = args.port;
            settings
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load settings from environment: {}. Using defaults.",
                e
            );
            let mut settings = Settings::default();
            settings.server.host = args.host.clone();
            settings.server.port = args.port;
            settings
        }
    };

    tracing::info!("Starting POT server v{}", version::get_version());

    // Create the Axum application
    let app = app::create_app(settings.clone());

    // Parse address and attempt IPv6/IPv4 fallback like TypeScript implementation
    let addr = parse_and_bind_address(&args.host, args.port).await?;

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
}
