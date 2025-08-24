//! Script mode binary for one-time POT token generation
//!
//! Generates a single POT token and outputs it to stdout.
//! This mode is used when yt-dlp invokes the provider as a script.
//!
//! # Usage
//!
//! ```bash
//! bgutil-pot-generate --content-binding "video_id"
//! ```
//!
//! # Output
//!
//! Outputs a JSON object containing the POT token:
//! ```json
//! {
//!   "poToken": "generated_token",
//!   "contentBinding": "video_id",
//!   "expiresAt": "2025-01-01T00:00:00Z"
//! }
//! ```
//!
//! Based on TypeScript implementation in `server/src/generate_once.ts`

use clap::Parser;
use tracing::{debug, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bgutil_ytdlp_pot_provider::{
    Result, SessionManager, Settings,
    types::PotRequest,
    utils::{
        VERSION,
        cache::{FileCache, get_cache_path},
    },
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "bgutil-pot-generate")]
#[command(disable_version_flag = true)]
struct Cli {
    /// Content binding (video ID, visitor data, etc.)
    #[arg(short, long, value_name = "CONTENT_BINDING")]
    content_binding: Option<String>,

    /// Visitor data (DEPRECATED: use --content-binding instead)
    #[arg(short = 'v', long, value_name = "VISITOR_DATA")]
    visitor_data: Option<String>,

    /// Data sync ID (DEPRECATED: use --content-binding instead)
    #[arg(short = 'd', long, value_name = "DATA_SYNC_ID")]
    data_sync_id: Option<String>,

    /// Proxy server URL (http://host:port, socks5://host:port, etc.)
    #[arg(short, long, value_name = "PROXY")]
    proxy: Option<String>,

    /// Bypass cache and force new token generation
    #[arg(short, long)]
    bypass_cache: bool,

    /// Source IP address for outbound connections
    #[arg(short, long, value_name = "SOURCE_ADDRESS")]
    source_address: Option<String>,

    /// Disable TLS certificate verification
    #[arg(long)]
    disable_tls_verification: bool,

    /// Show version information
    #[arg(long)]
    version: bool,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle version flag early
    if cli.version {
        println!("{}", VERSION);
        return Ok(());
    }

    // Initialize logging (minimal for script mode)
    if cli.verbose {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "error".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    }

    // Handle deprecated parameters
    if let Some(ref _data_sync_id) = cli.data_sync_id {
        eprintln!("Data sync id is deprecated, use --content-binding instead");
        std::process::exit(1);
    }

    if let Some(ref _visitor_data) = cli.visitor_data {
        eprintln!("Visitor data is deprecated, use --content-binding instead");
        std::process::exit(1);
    }

    debug!(
        "Starting POT generation with parameters: content_binding={:?}, proxy={:?}, bypass_cache={}",
        cli.content_binding, cli.proxy, cli.bypass_cache
    );

    // Initialize file cache
    let cache_path = get_cache_path()?;
    let file_cache = FileCache::new(cache_path);

    // Load existing cache
    let session_data_caches = file_cache.load_cache().await.unwrap_or_else(|e| {
        warn!("Failed to load cache: {}. Starting with empty cache.", e);
        std::collections::HashMap::new()
    });

    // Initialize session manager with cache
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);
    session_manager
        .set_session_data_caches(session_data_caches)
        .await;

    // Build POT request
    let request = build_pot_request(&cli)?;

    // Generate POT token
    match session_manager.generate_pot_token(&request).await {
        Ok(response) => {
            // Save updated cache
            if let Err(e) = file_cache
                .save_cache(session_manager.get_session_data_caches(true).await)
                .await
            {
                warn!("Failed to save cache: {}", e);
            }

            // Output result as JSON
            let output = serde_json::to_string(&response)?;
            println!("{}", output);

            info!(
                "Successfully generated POT token for content binding: {:?}",
                request.content_binding
            );
        }
        Err(e) => {
            eprintln!("Failed while generating POT. Error: {}", e);

            // Output empty JSON on error (matching TypeScript behavior)
            println!("{{}}");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Build POT request from CLI arguments
fn build_pot_request(cli: &Cli) -> Result<PotRequest> {
    let mut request = PotRequest::new();

    if let Some(ref content_binding) = cli.content_binding {
        request = request.with_content_binding(content_binding);
    }

    if let Some(ref proxy) = cli.proxy {
        request = request.with_proxy(proxy);
    }

    if cli.bypass_cache {
        request = request.with_bypass_cache(true);
    }

    if let Some(ref source_address) = cli.source_address {
        request = request.with_source_address(source_address);
    }

    if cli.disable_tls_verification {
        request = request.with_disable_tls_verification(true);
    }

    // Force disable Innertube for script mode (matching TypeScript behavior)
    request = request.with_disable_innertube(true);

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pot_request() {
        let cli = Cli {
            content_binding: Some("test_video_id".to_string()),
            proxy: Some("http://proxy:8080".to_string()),
            bypass_cache: true,
            source_address: Some("192.168.1.100".to_string()),
            disable_tls_verification: true,
            // ... other fields with default values
            visitor_data: None,
            data_sync_id: None,
            version: false,
            verbose: false,
        };

        let request = build_pot_request(&cli).unwrap();

        assert_eq!(request.content_binding, Some("test_video_id".to_string()));
        assert_eq!(request.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(request.bypass_cache, Some(true));
        assert_eq!(request.source_address, Some("192.168.1.100".to_string()));
        assert_eq!(request.disable_tls_verification, Some(true));
        assert_eq!(request.disable_innertube, Some(true)); // Should be forced to true
    }
}
