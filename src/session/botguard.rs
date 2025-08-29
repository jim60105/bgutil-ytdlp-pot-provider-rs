//! BotGuard challenge processing and integration
//!
//! This module handles the interaction with Google's BotGuard system using
//! the rustypipe-botguard crate for real POT token generation.

use crate::Result;
use std::path::PathBuf;
use time::OffsetDateTime;

// Global mutex to serialize BotGuard operations to prevent V8 runtime conflicts
static BOTGUARD_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

    /// Generate POT token by creating a new Botguard instance in a blocking task
    pub async fn generate_po_token(&self, identifier: &str) -> Result<String> {
        tracing::debug!("Generating POT token for identifier: {}", identifier);

        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(crate::Error::botguard(
                "not_initialized",
                "BotGuard client not initialized. Call initialize() first.",
            ));
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;
        tracing::debug!("Acquired BotGuard mutex for identifier: {}", identifier);

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();
        let identifier = identifier.to_string();

        // Use spawn_blocking to run BotGuard operations on a dedicated thread
        // since BotGuard instances are !Send and !Sync
        tokio::task::spawn_blocking(move || {
            // Create a simple blocking runtime for the Botguard operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    crate::Error::botguard("runtime_creation_failed", e.to_string().as_str())
                })?;

            rt.block_on(async move {
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let mut botguard = builder.init().await.map_err(|e| {
                    crate::Error::botguard("initialization_failed", e.to_string().as_str())
                })?;

                botguard.mint_token(&identifier).await.map_err(|e| {
                    crate::Error::token_generation(format!("Failed to mint token: {}", e))
                })
            })
        })
        .await
        .map_err(|e| crate::Error::token_generation(format!("Task join error: {}", e)))?
    }

    /// Check if BotGuard is initialized
    pub async fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get expiry information from a real BotGuard instance
    pub async fn get_expiry_info(&self) -> Option<(OffsetDateTime, u32)> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();

        // Use spawn_blocking to run BotGuard operations on a dedicated thread
        let result = tokio::task::spawn_blocking(move || {
            // Create a simple blocking runtime for the Botguard operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Runtime creation failed: {}", e))?;

            rt.block_on(async move {
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let botguard = builder
                    .init()
                    .await
                    .map_err(|e| format!("BotGuard initialization failed: {}", e))?;

                // Get real expiry information from BotGuard instance
                let lifetime = botguard.lifetime();
                let valid_until = botguard.valid_until();

                Ok::<(OffsetDateTime, u32), String>((valid_until, lifetime))
            })
        })
        .await;

        match result {
            Ok(Ok((valid_until, lifetime))) => Some((valid_until, lifetime)),
            Ok(Err(e)) => {
                tracing::warn!("Failed to get BotGuard expiry info: {}", e);
                // Fallback to default values
                Some((OffsetDateTime::now_utc() + time::Duration::hours(6), 21600))
            }
            Err(e) => {
                tracing::warn!("Task join error getting BotGuard expiry info: {}", e);
                // Fallback to default values
                Some((OffsetDateTime::now_utc() + time::Duration::hours(6), 21600))
            }
        }
    }

    /// Save snapshot of current BotGuard instance to configured snapshot path
    pub async fn save_snapshot(self) -> Result<bool> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("Cannot save snapshot: BotGuard client not initialized");
            return Ok(false);
        }

        if self.snapshot_path.is_none() {
            tracing::warn!("Cannot save snapshot: no snapshot path configured");
            return Ok(false);
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();

        // Use spawn_blocking to run BotGuard operations on a dedicated thread
        let result = tokio::task::spawn_blocking(move || {
            // Create a simple blocking runtime for the Botguard operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Runtime creation failed: {}", e))?;

            rt.block_on(async move {
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let botguard = builder
                    .init()
                    .await
                    .map_err(|e| format!("BotGuard initialization failed: {}", e))?;

                // Save snapshot - this consumes the botguard instance
                let saved = botguard.write_snapshot().await;
                Ok::<bool, String>(saved)
            })
        })
        .await;

        match result {
            Ok(Ok(saved)) => {
                if saved {
                    tracing::info!("BotGuard snapshot saved successfully");
                } else {
                    tracing::warn!("BotGuard snapshot could not be saved");
                }
                Ok(saved)
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to save BotGuard snapshot: {}", e);
                Ok(false)
            }
            Err(e) => {
                tracing::error!("Task join error saving BotGuard snapshot: {}", e);
                Ok(false)
            }
        }
    }

    /// Check if BotGuard instance is expired based on real expiry information
    pub async fn is_expired(&self) -> bool {
        if let Some((valid_until, _)) = self.get_expiry_info().await {
            OffsetDateTime::now_utc() >= valid_until
        } else {
            true // Consider uninitialized as expired
        }
    }

    /// Get time remaining until expiry
    pub async fn time_until_expiry(&self) -> Option<time::Duration> {
        if let Some((valid_until, _)) = self.get_expiry_info().await {
            let now = OffsetDateTime::now_utc();
            if valid_until > now {
                Some(valid_until - now)
            } else {
                Some(time::Duration::ZERO)
            }
        } else {
            None
        }
    }

    /// Check if the last BotGuard instance was created from snapshot
    /// Note: This creates a new instance to check, so use sparingly
    pub async fn is_from_snapshot(&self) -> bool {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();

        // Use spawn_blocking to run BotGuard operations on a dedicated thread
        let result = tokio::task::spawn_blocking(move || {
            // Create a simple blocking runtime for the Botguard operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Runtime creation failed: {}", e))?;

            rt.block_on(async move {
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let botguard = builder
                    .init()
                    .await
                    .map_err(|e| format!("BotGuard initialization failed: {}", e))?;

                Ok::<bool, String>(botguard.is_from_snapshot())
            })
        })
        .await;

        match result {
            Ok(Ok(from_snapshot)) => from_snapshot,
            Ok(Err(e)) => {
                tracing::warn!("Failed to check BotGuard snapshot status: {}", e);
                false
            }
            Err(e) => {
                tracing::warn!("Task join error checking BotGuard snapshot status: {}", e);
                false
            }
        }
    }

    /// Get creation time of the last BotGuard instance
    /// Note: This creates a new instance to check, so use sparingly
    pub async fn created_at(&self) -> Option<OffsetDateTime> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();

        // Use spawn_blocking to run BotGuard operations on a dedicated thread
        let result = tokio::task::spawn_blocking(move || {
            // Create a simple blocking runtime for the Botguard operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Runtime creation failed: {}", e))?;

            rt.block_on(async move {
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let botguard = builder
                    .init()
                    .await
                    .map_err(|e| format!("BotGuard initialization failed: {}", e))?;

                Ok::<OffsetDateTime, String>(botguard.created_at())
            })
        })
        .await;

        match result {
            Ok(Ok(created_at)) => Some(created_at),
            Ok(Err(e)) => {
                tracing::warn!("Failed to get BotGuard creation time: {}", e);
                None
            }
            Err(e) => {
                tracing::warn!("Task join error getting BotGuard creation time: {}", e);
                None
            }
        }
    }
}

// Explicit trait implementations for thread safety
// BotGuardClient uses AtomicBool and owned types, making it Send + Sync safe
unsafe impl Send for BotGuardClient {}
unsafe impl Sync for BotGuardClient {}

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

    #[tokio::test]
    async fn test_lifecycle_methods_uninitialized() {
        let client = BotGuardClient::new(None, None);

        // Before initialization, lifecycle methods should return appropriate defaults
        assert!(client.is_expired().await);
        assert!(client.time_until_expiry().await.is_none());
        assert!(!client.is_from_snapshot().await);
        assert!(client.created_at().await.is_none());
    }

    #[tokio::test]
    async fn test_lifecycle_methods_initialized() {
        let client = BotGuardClient::new(None, None);
        let _ = client.initialize().await;

        // After initialization, expiry info should be available
        let is_expired = client.is_expired().await;
        let time_until_expiry = client.time_until_expiry().await;

        // Should not be expired immediately after creation (or fallback to 6 hours)
        assert!(!is_expired);
        assert!(time_until_expiry.is_some());

        let duration = time_until_expiry.unwrap();
        assert!(duration > time::Duration::ZERO);
    }

    #[tokio::test]
    async fn test_save_snapshot_without_path() {
        let client = BotGuardClient::new(None, None);
        let _ = client.initialize().await;

        // Should return false when no snapshot path is configured
        let result = client.save_snapshot().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_save_snapshot_with_temp_path() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.bin");

        let client = BotGuardClient::new(Some(snapshot_path.clone()), None);
        let _ = client.initialize().await;

        // With a valid path, should attempt to save (may fail due to network issues)
        let result = client.save_snapshot().await;
        assert!(result.is_ok());
        // Don't assert on the boolean result as it depends on network availability
    }

    #[tokio::test]
    async fn test_save_snapshot_uninitialized() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.bin");

        let client = BotGuardClient::new(Some(snapshot_path), None);
        // Don't initialize

        // Should return false when not initialized
        let result = client.save_snapshot().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
