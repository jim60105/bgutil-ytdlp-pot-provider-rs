//! Basic usage example for the BgUtils POT Provider
//!
//! This example demonstrates how to use the SessionManager to generate POT tokens
//! using the rustypipe-botguard integration.

use bgutil_ytdlp_pot_provider::{SessionManager, Settings, types::PotRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    // Create default settings
    let settings = Settings::default();

    // Create session manager
    let session_manager = SessionManager::new(settings);

    // Create a request for token generation
    let request = PotRequest::new().with_content_binding("dQw4w9WgXcQ");

    // Generate POT token
    match session_manager.generate_pot_token(&request).await {
        Ok(response) => {
            println!("Successfully generated POT token:");
            println!("Token: {}", response.po_token);
            println!("Expires at: {:?}", response.expires_at);
            println!("Token length: {} characters", response.po_token.len());
        }
        Err(e) => {
            eprintln!("Failed to generate POT token: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
