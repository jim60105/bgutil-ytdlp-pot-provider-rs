//! Internal data structures
//!
//! Defines the internal data types used for session management and BotGuard processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Import WebPoMinter
use crate::session::WebPoMinter;

/// YouTube session data for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// POT token
    pub po_token: String,
    /// Content binding
    pub content_binding: String,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
}

impl SessionData {
    /// Create new session data
    pub fn new(
        po_token: impl Into<String>,
        content_binding: impl Into<String>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            po_token: po_token.into(),
            content_binding: content_binding.into(),
            expires_at,
        }
    }

    /// Check if session data has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get time remaining until expiration
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }
}

/// BotGuard challenge data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeData {
    /// Interpreter URL information
    pub interpreter_url: TrustedResourceUrl,
    /// Interpreter hash
    pub interpreter_hash: String,
    /// Challenge program
    pub program: String,
    /// Global VM name
    pub global_name: String,
    /// Client experiments state blob
    pub client_experiments_state_blob: Option<String>,
}

/// Trusted resource URL wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedResourceUrl {
    /// The wrapped trusted resource URL value (Google's private field naming convention)
    #[serde(rename = "privateDoNotAccessOrElseTrustedResourceUrlWrappedValue")]
    pub private_do_not_access_or_else_trusted_resource_url_wrapped_value: String,
}

impl TrustedResourceUrl {
    /// Create a new trusted resource URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            private_do_not_access_or_else_trusted_resource_url_wrapped_value: url.into(),
        }
    }

    /// Get the wrapped URL value
    pub fn url(&self) -> &str {
        &self.private_do_not_access_or_else_trusted_resource_url_wrapped_value
    }
}

/// Descrambled challenge for BotGuard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescrambledChallenge {
    /// Message ID
    pub message_id: Option<String>,
    /// Interpreter JavaScript code
    pub interpreter_javascript: TrustedScript,
    /// Interpreter hash
    pub interpreter_hash: String,
    /// Challenge program
    pub program: String,
    /// Global VM name
    pub global_name: String,
    /// Client experiments state blob
    pub client_experiments_state_blob: Option<String>,
}

/// Trusted script wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedScript {
    /// The wrapped script content (Google's private field naming convention)
    #[serde(rename = "privateDoNotAccessOrElseSafeScriptWrappedValue")]
    pub private_do_not_access_or_else_safe_script_wrapped_value: String,
    /// The trusted resource URL where the script originated (Google's private field naming convention)
    #[serde(rename = "privateDoNotAccessOrElseTrustedResourceUrlWrappedValue")]
    pub private_do_not_access_or_else_trusted_resource_url_wrapped_value: String,
}

impl TrustedScript {
    /// Create a new trusted script
    pub fn new(script: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            private_do_not_access_or_else_safe_script_wrapped_value: script.into(),
            private_do_not_access_or_else_trusted_resource_url_wrapped_value: url.into(),
        }
    }

    /// Get the script content
    pub fn script(&self) -> &str {
        &self.private_do_not_access_or_else_safe_script_wrapped_value
    }

    /// Get the script URL
    pub fn url(&self) -> &str {
        &self.private_do_not_access_or_else_trusted_resource_url_wrapped_value
    }
}

/// Token minter cache entry matching TypeScript TokenMinter
#[derive(Debug, Clone)]
pub struct TokenMinterEntry {
    /// Expiry time
    pub expiry: DateTime<Utc>,
    /// Integrity token for BotGuard
    pub integrity_token: String,
    /// Estimated TTL in seconds
    pub estimated_ttl_secs: u32,
    /// Mint refresh threshold
    pub mint_refresh_threshold: u32,
    /// Websafe fallback token
    pub websafe_fallback_token: Option<String>,
    /// Associated POT minter
    pub minter: WebPoMinter,
}

impl TokenMinterEntry {
    /// Create a new token minter entry
    pub fn new(
        expiry: DateTime<Utc>,
        integrity_token: impl Into<String>,
        estimated_ttl_secs: u32,
        mint_refresh_threshold: u32,
        websafe_fallback_token: Option<String>,
        minter: WebPoMinter,
    ) -> Self {
        Self {
            expiry,
            integrity_token: integrity_token.into(),
            estimated_ttl_secs,
            mint_refresh_threshold,
            websafe_fallback_token,
            minter,
        }
    }

    /// Check if the minter has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }

    /// Get time remaining until expiration
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expiry - Utc::now()
    }
}

/// Innertube context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnertubeContext {
    /// Client information
    pub client: ClientInfo,
}

impl InnertubeContext {
    /// Create a new Innertube context
    pub fn new(client: ClientInfo) -> Self {
        Self { client }
    }
}

/// Client information for Innertube
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Remote host
    pub remote_host: Option<String>,
    /// Visitor data
    pub visitor_data: Option<String>,
}

impl ClientInfo {
    /// Create new client info
    pub fn new() -> Self {
        Self {
            remote_host: None,
            visitor_data: None,
        }
    }

    /// Set remote host
    pub fn with_remote_host(mut self, remote_host: impl Into<String>) -> Self {
        self.remote_host = Some(remote_host.into());
        self
    }

    /// Set visitor data
    pub fn with_visitor_data(mut self, visitor_data: impl Into<String>) -> Self {
        self.visitor_data = Some(visitor_data.into());
        self
    }
}

impl Default for InnertubeContext {
    fn default() -> Self {
        Self::new(ClientInfo::default())
    }
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_session_data_creation() {
        let expires_at = Utc::now() + Duration::hours(6);
        let session = SessionData::new("token456", "binding789", expires_at);

        assert_eq!(session.po_token, "token456");
        assert_eq!(session.content_binding, "binding789");
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_data_expiration() {
        let past_time = Utc::now() - Duration::hours(1);
        let session = SessionData::new("token", "binding", past_time);

        assert!(session.is_expired());
        assert!(session.time_until_expiry().num_seconds() < 0);
    }

    #[test]
    fn test_trusted_resource_url() {
        let url = TrustedResourceUrl::new("https://example.com");
        assert_eq!(url.url(), "https://example.com");
    }

    #[test]
    fn test_trusted_script() {
        let script = TrustedScript::new("console.log('test')", "https://example.com/script.js");
        assert_eq!(script.script(), "console.log('test')");
        assert_eq!(script.url(), "https://example.com/script.js");
    }

    #[test]
    fn test_token_minter_entry_creation_replacement() {
        let future_time = Utc::now() + Duration::hours(1);
        let test_minter = create_test_webpo_minter();

        let entry =
            TokenMinterEntry::new(future_time, "integrity_token", 3600, 300, None, test_minter);

        assert!(!entry.is_expired());
        assert_eq!(entry.integrity_token, "integrity_token");
    }

    #[test]
    fn test_token_minter_entry_expiry_replacement() {
        let past_time = Utc::now() - Duration::hours(1);
        let test_minter = create_test_webpo_minter();

        let entry = TokenMinterEntry::new(past_time, "token", 3600, 300, None, test_minter);

        assert!(entry.is_expired());
    }

    #[test]
    fn test_token_minter_entry_creation() {
        let future_time = Utc::now() + Duration::hours(1);
        let test_minter = create_test_webpo_minter();
        let minter = TokenMinterEntry::new(
            future_time,
            "integrity_token_123",
            3600,
            300,
            Some("websafe_token".to_string()),
            test_minter,
        );

        assert_eq!(minter.integrity_token, "integrity_token_123");
        assert_eq!(minter.estimated_ttl_secs, 3600);
        assert_eq!(minter.mint_refresh_threshold, 300);
        assert_eq!(
            minter.websafe_fallback_token,
            Some("websafe_token".to_string())
        );
        assert!(!minter.is_expired());
    }

    #[test]
    fn test_token_minter_entry_expiration() {
        let past_time = Utc::now() - Duration::hours(1);
        let future_time = Utc::now() + Duration::hours(1);

        let test_minter1 = create_test_webpo_minter();
        let expired_minter =
            TokenMinterEntry::new(past_time, "token", 3600, 300, None, test_minter1);

        let test_minter2 = create_test_webpo_minter();
        let valid_minter =
            TokenMinterEntry::new(future_time, "token", 3600, 300, None, test_minter2);

        assert!(expired_minter.is_expired());
        assert!(!valid_minter.is_expired());
        assert!(valid_minter.time_until_expiry().num_seconds() > 0);
    }

    /// Helper function to create a test WebPoMinter
    fn create_test_webpo_minter() -> WebPoMinter {
        use crate::session::webpo_minter::JsRuntimeHandle;

        WebPoMinter {
            mint_callback_ref: "test_callback".to_string(),
            runtime_handle: JsRuntimeHandle::new_for_test(),
        }
    }

    #[test]
    fn test_client_info_builder() {
        let client = ClientInfo::new()
            .with_remote_host("youtube.com")
            .with_visitor_data("visitor123");

        assert_eq!(client.remote_host, Some("youtube.com".to_string()));
        assert_eq!(client.visitor_data, Some("visitor123".to_string()));
    }

    #[test]
    fn test_innertube_context() {
        let client = ClientInfo::new().with_visitor_data("test_visitor");
        let context = InnertubeContext::new(client);

        assert_eq!(
            context.client.visitor_data,
            Some("test_visitor".to_string())
        );
    }

    #[test]
    fn test_json_serialization() {
        let session = SessionData::new("token", "binding", Utc::now());
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: SessionData = serde_json::from_str(&json).unwrap();

        assert_eq!(session.po_token, deserialized.po_token);
        assert_eq!(session.content_binding, deserialized.content_binding);
    }
}
