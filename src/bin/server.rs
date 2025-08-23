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

    tracing::info!("Starting POT server on {}:{}", cli.host, cli.port);

    // TODO: Implement server startup logic
    println!("Server implementation to be added");

    Ok(())
}
