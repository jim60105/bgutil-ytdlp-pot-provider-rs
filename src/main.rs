//! Unified CLI for BgUtils POT Provider
//!
//! This is the main binary that provides both server and generate modes
//! through a unified command-line interface using subcommands.
//!
//! # Usage
//!
//! ## Server Mode
//! ```bash
//! bgutil-pot server --port 4416 --host 0.0.0.0
//! ```
//!
//! ## Generate Mode
//! ```bash
//! bgutil-pot --content-binding "video_id" --verbose
//! ```
//!
//! ## Help and Version
//! ```bash
//! bgutil-pot --version
//! bgutil-pot --help
//! bgutil-pot server --help
//! ```

use clap::{Parser, Subcommand};

use bgutil_ytdlp_pot_provider::cli::{
    generate::{GenerateArgs, run_generate_mode},
    server::{ServerArgs, run_server_mode},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "bgutil-pot")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Generate mode options (when no subcommand is provided)
    /// Content binding (video ID, visitor data, etc.)
    #[arg(
        short,
        long,
        value_name = "CONTENT_BINDING",
        allow_hyphen_values = true
    )]
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
    #[arg(short = 'b', long)]
    bypass_cache: bool,

    /// Source IP address for outbound connections
    #[arg(short, long, value_name = "SOURCE_ADDRESS")]
    source_address: Option<String>,

    /// Disable TLS certificate verification
    #[arg(long)]
    disable_tls_verification: bool,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server mode
    Server {
        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to
        #[arg(long)]
        host: Option<String>,

        /// Configuration file path
        #[arg(long)]
        config: Option<String>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Server {
            port,
            host,
            config,
            verbose,
        }) => {
            // Server mode logic
            let args = ServerArgs {
                port,
                host,
                config,
                verbose,
            };
            run_server_mode(args).await
        }
        None => {
            // Generate mode logic (default when no subcommand)
            let args = GenerateArgs {
                content_binding: cli.content_binding,
                visitor_data: cli.visitor_data,
                data_sync_id: cli.data_sync_id,
                proxy: cli.proxy,
                bypass_cache: cli.bypass_cache,
                source_address: cli.source_address,
                disable_tls_verification: cli.disable_tls_verification,
                version: false, // Version is handled by clap itself
                verbose: cli.verbose,
            };
            run_generate_mode(args).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_server_subcommand() {
        let cli = Cli::parse_from(&[
            "bgutil-pot",
            "server",
            "--port",
            "8080",
            "--host",
            "0.0.0.0",
        ]);

        match cli.command {
            Some(Commands::Server {
                port, host, config, ..
            }) => {
                assert_eq!(port, Some(8080));
                assert_eq!(host, Some("0.0.0.0".to_string()));
                assert_eq!(config, None);
            }
            _ => panic!("Expected server subcommand"),
        }
    }

    #[test]
    fn test_generate_mode() {
        let cli = Cli::parse_from(&["bgutil-pot", "--content-binding", "test", "--verbose"]);

        assert!(cli.command.is_none());
        assert_eq!(cli.content_binding, Some("test".to_string()));
        assert!(cli.verbose);
    }

    #[test]
    fn test_parameter_conflicts() {
        // Test that clap prevents server subcommand from accepting generate arguments
        let result = Cli::try_parse_from(&["bgutil-pot", "server", "--content-binding", "test"]);

        // Should fail due to clap structure preventing invalid arguments
        assert!(result.is_err());
        // clap error types have their own formatting
    }

    #[test]
    fn test_server_default_values() {
        let cli = Cli::parse_from(&["bgutil-pot", "server"]);

        match cli.command {
            Some(Commands::Server {
                port,
                host,
                config,
                verbose,
            }) => {
                assert_eq!(port, None);
                assert_eq!(host, None);
                assert_eq!(config, None);
                assert!(!verbose);
            }
            _ => panic!("Expected server subcommand"),
        }
    }

    #[test]
    fn test_server_config_option() {
        let cli = Cli::parse_from(&["bgutil-pot", "server", "--config", "/path/to/config.toml"]);

        match cli.command {
            Some(Commands::Server { config, .. }) => {
                assert_eq!(config, Some("/path/to/config.toml".to_string()));
            }
            _ => panic!("Expected server subcommand"),
        }
    }

    #[test]
    fn test_generate_default_values() {
        let cli = Cli::parse_from(&["bgutil-pot"]);

        assert!(cli.command.is_none());
        assert!(cli.content_binding.is_none());
        assert!(!cli.bypass_cache);
        assert!(!cli.verbose);
    }

    #[test]
    fn test_content_binding_with_dash_prefix() {
        // Test video ID starting with dash (e.g., YouTube video ID -6OjhRWNLfk)
        let cli = Cli::parse_from(&["bgutil-pot", "-c", "-6OjhRWNLfk"]);

        assert!(cli.command.is_none());
        assert_eq!(cli.content_binding, Some("-6OjhRWNLfk".to_string()));
    }

    #[test]
    fn test_content_binding_with_dash_prefix_long_form() {
        // Test with long form argument
        let cli = Cli::parse_from(&["bgutil-pot", "--content-binding", "-6OjhRWNLfk"]);

        assert!(cli.command.is_none());
        assert_eq!(cli.content_binding, Some("-6OjhRWNLfk".to_string()));
    }
}
