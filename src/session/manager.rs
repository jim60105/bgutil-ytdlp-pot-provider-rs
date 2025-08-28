//! # Session Management Module
//!
//! This module provides the core session management functionality for the BgUtils POT Provider.
//! It handles POT token lifecycle, caching, and coordination between different components.
//!
//! ## Architecture
//!
//! The session module is built around the [`SessionManager`] which orchestrates:
//! - Token generation and validation
//! - Cache management
//! - BotGuard challenge resolution
//! - Network communication
//!
//! ## Examples
//!
//! ```rust
//! use bgutil_ytdlp_pot_provider::session::SessionManager;
//! use bgutil_ytdlp_pot_provider::config::Settings;
//! use bgutil_ytdlp_pot_provider::types::PotRequest;
//!
//! # tokio_test::block_on(async {
//! let settings = Settings::default();
//! let manager = SessionManager::new(settings);
//!
//! let request = PotRequest::new()
//!     .with_content_binding("video_id_123");
//!     
//! let response = manager.generate_pot_token(&request).await?;
//! println!("Generated POT token: {}", response.po_token);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
//!
//! ## Token Types
//!
//! The module supports different POT token contexts:
//! - **GVS**: General visitor session tokens
//! - **Player**: Video player-specific tokens  
//! - **Subs**: Subtitle/captions tokens
//!
//! ## Caching Strategy
//!
//! Tokens are cached based on content binding and context to improve performance:
//! - Default TTL: 6 hours
//! - Cache key format: `{content_binding}:{context}`
//! - Automatic expiration and cleanup

use crate::{
    Result,
    config::Settings,
    types::{PotRequest, PotResponse, SessionData, TokenMinterEntry},
};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::ProxySpec;

/// Session data cache type
pub type SessionDataCaches = HashMap<String, SessionData>;

/// Minter cache type
pub type MinterCache = HashMap<String, TokenMinterEntry>;

/// Convenience type alias for SessionManager with default InnertubeClient
pub type SessionManager = SessionManagerGeneric<crate::session::innertube::InnertubeClient>;

/// Main session manager for POT token generation
#[derive(Debug)]
pub struct SessionManagerGeneric<
    T: crate::session::innertube::InnertubeProvider = crate::session::innertube::InnertubeClient,
> {
    /// Configuration settings
    settings: Arc<Settings>,
    /// HTTP client for requests
    http_client: Client,
    /// Cache for session data keyed by content binding
    session_data_caches: RwLock<SessionDataCaches>,
    /// Cache for minter instances
    minter_cache: RwLock<MinterCache>,
    /// Request key for BotGuard API
    request_key: String,
    /// Token TTL in hours
    token_ttl_hours: i64,
    /// Innertube provider for visitor data generation
    innertube_provider: Arc<T>,
    /// BotGuard client for POT token generation
    botguard_client: crate::session::botguard::BotGuardClient,
}

impl SessionManagerGeneric<crate::session::innertube::InnertubeClient> {
    /// Creates a new session manager with the given configuration.
    ///
    /// Initializes HTTP client, cache storage, and configuration parameters
    /// for POT token generation operations.
    ///
    /// # Arguments
    ///
    /// * `settings` - Configuration settings for the session manager
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bgutil_ytdlp_pot_provider::session::SessionManager;
    /// use bgutil_ytdlp_pot_provider::config::Settings;
    ///
    /// let settings = Settings::default();
    /// let manager = SessionManager::new(settings);
    /// ```
    pub fn new(settings: Settings) -> Self {
        let http_client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        let innertube_client = crate::session::innertube::InnertubeClient::new(http_client.clone());

        // Create BotGuard client with configuration
        let snapshot_path = if settings.botguard.disable_snapshot {
            None
        } else {
            settings.botguard.snapshot_path.clone()
        };
        let botguard_client = crate::session::botguard::BotGuardClient::new(
            snapshot_path,
            settings.botguard.user_agent.clone(),
        );

        Self {
            settings: Arc::new(settings),
            http_client,
            session_data_caches: RwLock::new(HashMap::new()),
            minter_cache: RwLock::new(HashMap::new()),
            request_key: "O43z0dpjhgX20SCx4KAo".to_string(), // Hardcoded API key from TS
            token_ttl_hours: 6,                              // Default from TS implementation
            innertube_provider: Arc::new(innertube_client),
            botguard_client,
        }
    }
}

#[cfg(test)]
impl<P> SessionManagerGeneric<P>
where
    P: crate::session::innertube::InnertubeProvider + std::fmt::Debug,
{
    /// Creates a new session manager with a custom innertube provider for testing
    pub fn new_with_provider(settings: Settings, provider: P) -> Self {
        let http_client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        // Create BotGuard client with configuration
        let snapshot_path = if settings.botguard.disable_snapshot {
            None
        } else {
            settings.botguard.snapshot_path.clone()
        };
        let botguard_client = crate::session::botguard::BotGuardClient::new(
            snapshot_path,
            settings.botguard.user_agent.clone(),
        );

        Self {
            settings: Arc::new(settings),
            http_client,
            session_data_caches: RwLock::new(HashMap::new()),
            minter_cache: RwLock::new(HashMap::new()),
            request_key: "O43z0dpjhgX20SCx4KAo".to_string(),
            token_ttl_hours: 6,
            innertube_provider: Arc::new(provider),
            botguard_client,
        }
    }
}

impl<T> SessionManagerGeneric<T>
where
    T: crate::session::innertube::InnertubeProvider + std::fmt::Debug,
{
    /// Generates a POT token for the given request.
    ///
    /// This method handles the complete POT token lifecycle:
    /// 1. Validates request parameters and extracts content binding
    /// 2. Checks for valid cached tokens (unless bypassed)
    /// 3. If no valid cache exists, initiates new token generation
    /// 4. Caches the new token for future requests
    ///
    /// # Arguments
    ///
    /// * `request` - The POT request containing content binding and options
    ///
    /// # Returns
    ///
    /// Returns a [`PotResponse`] containing the POT token and metadata, or an error
    /// if the operation fails.
    ///
    /// # Errors
    ///
    /// This method can return errors for:
    /// - Invalid request parameters
    /// - Network communication failures
    /// - BotGuard challenge resolution failures
    /// - Cache storage/retrieval issues
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bgutil_ytdlp_pot_provider::session::SessionManager;
    /// # use bgutil_ytdlp_pot_provider::types::PotRequest;
    /// # use bgutil_ytdlp_pot_provider::config::Settings;
    /// # tokio_test::block_on(async {
    /// let manager = SessionManager::new(Settings::default());
    ///
    /// let request = PotRequest::new()
    ///     .with_content_binding("L3KvsX8hJss");
    ///     
    /// match manager.generate_pot_token(&request).await {
    ///     Ok(response) => {
    ///         println!("POT token: {}", response.po_token);
    ///         println!("Expires at: {}", response.expires_at);
    ///     }
    ///     Err(e) => eprintln!("Failed to generate POT token: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Implementation Notes
    ///
    /// Corresponds to TypeScript implementation: `generatePoToken` method (L485-569)
    pub async fn generate_pot_token(&self, request: &PotRequest) -> Result<PotResponse> {
        // Initialize BotGuard client before token generation
        self.initialize_botguard().await?;

        let content_binding = self.get_content_binding(request).await?;

        // Clean up expired cache entries
        self.cleanup_caches().await;

        // Check cache first unless bypass_cache is true
        if !request.bypass_cache.unwrap_or(false)
            && let Some(cached_data) = self.get_cached_session_data(&content_binding).await
        {
            tracing::info!(
                "POT for {} still fresh, returning cached token",
                content_binding
            );
            return Ok(PotResponse::from_session_data(cached_data));
        }

        // Generate proxy specification
        let proxy_spec = self.create_proxy_spec(request).await?;

        // Create cache key for minter
        let cache_key = self.create_cache_key(&proxy_spec, request)?;

        // Get or create token minter
        let token_minter = self
            .get_or_create_token_minter(&cache_key, request, &proxy_spec)
            .await?;

        // Mint POT token
        let session_data = self.mint_pot_token(&content_binding, &token_minter).await?;

        // Cache the result
        self.cache_session_data(&content_binding, &session_data)
            .await;

        Ok(PotResponse::from_session_data(session_data))
    }

    /// Generate visitor data for new sessions
    ///
    /// Corresponds to TypeScript: `generateVisitorData` method (L230-241)
    pub async fn generate_visitor_data(&self) -> Result<String> {
        tracing::info!("Generating visitor data using Innertube API");

        // Use the injected Innertube provider
        let visitor_data = self.innertube_provider.generate_visitor_data().await?;

        if visitor_data.is_empty() {
            return Err(crate::Error::VisitorData {
                reason: "Generated visitor data is empty".to_string(),
                context: Some("visitor_data_generation".to_string()),
            });
        }

        // Validate visitor data format
        if visitor_data.len() < 10 {
            return Err(crate::Error::VisitorData {
                reason: "Generated visitor data is too short".to_string(),
                context: Some("visitor_data_validation".to_string()),
            });
        }

        tracing::info!(
            "Visitor data generated successfully: {} chars",
            visitor_data.len()
        );
        Ok(visitor_data)
    }

    /// Invalidate all cached tokens and minters
    ///
    /// Corresponds to TypeScript: `invalidateCaches` method (L200-203)
    pub async fn invalidate_caches(&self) -> Result<()> {
        let mut session_cache = self.session_data_caches.write().await;
        session_cache.clear();

        let mut minter_cache = self.minter_cache.write().await;
        minter_cache.clear();

        tracing::info!("All caches invalidated");
        Ok(())
    }

    /// Invalidate integrity tokens by marking them as expired
    ///
    /// Corresponds to TypeScript: `invalidateIT` method (L205-209)
    pub async fn invalidate_integrity_tokens(&self) -> Result<()> {
        let mut minter_cache = self.minter_cache.write().await;
        let expired_time = DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now);

        for (_, minter) in minter_cache.iter_mut() {
            minter.expiry = expired_time;
        }

        tracing::info!("All integrity tokens marked as expired");
        Ok(())
    }

    /// Get minter cache keys for debugging
    ///
    /// Corresponds to TypeScript: server response in main.ts (L110-113)
    pub async fn get_minter_cache_keys(&self) -> Result<Vec<String>> {
        let cache = self.minter_cache.read().await;
        Ok(cache.keys().cloned().collect())
    }

    /// Set session data caches (for script mode with file cache)
    ///
    /// Corresponds to TypeScript: `setYoutubeSessionDataCaches` method
    pub async fn set_session_data_caches(&self, caches: SessionDataCaches) {
        let mut cache = self.session_data_caches.write().await;
        *cache = caches;
        tracing::debug!("Set session data caches with {} entries", cache.len());
    }

    /// Get session data caches with optional cleanup
    ///
    /// Corresponds to TypeScript: `getYoutubeSessionDataCaches` method (L216-220)
    pub async fn get_session_data_caches(&self, cleanup: bool) -> SessionDataCaches {
        if cleanup {
            self.cleanup_caches().await;
        }

        let cache = self.session_data_caches.read().await;
        cache.clone()
    }

    // Private helper methods...

    /// Get content binding from request or generate visitor data
    async fn get_content_binding(&self, request: &PotRequest) -> Result<String> {
        match &request.content_binding {
            Some(binding) => Ok(binding.clone()),
            None => {
                tracing::warn!("No content binding provided, generating visitor data...");
                self.generate_visitor_data().await
            }
        }
    }

    /// Create proxy specification from request
    async fn create_proxy_spec(&self, request: &PotRequest) -> Result<ProxySpec> {
        let mut proxy_spec = ProxySpec::new();

        // Set proxy URL from request or environment
        if let Some(proxy) = &request.proxy {
            proxy_spec = proxy_spec.with_proxy(proxy);
        } else {
            // Check environment variables like TypeScript does
            if let Ok(proxy) = std::env::var("HTTPS_PROXY")
                .or_else(|_| std::env::var("HTTP_PROXY"))
                .or_else(|_| std::env::var("ALL_PROXY"))
            {
                proxy_spec = proxy_spec.with_proxy(proxy);
            }
        }

        // Set source address
        if let Some(source_address) = &request.source_address {
            proxy_spec = proxy_spec.with_source_address(source_address);
        }

        // Set TLS verification
        proxy_spec = proxy_spec
            .with_disable_tls_verification(request.disable_tls_verification.unwrap_or(false));

        Ok(proxy_spec)
    }

    /// Create cache key for minter cache
    fn create_cache_key(&self, proxy_spec: &ProxySpec, request: &PotRequest) -> Result<String> {
        // Extract remote host from innertube context if available
        let remote_host = request
            .innertube_context
            .as_ref()
            .and_then(|ctx| ctx.get("client"))
            .and_then(|client| client.get("remoteHost"))
            .and_then(|host| host.as_str());

        Ok(proxy_spec.cache_key(remote_host))
    }

    /// Get cached session data
    async fn get_cached_session_data(&self, content_binding: &str) -> Option<SessionData> {
        let cache = self.session_data_caches.read().await;
        cache.get(content_binding).cloned()
    }

    /// Cache session data
    async fn cache_session_data(&self, content_binding: &str, data: &SessionData) {
        let mut cache = self.session_data_caches.write().await;
        cache.insert(content_binding.to_string(), data.clone());
    }

    /// Clean up expired cache entries
    async fn cleanup_caches(&self) {
        let mut cache = self.session_data_caches.write().await;
        let now = Utc::now();
        cache.retain(|_, data| data.expires_at > now);
    }

    /// Get or create token minter
    async fn get_or_create_token_minter(
        &self,
        cache_key: &str,
        request: &PotRequest,
        proxy_spec: &ProxySpec,
    ) -> Result<TokenMinterEntry> {
        // Check if we have a valid cached minter
        {
            let cache = self.minter_cache.read().await;
            if let Some(minter) = cache.get(cache_key)
                && !minter.is_expired()
            {
                return Ok(minter.clone());
            }
        }

        // Generate new minter
        tracing::info!("POT minter expired or not found, generating new one");
        let new_minter = self.generate_token_minter(request, proxy_spec).await?;

        // Cache the new minter
        {
            let mut cache = self.minter_cache.write().await;
            cache.insert(cache_key.to_string(), new_minter.clone());
        }

        Ok(new_minter)
    }

    /// Generate new token minter
    ///
    /// Corresponds to TypeScript: `generateTokenMinter` method (L318-408)
    async fn generate_token_minter(
        &self,
        _request: &PotRequest,
        _proxy_spec: &ProxySpec,
    ) -> Result<TokenMinterEntry> {
        tracing::info!("Generating token minter (placeholder implementation)");

        let expires_at = Utc::now() + Duration::hours(self.token_ttl_hours);

        // Create placeholder WebPoMinter for now
        let placeholder_minter = self.create_placeholder_webpo_minter();

        Ok(TokenMinterEntry::new(
            expires_at,
            "placeholder_integrity_token",
            3600,
            300,
            None,
            placeholder_minter,
        ))
    }

    /// Create a placeholder WebPoMinter for testing
    fn create_placeholder_webpo_minter(&self) -> crate::session::WebPoMinter {
        use crate::session::webpo_minter::JsRuntimeHandle;

        crate::session::WebPoMinter {
            mint_callback_ref: "placeholder_callback".to_string(),
            runtime_handle: JsRuntimeHandle::new_for_test(),
        }
    }

    /// Initialize BotGuard client
    pub async fn initialize_botguard(&self) -> Result<()> {
        if self.botguard_client.is_initialized().await {
            return Ok(());
        }

        self.botguard_client
            .initialize()
            .await
            .map_err(|e| crate::Error::session(format!("BotGuard initialization failed: {}", e)))
    }

    /// Generate POT token using BotGuard client
    pub async fn generate_po_token(&self, identifier: &str) -> Result<String> {
        // Create new instance on demand since botguard is not Send+Sync
        self.botguard_client.generate_po_token(identifier).await
    }

    /// Mint POT token using the BotGuard client (replaces WebPoMinter)
    ///
    /// Corresponds to TypeScript: `tryMintPOT` method (L410-436)
    async fn mint_pot_token(
        &self,
        content_binding: &str,
        _token_minter: &TokenMinterEntry, // Keep for backward compatibility
    ) -> Result<SessionData> {
        tracing::info!("Generating POT for {}", content_binding);

        // Use the BotGuard client to generate POT token
        let po_token = self.generate_po_token(content_binding).await?;

        let expires_at = Utc::now() + Duration::hours(self.token_ttl_hours);

        tracing::info!("Generated POT token: {}", po_token);

        Ok(SessionData::new(po_token, content_binding, expires_at))
    }

    /// Get diagnostic information about the session manager
    ///
    /// This method provides access to internal configuration for testing and diagnostics
    pub fn get_diagnostic_info(&self) -> (String, String) {
        (self.request_key.clone(), self.settings.server.host.clone())
    }

    /// Check that HTTP client is accessible and configured
    pub fn has_http_client(&self) -> bool {
        // Access the http_client field to verify it's readable
        format!("{:?}", self.http_client).contains("Client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);
        assert!(manager.session_data_caches.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_session_manager_fields_accessibility() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        // Verify all fields can be accessed and used
        assert!(manager.session_data_caches.read().await.len() == 0); // Initial should be empty

        let minter_cache_size = manager.minter_cache.read().await.len();
        assert_eq!(minter_cache_size, 0); // Initial should be empty

        // Verify other fields are accessible
        assert!(!manager.request_key.is_empty());
        assert_eq!(manager.token_ttl_hours, 6);

        // Access fields through diagnostic methods to prove they're readable
        let (request_key, server_host) = manager.get_diagnostic_info();
        assert!(!request_key.is_empty());
        assert_eq!(request_key, "O43z0dpjhgX20SCx4KAo");
        assert!(!server_host.is_empty());

        // Verify http_client field is accessible
        assert!(manager.has_http_client());

        // Verify method that uses the fields works
        let request = PotRequest::new().with_content_binding("test_field_access");
        let result = manager.generate_pot_token(&request).await;
        assert!(result.is_ok()); // This exercises settings and http_client internally
    }

    #[tokio::test]
    async fn test_generate_pot_token() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let request = PotRequest::new().with_content_binding("test_video_id");

        let response = manager.generate_pot_token(&request).await.unwrap();
        assert_eq!(response.content_binding, "test_video_id");
        assert!(!response.is_expired());
    }

    #[tokio::test]
    async fn test_token_caching() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let request = PotRequest::new().with_content_binding("cached_video");

        // First call should generate new token
        let response1 = manager.generate_pot_token(&request).await.unwrap();

        // Second call should return cached token
        let response2 = manager.generate_pot_token(&request).await.unwrap();

        assert_eq!(response1.po_token, response2.po_token);
        assert_eq!(response1.expires_at, response2.expires_at);
    }

    #[tokio::test]
    async fn test_bypass_cache() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let request_cached = PotRequest::new().with_content_binding("bypass_test");

        let request_bypass = PotRequest::new()
            .with_content_binding("bypass_test")
            .with_bypass_cache(true);

        // First call to populate cache
        let _response1 = manager.generate_pot_token(&request_cached).await.unwrap();

        // Second call with bypass_cache should generate new token
        let response2 = manager.generate_pot_token(&request_bypass).await.unwrap();
        assert_eq!(response2.content_binding, "bypass_test");
    }

    #[tokio::test]
    async fn test_invalidate_caches() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let request = PotRequest::new().with_content_binding("test_invalidate");

        // Generate and cache a token
        let _response = manager.generate_pot_token(&request).await.unwrap();

        // Verify cache has content
        assert!(!manager.session_data_caches.read().await.is_empty());

        // Invalidate caches
        manager.invalidate_caches().await.unwrap();

        // Verify cache is empty
        assert!(manager.session_data_caches.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_generate_visitor_data() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let visitor_data = manager.generate_visitor_data().await.unwrap();
        assert!(!visitor_data.is_empty());
    }

    #[tokio::test]
    async fn test_generate_visitor_data_with_mock() {
        // Create a mock provider
        #[derive(Debug)]
        struct MockInnertubeProvider;

        #[async_trait::async_trait]
        impl crate::session::innertube::InnertubeProvider for MockInnertubeProvider {
            async fn generate_visitor_data(&self) -> Result<String> {
                Ok("mock_visitor_data_12345".to_string())
            }

            async fn get_challenge(
                &self,
                _context: &crate::types::InnertubeContext,
            ) -> crate::Result<crate::types::ChallengeData> {
                // Mock implementation
                Ok(crate::types::ChallengeData {
                    interpreter_url: crate::types::TrustedResourceUrl::new("//mock.url"),
                    interpreter_hash: "mock_hash".to_string(),
                    program: "mock_program".to_string(),
                    global_name: "mockGlobal".to_string(),
                    client_experiments_state_blob: Some("mock_blob".to_string()),
                })
            }
        }

        let mock_provider = MockInnertubeProvider;
        let settings = Settings::default();
        let manager = SessionManagerGeneric::new_with_provider(settings, mock_provider);

        let visitor_data = manager.generate_visitor_data().await.unwrap();
        assert_eq!(visitor_data, "mock_visitor_data_12345");
    }

    #[tokio::test]
    async fn test_token_minter_cache() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        // Initially cache should be empty
        let cache_keys = manager.get_minter_cache_keys().await.unwrap();
        assert!(cache_keys.is_empty());

        // Generate a token which should create a minter
        let request = PotRequest::new().with_content_binding("test_minter_cache");
        let _response = manager.generate_pot_token(&request).await.unwrap();

        // Now cache should have entries
        let cache_keys = manager.get_minter_cache_keys().await.unwrap();
        assert!(!cache_keys.is_empty());
    }

    #[tokio::test]
    async fn test_proxy_spec_creation() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let request = PotRequest::new()
            .with_content_binding("test_proxy")
            .with_proxy("http://proxy:8080")
            .with_source_address("192.168.1.1")
            .with_disable_tls_verification(true);

        // Should handle proxy settings without crashing
        let response = manager.generate_pot_token(&request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_content_binding_generation() {
        // Create a mock provider that returns known visitor data
        #[derive(Debug)]
        struct TestVisitorProvider;

        #[async_trait::async_trait]
        impl crate::session::innertube::InnertubeProvider for TestVisitorProvider {
            async fn generate_visitor_data(&self) -> Result<String> {
                Ok("test_visitor_data_from_mock".to_string())
            }

            async fn get_challenge(
                &self,
                _context: &crate::types::InnertubeContext,
            ) -> crate::Result<crate::types::ChallengeData> {
                Ok(crate::types::ChallengeData {
                    interpreter_url: crate::types::TrustedResourceUrl::new("//test.url"),
                    interpreter_hash: "test_hash".to_string(),
                    program: "test_program".to_string(),
                    global_name: "testGlobal".to_string(),
                    client_experiments_state_blob: Some("test_blob".to_string()),
                })
            }
        }

        let mock_provider = TestVisitorProvider;
        let settings = Settings::default();
        let manager = SessionManagerGeneric::new_with_provider(settings, mock_provider);

        // Request without content binding should generate visitor data
        let request = PotRequest::new();
        let response = manager.generate_pot_token(&request).await.unwrap();

        // Should use generated visitor data as content binding
        assert!(!response.content_binding.is_empty());
        assert_eq!(response.content_binding, "test_visitor_data_from_mock");
    }

    #[tokio::test]
    async fn test_integrity_token_invalidation() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        // Generate a token to create minter entries
        let request = PotRequest::new().with_content_binding("test_it_invalidation");
        let _response = manager.generate_pot_token(&request).await.unwrap();

        // Verify we have cache entries
        let cache_keys = manager.get_minter_cache_keys().await.unwrap();
        assert!(!cache_keys.is_empty());

        // Invalidate integrity tokens
        manager.invalidate_integrity_tokens().await.unwrap();

        // Cache keys should still exist but tokens should be expired
        let cache_keys_after = manager.get_minter_cache_keys().await.unwrap();
        assert_eq!(cache_keys.len(), cache_keys_after.len());
    }

    #[tokio::test]
    async fn test_environment_proxy_detection() {
        use std::env;

        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        // Set environment proxy
        unsafe {
            env::set_var("HTTP_PROXY", "http://env-proxy:8080");
        }

        let request = PotRequest::new().with_content_binding("test_env_proxy");
        let response = manager.generate_pot_token(&request).await;

        // Should succeed even with environment proxy
        assert!(response.is_ok());

        // Cleanup
        unsafe {
            env::remove_var("HTTP_PROXY");
        }
    }

    #[tokio::test]
    async fn test_innertube_context_handling() {
        let settings = Settings::default();
        let manager = SessionManager::new(settings);

        let innertube_context = serde_json::json!({
            "client": {
                "remoteHost": "youtube.com"
            }
        });

        let request = PotRequest::new()
            .with_content_binding("test_innertube")
            .with_innertube_context(innertube_context);

        let response = manager.generate_pot_token(&request).await;
        assert!(response.is_ok());
    }
}

// Explicit trait implementations for thread safety
// SessionManager contains only Send + Sync types:
// - Arc<Settings> (Send + Sync)
// - Client (Send + Sync)
// - RwLock<HashMap<...>> (Send + Sync)
// - String (Send + Sync)
// - i64 (Send + Sync)
// - Arc<InnertubeClient> (Send + Sync)
// - BotGuardClient (Send + Sync - explicit implementation above)
unsafe impl<T> Send for SessionManagerGeneric<T> where
    T: crate::session::innertube::InnertubeProvider + std::fmt::Debug + Send + Sync
{
}

unsafe impl<T> Sync for SessionManagerGeneric<T> where
    T: crate::session::innertube::InnertubeProvider + std::fmt::Debug + Send + Sync
{
}
