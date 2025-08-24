//! HTTP server binary for POT token generation
//!
//! Starts an HTTP server that provides REST API endpoints for generating
//! POT tokens. This is the recommended mode for production deployments.
//!
//! # Usage
//!
//! ```bash
//! bgutil-pot-server --port 4416 --host 0.0.0.0
//! ```
//!
//! # API Endpoints
//!
//! - `POST /get_pot`: Generate a new POT token
//! - `GET /ping`: Health check endpoint
//! - `POST /invalidate_caches`: Clear internal caches

use clap::Parser;

/// HTTP server for POT token generation
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "4416")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "::")]
    host: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Load configuration
    let settings = match bgutil_ytdlp_pot_provider::Settings::from_env() {
        Ok(mut settings) => {
            // Override with CLI arguments
            settings.server.host = cli.host.clone();
            settings.server.port = cli.port;
            settings
        }
        Err(e) => {
            tracing::warn!("Failed to load settings from environment: {}. Using defaults.", e);
            let mut settings = bgutil_ytdlp_pot_provider::Settings::default();
            settings.server.host = cli.host.clone();
            settings.server.port = cli.port;
            settings
        }
    };

    tracing::info!("Starting POT server v{}", bgutil_ytdlp_pot_provider::utils::version::get_version());

    // Create the Axum application
    let app = bgutil_ytdlp_pot_provider::server::app::create_app(settings.clone());

    // Parse address and attempt IPv6/IPv4 fallback like TypeScript implementation
    let addr = parse_and_bind_address(&cli.host, cli.port).await?;

    tracing::info!(
        "POT server v{} listening on {}",
        bgutil_ytdlp_pot_provider::utils::version::get_version(),
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
async fn parse_and_bind_address(host: &str, port: u16) -> anyhow::Result<std::net::SocketAddr> {
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
                    tracing::warn!("Could not listen on [::]:{} (Caused by {}), falling back to 0.0.0.0", port, e);
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
            anyhow::bail!("Invalid host address: {}. Use '::' for IPv6 or '0.0.0.0' for IPv4", host);
        }
    }
}
