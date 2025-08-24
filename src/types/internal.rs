//! Internal data structures
//!
//! Defines the internal data types used for session management and BotGuard processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "privateDoNotAccessOrElseSafeScriptWrappedValue")]
    pub private_do_not_access_or_else_safe_script_wrapped_value: String,
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

/// Token minter information
#[derive(Debug, Clone)]
pub struct TokenMinter {
    /// Expiry time
    pub expiry: DateTime<Utc>,
    /// Integrity token
    pub integrity_token: String,
    /// Minter instance (placeholder for now)
    pub minter: String, // TODO: Replace with actual minter type
}

impl TokenMinter {
    /// Create a new token minter
    pub fn new(
        expiry: DateTime<Utc>,
        integrity_token: impl Into<String>,
        minter: impl Into<String>,
    ) -> Self {
        Self {
            expiry,
            integrity_token: integrity_token.into(),
            minter: minter.into(),
        }
    }

    /// Check if the minter has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
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
    fn test_token_minter() {
        let future_time = Utc::now() + Duration::hours(1);
        let minter = TokenMinter::new(future_time, "integrity_token", "minter_instance");

        assert!(!minter.is_expired());
        assert_eq!(minter.integrity_token, "integrity_token");
        assert_eq!(minter.minter, "minter_instance");
    }

    #[test]
    fn test_token_minter_expired() {
        let past_time = Utc::now() - Duration::hours(1);
        let minter = TokenMinter::new(past_time, "token", "minter");

        assert!(minter.is_expired());
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
