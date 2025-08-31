//! BgUtils POT Provider - Rust Implementation
//!
//! A high-performance proof-of-origin token (POT) provider for yt-dlp using the rustypipe-botguard crate.
//! This library provides both HTTP server and script-based modes for generating authentic POT tokens
//! to bypass YouTube's bot detection.
//!
//! # Features
//!
//! - **Real BotGuard Integration**: Uses rustypipe-botguard for authentic POT token generation
//! - **High Performance**: Written in Rust for optimal speed and memory efficiency
//! - **HTTP Server Mode**: Always-running REST API for token generation
//! - **Script Mode**: Command-line interface for single token requests
//! - **Session Management**: Intelligent caching and session handling
//! - **Cross-Platform**: Native support for Linux, Windows, and macOS
//!
//! # Architecture
//!
//! The project consists of two main operation modes:
//! - **HTTP Server Mode**: An always-running REST API service for token generation
//! - **Script Mode**: A command-line tool for one-time token generation
//!
//! # Usage
//!
//! ## HTTP Server Mode
//!
//! ```bash
//! bgutil-pot-server --port 4416 --host 0.0.0.0
//! ```
//!
//! ## Script Mode
//!
//! ```bash
//! bgutil-pot-generate --content-binding "video_id"
//! ```
//!
//! # Examples
//!
//! ```rust
//! use bgutil_ytdlp_pot_provider::{SessionManager, Settings};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let settings = Settings::default();
//! let session_manager = SessionManager::new(settings);
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod server;
pub mod session;
pub mod types;
pub mod utils;

pub use config::{ConfigLoader, Settings};
pub use error::{Error, Result};
pub use session::SessionManager;
pub use types::{ErrorResponse, PingResponse, PotRequest, PotResponse};
