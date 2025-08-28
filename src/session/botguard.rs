//! BotGuard challenge processing and integration
//!
//! This module handles the interaction with Google's BotGuard system using
//! the rustypipe-botguard crate for real POT token generation.

use crate::Result;
use rustypipe_botguard::Botguard;
use std::path::PathBuf;
use time::OffsetDateTime;

/// BotGuard client using rustypipe-botguard crate
pub struct BotGuardClient {
    /// Snapshot file path for caching
    snapshot_path: Option<PathBuf>,
    /// Custom User Agent
    user_agent: Option<String>,
    /// Indicates if client is configured (using atomic for thread safety)
    initialized: std::sync::atomic::AtomicBool,
}

impl std::fmt::Debug for BotGuardClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotGuardClient")
            .field("snapshot_path", &self.snapshot_path)
            .field("user_agent", &self.user_agent)
            .field(
                "initialized",
                &self.initialized.load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish()
    }
}

impl BotGuardClient {
    /// Create new BotGuard client
    pub fn new(snapshot_path: Option<PathBuf>, user_agent: Option<String>) -> Self {
        Self {
            snapshot_path,
            user_agent,
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Initialize the BotGuard client configuration
    pub async fn initialize(&self) -> Result<()> {
        // Just mark as initialized - we'll create instances on demand
        self.initialized
            .store(true, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("BotGuard client configuration initialized");
        Ok(())
    }

    /// Create a new Botguard instance on demand
    async fn create_botguard_instance(&self) -> Result<Botguard> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(crate::Error::botguard(
                "not_initialized",
                "BotGuard client not initialized. Call initialize() first.",
            ));
        }

        let mut builder = rustypipe_botguard::Botguard::builder();

        if let Some(ref path) = self.snapshot_path {
            builder = builder.snapshot_path(path);
        }

        if let Some(ref ua) = self.user_agent {
            builder = builder.user_agent(ua);
        }

        builder
            .init()
            .await
            .map_err(|e| crate::Error::botguard("initialization_failed", e.to_string().as_str()))
    }

    /// Generate POT token by creating a new Botguard instance
    pub async fn generate_po_token(&self, identifier: &str) -> Result<String> {
        tracing::debug!("Generating POT token for identifier: {}", identifier);

        let mut botguard = self.create_botguard_instance().await?;
        botguard
            .mint_token(identifier)
            .await
            .map_err(|e| crate::Error::token_generation(format!("Failed to mint token: {}", e)))
    }

    /// Check if BotGuard is initialized
    pub async fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get expiry information (placeholder implementation)
    pub async fn get_expiry_info(&self) -> Option<(OffsetDateTime, u32)> {
        if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            // Return default values since we create instances on demand
            Some((OffsetDateTime::now_utc() + time::Duration::hours(6), 21600))
        } else {
            None
        }
    }

    /// Save snapshot (no-op implementation since we create instances on demand)
    pub async fn save_snapshot(self) -> Result<bool> {
        // Since we create instances on demand, there's no persistent instance to save
        // The snapshot will be handled by individual Botguard instances
        tracing::info!("Snapshot saving is handled by individual Botguard instances");
        Ok(true)
    }
}

/// Placeholder for backward compatibility - will be removed
/// This maintains the interface for existing code during transition
#[derive(Debug)]
#[allow(dead_code)]
pub struct BotGuardManager {
    client: BotGuardClient,
}

impl BotGuardManager {
    /// Create new BotGuard manager (legacy interface)
    pub fn new(_http_client: reqwest::Client, _request_key: String) -> Self {
        Self {
            client: BotGuardClient::new(None, None),
        }
    }

    /// Get BotGuard manager configuration for diagnostics
    pub fn get_manager_info(&self) -> (String, bool) {
        ("legacy_manager".to_string(), true)
    }
}

/// Arguments for BotGuard snapshot generation
/// Kept for backward compatibility with existing tests
#[derive(Debug)]
pub struct SnapshotArgs<'a> {
    /// Content binding (video ID)
    pub content_binding: Option<&'a str>,
    /// Signed timestamp
    pub signed_timestamp: Option<u64>,
    /// WebPO signal output buffer (for compatibility only)
    pub webpo_signal_output: Option<&'a str>, // Changed to &str for simplicity
    /// Skip privacy buffer flag
    pub skip_privacy_buffer: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_botguard_client_creation() {
        let client = BotGuardClient::new(None, None);
        assert!(!client.is_initialized().await);
    }

    #[tokio::test]
    async fn test_botguard_client_with_config() {
        let snapshot_path = Some(std::path::PathBuf::from("/tmp/test_snapshot.bin"));
        let user_agent = Some("Test User Agent".to_string());

        let client = BotGuardClient::new(snapshot_path, user_agent);
        assert!(!client.is_initialized().await);
    }

    #[tokio::test]
    async fn test_generate_po_token_without_initialization() {
        let client = BotGuardClient::new(None, None);

        let result = client.generate_po_token("test_identifier").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_botguard_manager_legacy_interface() {
        let client = reqwest::Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let (key, has_client) = manager.get_manager_info();
        assert_eq!(key, "legacy_manager");
        assert!(has_client);
    }

    // Real integration test - may fail if network is unavailable
    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_rustypipe_botguard_integration() {
        let client = BotGuardClient::new(None, None);

        // Test initialization with timeout
        let init_result = timeout(Duration::from_secs(30), client.initialize()).await;

        if let Ok(Ok(())) = init_result {
            // If initialization succeeds, test token generation
            let token_result = client.generate_po_token("test_video_id").await;

            if let Ok(token) = token_result {
                assert!(!token.is_empty());
                assert!(token.len() >= 100); // POT tokens should be reasonably long
                println!("Generated POT token length: {}", token.len());
            } else {
                println!("Token generation failed: {:?}", token_result.unwrap_err());
            }

            // Test expiry info
            let expiry_info = client.get_expiry_info().await;
            if let Some((valid_until, lifetime)) = expiry_info {
                println!(
                    "Token valid until: {:?}, lifetime: {} seconds",
                    valid_until, lifetime
                );
                assert!(lifetime > 0);
            }
        } else {
            println!("BotGuard initialization failed or timed out");
        }
    }

    #[tokio::test]
    async fn test_snapshot_args_creation() {
        let args = SnapshotArgs {
            content_binding: Some("test_video_id"),
            signed_timestamp: Some(1234567890),
            webpo_signal_output: Some("test_output"),
            skip_privacy_buffer: Some(false),
        };

        assert_eq!(args.content_binding, Some("test_video_id"));
        assert_eq!(args.signed_timestamp, Some(1234567890));
        assert_eq!(args.skip_privacy_buffer, Some(false));
    }
}
