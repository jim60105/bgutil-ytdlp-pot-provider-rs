//! Server integration example
//!
//! This example shows how to start an HTTP server and handle POT token requests
//! using the BgUtils POT Provider.

use bgutil_ytdlp_pot_provider::{Settings, server::create_app};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with INFO level
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Create settings with custom configuration
    let mut settings = Settings::default();
    settings.server.host = "127.0.0.1".to_string();
    settings.server.port = 8080;
    settings.token.ttl_hours = 12; // Extended TTL for this example

    // Create the Axum app with settings
    let app = create_app(settings.clone());

    // Bind to the configured address
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = TcpListener::bind(&addr).await?;

    println!("🚀 BgUtils POT Provider server starting...");
    println!("📍 Listening on: http://{}", addr);
    println!("🔍 Health check: http://{}/ping", addr);
    println!("📝 API docs: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs");
    println!();
    println!("Example usage:");
    println!("  curl \"http://{}/get_pot\" \\", addr);
    println!("    -H \"Content-Type: application/json\" \\");
    println!("    -d '{{\"content_binding\": \"dQw4w9WgXcQ\"}}'");
    println!();
    println!("Press Ctrl+C to stop the server.");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
