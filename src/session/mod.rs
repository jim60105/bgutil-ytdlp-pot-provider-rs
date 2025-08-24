//! Session management for POT token generation
//!
//! This module handles session management, token caching, and the core logic
//! for generating POT tokens using the BgUtils library, including BotGuard 
//! integration, Innertube API communication, and network handling.

pub mod manager;
pub mod botguard;
pub mod innertube;
pub mod network;

pub use manager::SessionManager;
pub use botguard::BotGuardManager;
pub use innertube::InnertubeClient;
pub use network::{NetworkManager, ProxySpec, RequestOptions};
