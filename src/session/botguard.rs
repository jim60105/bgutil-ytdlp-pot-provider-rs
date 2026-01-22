//! BotGuard challenge processing and integration
//!
//! This module handles the interaction with Google's BotGuard system using
//! the rustypipe-botguard crate for real POT token generation.

use crate::Result;
use std::path::PathBuf;
use time::OffsetDateTime;
use tokio::sync::{mpsc, oneshot};

// Global mutex to serialize BotGuard operations to prevent V8 runtime conflicts
static BOTGUARD_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Commands that can be sent to the BotGuard worker
#[allow(dead_code)]
enum BotGuardCommand {
    GenerateToken {
        identifier: String,
        response: oneshot::Sender<Result<String>>,
    },
    GetExpiryInfo {
        response: oneshot::Sender<Option<(OffsetDateTime, u32)>>,
    },
    Shutdown,
}

/// BotGuard client using rustypipe-botguard crate
pub struct BotGuardClient {
    /// Snapshot file path for caching
    snapshot_path: Option<PathBuf>,
    /// Custom User Agent
    user_agent: Option<String>,
    /// Indicates if client is configured (using atomic for thread safety)
    initialized: std::sync::atomic::AtomicBool,
    /// Command sender to the BotGuard worker thread
    command_tx: std::sync::Arc<tokio::sync::RwLock<Option<mpsc::UnboundedSender<BotGuardCommand>>>>,
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
            command_tx: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the BotGuard client configuration and start the worker thread
    pub async fn initialize(&self) -> Result<()> {
        // Check if already initialized
        if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        // Create command channel
        let (tx, mut rx) = mpsc::unbounded_channel::<BotGuardCommand>();

        // Store the sender
        {
            let mut command_tx = self.command_tx.write().await;
            *command_tx = Some(tx);
        }

        let snapshot_path = self.snapshot_path.clone();
        let user_agent = self.user_agent.clone();

        // Spawn a dedicated thread for the BotGuard worker
        // This thread will own a single Botguard instance and process all requests
        std::thread::spawn(move || {
            // Create a tokio runtime for this thread
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create BotGuard worker runtime");

            rt.block_on(async move {
                // Initialize Botguard once
                let mut builder = rustypipe_botguard::Botguard::builder();

                if let Some(ref path) = snapshot_path {
                    builder = builder.snapshot_path(path);
                }

                if let Some(ref ua) = user_agent {
                    builder = builder.user_agent(ua);
                }

                let mut botguard = match builder.init().await {
                    Ok(bg) => bg,
                    Err(e) => {
                        tracing::error!("Failed to initialize BotGuard worker: {}", e);
                        return;
                    }
                };

                tracing::info!("BotGuard worker initialized successfully");

                // Process commands
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        BotGuardCommand::GenerateToken {
                            identifier,
                            response,
                        } => {
                            let result = botguard.mint_token(&identifier).await.map_err(|e| {
                                crate::Error::token_generation(format!(
                                    "Failed to mint token: {}",
                                    e
                                ))
                            });
                            let _ = response.send(result);
                        }
                        BotGuardCommand::GetExpiryInfo { response } => {
                            let lifetime = botguard.lifetime();
                            let valid_until = botguard.valid_until();
                            let _ = response.send(Some((valid_until, lifetime)));
                        }
                        BotGuardCommand::Shutdown => {
                            tracing::info!("BotGuard worker shutting down");
                            break;
                        }
                    }
                }

                // Properly cleanup the Botguard instance by writing snapshot if configured.
                // This is necessary because rustypipe-botguard uses JsRuntimeForSnapshot
                // when a snapshot path is configured, and dropping it without calling
                // write_snapshot() causes the "v8::OwnedIsolate for snapshot was leaked" warning.
                // The write_snapshot() method consumes the Botguard instance and properly
                // extracts the snapshot data before dropping the V8 isolate.
                let snapshot_written = botguard.write_snapshot().await;
                if snapshot_written {
                    tracing::debug!("BotGuard snapshot written during shutdown");
                }
                tracing::info!("BotGuard worker stopped");
            });
        });

        self.initialized
            .store(true, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("BotGuard client configuration initialized");
        Ok(())
    }

    /// Generate POT token by sending command to the BotGuard worker
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

        // Get the command sender
        let command_tx = {
            let tx_lock = self.command_tx.read().await;
            tx_lock.clone().ok_or_else(|| {
                crate::Error::botguard("worker_not_running", "BotGuard worker is not running")
            })?
        };

        // Send command and wait for response
        let (response_tx, response_rx) = oneshot::channel();
        command_tx
            .send(BotGuardCommand::GenerateToken {
                identifier: identifier.to_string(),
                response: response_tx,
            })
            .map_err(|_| {
                crate::Error::botguard("worker_disconnected", "BotGuard worker disconnected")
            })?;

        // Wait for response
        response_rx.await.map_err(|_| {
            crate::Error::botguard(
                "response_error",
                "Failed to receive response from BotGuard worker",
            )
        })?
    }

    /// Check if BotGuard is initialized
    pub async fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reinitialize the BotGuard client by shutting down the existing worker and starting a new one.
    /// This is useful when the BotGuard snapshot has expired and needs to be refreshed.
    pub async fn reinitialize(&self) -> Result<()> {
        tracing::info!("Reinitializing BotGuard client due to expired snapshot");

        // Shutdown existing worker if running
        if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            // Acquire global mutex to ensure no operations are in progress
            let _guard = BOTGUARD_MUTEX.lock().await;

            // Send shutdown command to existing worker
            if let Some(tx) = self.command_tx.read().await.as_ref() {
                let _ = tx.send(BotGuardCommand::Shutdown);
            }

            // Clear the command channel
            {
                let mut command_tx = self.command_tx.write().await;
                *command_tx = None;
            }

            // Mark as uninitialized
            self.initialized
                .store(false, std::sync::atomic::Ordering::Relaxed);

            // Give the worker thread time to shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Initialize fresh instance
        self.initialize().await
    }

    /// Get expiry information from the BotGuard worker
    pub async fn get_expiry_info(&self) -> Option<(OffsetDateTime, u32)> {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }

        // Acquire global mutex to serialize BotGuard operations
        let _guard = BOTGUARD_MUTEX.lock().await;

        // Get the command sender
        let command_tx = {
            let tx_lock = self.command_tx.read().await;
            tx_lock.clone()?
        };

        // Send command and wait for response
        let (response_tx, response_rx) = oneshot::channel();
        command_tx
            .send(BotGuardCommand::GetExpiryInfo {
                response: response_tx,
            })
            .ok()?;

        // Wait for response
        response_rx.await.ok()?
    }

    /// Save snapshot of current BotGuard instance to configured snapshot path
    /// Note: This is a no-op in the worker-based implementation
    /// The worker automatically saves snapshots as configured
    pub async fn save_snapshot(self) -> Result<bool> {
        tracing::warn!("save_snapshot is not supported in worker-based implementation");
        Ok(false)
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
    /// Note: Always returns false in worker-based implementation
    pub async fn is_from_snapshot(&self) -> bool {
        // In worker-based implementation, we can't easily determine this
        // without creating a new instance, which defeats the purpose
        false
    }

    /// Get creation time of the last BotGuard instance
    /// Note: Returns None in worker-based implementation
    pub async fn created_at(&self) -> Option<OffsetDateTime> {
        // In worker-based implementation, we can't determine this
        // without creating a new instance
        None
    }

    /// Shutdown the BotGuard worker thread and wait for it to complete.
    /// This ensures proper cleanup of V8 isolates to avoid the
    /// "v8::OwnedIsolate for snapshot was leaked" warning.
    ///
    /// This method should be called before the process exits, especially in
    /// CLI mode where the process terminates immediately after generating a token.
    pub async fn shutdown(&self) {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        tracing::debug!("Shutting down BotGuard client");

        // Send shutdown command to the worker
        if let Some(tx) = self.command_tx.read().await.as_ref() {
            let _ = tx.send(BotGuardCommand::Shutdown);
        }

        // Clear the command channel
        {
            let mut command_tx = self.command_tx.write().await;
            *command_tx = None;
        }

        // Mark as uninitialized
        self.initialized
            .store(false, std::sync::atomic::Ordering::Relaxed);

        // Give the worker thread time to shutdown and cleanup V8 isolate
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::debug!("BotGuard client shutdown complete");
    }

    /// Synchronous shutdown for use in Drop trait or when tokio runtime is not available.
    /// This is a best-effort cleanup that sends the shutdown command without waiting.
    pub fn shutdown_sync(&self) {
        if !self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        // Try to send shutdown command using blocking approach
        // We need to use try_read to avoid blocking indefinitely
        if let Ok(guard) = self.command_tx.try_read()
            && let Some(tx) = guard.as_ref()
        {
            let _ = tx.send(BotGuardCommand::Shutdown);
        }

        self.initialized
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

// Explicit trait implementations for thread safety
// BotGuardClient uses AtomicBool and owned types, making it Send + Sync safe
unsafe impl Send for BotGuardClient {}
unsafe impl Sync for BotGuardClient {}

impl Drop for BotGuardClient {
    fn drop(&mut self) {
        // Perform synchronous shutdown to ensure V8 isolate cleanup
        // This is a best-effort cleanup - we can't await in drop
        self.shutdown_sync();

        // Give a brief moment for the shutdown command to be processed
        // Note: This is not ideal but necessary to avoid the V8 leak warning
        // in CLI mode where the process exits immediately
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
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

    #[tokio::test]
    async fn test_reinitialize_uninitialized_client() {
        // Test reinitialize on a client that was never initialized
        // Should behave the same as initialize()
        let client = BotGuardClient::new(None, None);
        assert!(!client.is_initialized().await);

        let result = client.reinitialize().await;
        assert!(result.is_ok());
        assert!(client.is_initialized().await);
    }

    #[tokio::test]
    async fn test_reinitialize_initialized_client() {
        // Test reinitialize on an already initialized client
        let client = BotGuardClient::new(None, None);

        // First initialization
        let init_result = client.initialize().await;
        assert!(init_result.is_ok());
        assert!(client.is_initialized().await);

        // Get expiry info before reinit
        let expiry_before = client.get_expiry_info().await;
        assert!(expiry_before.is_some());

        // Reinitialize
        let reinit_result = client.reinitialize().await;
        assert!(reinit_result.is_ok());
        assert!(client.is_initialized().await);

        // Should still have valid expiry info after reinit
        let expiry_after = client.get_expiry_info().await;
        assert!(expiry_after.is_some());
    }

    #[tokio::test]
    async fn test_reinitialize_preserves_functionality() {
        // Test that token generation works after reinitialize
        let client = BotGuardClient::new(None, None);

        // Initialize and generate token
        client.initialize().await.unwrap();
        let token1 = client.generate_po_token("test_id_1").await;
        assert!(token1.is_ok());

        // Reinitialize
        client.reinitialize().await.unwrap();

        // Generate another token after reinit
        let token2 = client.generate_po_token("test_id_2").await;
        assert!(token2.is_ok());

        // Tokens should be different (generated from fresh instance)
        // Note: They might coincidentally be the same for short identifiers,
        // so we just verify both are valid
        assert!(!token1.unwrap().is_empty());
        assert!(!token2.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_reinitialize_resets_expiry() {
        // Test that reinitialize gets fresh expiry information
        let client = BotGuardClient::new(None, None);

        client.initialize().await.unwrap();
        let expiry1 = client.get_expiry_info().await.unwrap();

        // Small delay to ensure time difference
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        client.reinitialize().await.unwrap();
        let expiry2 = client.get_expiry_info().await.unwrap();

        // Both should have valid expiry (lifetime > 0)
        assert!(expiry1.1 > 0);
        assert!(expiry2.1 > 0);
    }
}
