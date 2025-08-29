//! Configuration example
//!
//! This example demonstrates various ways to configure the BgUtils POT Provider,
//! including loading from environment variables and configuration files.

use bgutil_ytdlp_pot_provider::{Settings, config::ConfigLoader};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().init();

    println!("🔧 BgUtils POT Provider - Configuration Examples");
    println!("================================================");

    // Example 1: Default configuration
    println!("\n1. Default Configuration:");
    let default_settings = Settings::default();
    println!(
        "   Server: {}:{}",
        default_settings.server.host, default_settings.server.port
    );
    println!("   Token TTL: {} hours", default_settings.token.ttl_hours);
    println!(
        "   BotGuard VM timeout: {} seconds",
        default_settings.botguard.vm_timeout
    );

    // Example 2: Environment variable configuration
    println!("\n2. Environment Variable Configuration:");

    // Set some example environment variables
    unsafe {
        env::set_var("POT_SERVER_PORT", "9000");
        env::set_var("TOKEN_TTL", "12");
        env::set_var("RUST_LOG", "debug");
    }

    let config_loader = ConfigLoader::new();
    let env_settings = config_loader.from_env_only()?;

    println!(
        "   Server port (from POT_SERVER_PORT): {}",
        env_settings.server.port
    );
    println!(
        "   Token TTL (from TOKEN_TTL): {} hours",
        env_settings.token.ttl_hours
    );

    // Example 3: Configuration file
    println!("\n3. Configuration File Example:");
    let config_toml = r#"
[server]
host = "0.0.0.0"
port = 4416
timeout = 60

[token]
ttl_hours = 8
enable_cache = true

[botguard]
vm_timeout = 45
disable_innertube = false

[network]
connect_timeout = 30
request_timeout = 120
max_retries = 5

[logging]
level = "info"
verbose = true
"#;

    println!("   Example config.toml content:");
    println!("{}", config_toml);

    // Example 4: Proxy configuration
    println!("\n4. Proxy Configuration:");
    let mut proxy_settings = Settings::default();
    proxy_settings.network.https_proxy = Some("https://proxy.example.com:8080".to_string());
    proxy_settings.network.http_proxy = Some("http://proxy.example.com:8080".to_string());

    if let Some(proxy_url) = proxy_settings.get_proxy_url() {
        println!("   Active proxy: {}", proxy_url);
    }

    // Example 5: BotGuard configuration
    println!("\n5. BotGuard Configuration:");
    println!("   Request key: {}", default_settings.botguard.request_key);
    println!("   VM enabled: {}", default_settings.botguard.enable_vm);
    println!(
        "   Snapshot path: {:?}",
        default_settings.botguard.snapshot_path
    );

    // Clean up environment variables
    unsafe {
        env::remove_var("POT_SERVER_PORT");
        env::remove_var("TOKEN_TTL");
        env::remove_var("RUST_LOG");
    }

    println!("\n✅ Configuration examples completed!");

    Ok(())
}
