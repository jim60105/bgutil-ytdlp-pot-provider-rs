//! Session management for POT token generation
//!
//! This module handles session management, token caching, and the core logic
//! for generating POT tokens using the BgUtils library, including BotGuard
//! integration, Innertube API communication, and network handling.

pub mod botguard;
pub mod innertube;
pub mod manager;
pub mod network;
pub mod webpo_minter;

pub use botguard::{BotGuardManager, SnapshotArgs};
pub use innertube::{InnertubeClient, InnertubeProvider};
pub use manager::{SessionManager, SessionManagerGeneric};
pub use network::{NetworkManager, ProxySpec, RequestOptions};
pub use webpo_minter::{JsRuntimeHandle, WebPoMinter};
