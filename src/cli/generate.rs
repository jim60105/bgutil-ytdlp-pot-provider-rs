//! Generate mode CLI logic
//!
//! Contains the core logic for the script mode POT token generation.

use anyhow::Result;
use tracing::{debug, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    SessionManager, Settings,
    types::PotRequest,
    utils::{
        VERSION,
        cache::{FileCache, get_cache_path},
    },
};

/// Arguments for generate mode
#[derive(Debug)]
pub struct GenerateArgs {
    pub content_binding: Option<String>,
    pub visitor_data: Option<String>,
    pub data_sync_id: Option<String>,
    pub proxy: Option<String>,
    pub bypass_cache: bool,
    pub source_address: Option<String>,
    pub disable_tls_verification: bool,
    pub version: bool,
    pub verbose: bool,
}

/// Run generate mode with the given arguments
pub async fn run_generate_mode(args: GenerateArgs) -> Result<()> {
    // Handle version flag early
    if args.version {
        println!("{}", VERSION);
        return Ok(());
    }

    // Initialize logging (minimal for script mode)
    if args.verbose {
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
    if let Some(ref _data_sync_id) = args.data_sync_id {
        eprintln!("Data sync id is deprecated, use --content-binding instead");
        std::process::exit(1);
    }

    if let Some(ref _visitor_data) = args.visitor_data {
        eprintln!("Visitor data is deprecated, use --content-binding instead");
        std::process::exit(1);
    }

    debug!(
        "Starting POT generation with parameters: content_binding={:?}, proxy={:?}, bypass_cache={}",
        args.content_binding, args.proxy, args.bypass_cache
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
    let request = build_pot_request(&args)?;

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

            // Shutdown session manager to properly cleanup V8 isolates
            // This prevents the "v8::OwnedIsolate for snapshot was leaked" warning
            session_manager.shutdown().await;
        }
        Err(e) => {
            // Shutdown session manager before exiting on error
            session_manager.shutdown().await;

            eprintln!("Failed while generating POT. Error: {}", e);

            // Output empty JSON on error (matching TypeScript behavior)
            println!("{{}}");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Build POT request from CLI arguments
fn build_pot_request(args: &GenerateArgs) -> Result<PotRequest> {
    let mut request = PotRequest::new();

    if let Some(ref content_binding) = args.content_binding {
        request = request.with_content_binding(content_binding);
    }

    if let Some(ref proxy) = args.proxy {
        request = request.with_proxy(proxy);
    }

    if args.bypass_cache {
        request = request.with_bypass_cache(true);
    }

    if let Some(ref source_address) = args.source_address {
        request = request.with_source_address(source_address);
    }

    if args.disable_tls_verification {
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
        let args = GenerateArgs {
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

        let request = build_pot_request(&args).unwrap();

        assert_eq!(request.content_binding, Some("test_video_id".to_string()));
        assert_eq!(request.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(request.bypass_cache, Some(true));
        assert_eq!(request.source_address, Some("192.168.1.100".to_string()));
        assert_eq!(request.disable_tls_verification, Some(true));
        assert_eq!(request.disable_innertube, Some(true)); // Should be forced to true
    }
}
