//! Response type definitions
//!
//! Defines the structure for POT token generation responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response for POT token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotResponse {
    /// The generated POT token
    #[serde(rename = "poToken")]
    pub po_token: String,

    /// The content binding used for token generation
    #[serde(rename = "contentBinding")]
    pub content_binding: String,

    /// Token expiration timestamp
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

impl PotResponse {
    /// Create a new POT response
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

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get time remaining until expiration
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }
}

/// Ping response for health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    /// Server uptime in seconds
    pub server_uptime: u64,

    /// Server version
    pub version: String,
}

impl PingResponse {
    /// Create a new ping response
    pub fn new(server_uptime: u64, version: impl Into<String>) -> Self {
        Self {
            server_uptime,
            version: version.into(),
        }
    }
}

/// Error response for API errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

/// Minter cache keys response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinterCacheResponse {
    /// List of cache keys
    pub cache_keys: Vec<String>,
}

impl MinterCacheResponse {
    /// Create a new minter cache response
    pub fn new(cache_keys: Vec<String>) -> Self {
        Self { cache_keys }
    }

    /// Create an empty minter cache response
    pub fn empty() -> Self {
        Self {
            cache_keys: Vec::new(),
        }
    }

    /// Add a cache key
    pub fn add_key(&mut self, key: impl Into<String>) {
        self.cache_keys.push(key.into());
    }

    /// Get the number of cache keys
    pub fn len(&self) -> usize {
        self.cache_keys.len()
    }

    /// Check if the cache keys list is empty
    pub fn is_empty(&self) -> bool {
        self.cache_keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_pot_response_creation() {
        let expires_at = Utc::now() + Duration::hours(6);
        let response = PotResponse::new("test_token", "test_binding", expires_at);

        assert_eq!(response.po_token, "test_token");
        assert_eq!(response.content_binding, "test_binding");
        assert_eq!(response.expires_at, expires_at);
    }

    #[test]
    fn test_pot_response_expiration() {
        let past_time = Utc::now() - Duration::hours(1);
        let future_time = Utc::now() + Duration::hours(1);

        let expired_response = PotResponse::new("token", "binding", past_time);
        let valid_response = PotResponse::new("token", "binding", future_time);

        assert!(expired_response.is_expired());
        assert!(!valid_response.is_expired());
    }

    #[test]
    fn test_pot_response_serialization() {
        let expires_at = Utc::now() + Duration::hours(6);
        let response = PotResponse::new("test_token", "test_binding", expires_at);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("poToken"));
        assert!(json.contains("contentBinding"));
        assert!(json.contains("expiresAt"));

        let deserialized: PotResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.po_token, "test_token");
        assert_eq!(deserialized.content_binding, "test_binding");
    }

    #[test]
    fn test_ping_response() {
        let response = PingResponse::new(3600, "1.0.0");
        assert_eq!(response.server_uptime, 3600);
        assert_eq!(response.version, "1.0.0");
    }

    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new("Test error");
        assert_eq!(response.error, "Test error");
    }

    #[test]
    fn test_minter_cache_response() {
        let mut response = MinterCacheResponse::empty();
        assert!(response.is_empty());
        assert_eq!(response.len(), 0);

        response.add_key("cache_key_1");
        response.add_key("cache_key_2");

        assert!(!response.is_empty());
        assert_eq!(response.len(), 2);
        assert_eq!(response.cache_keys, vec!["cache_key_1", "cache_key_2"]);
    }

    #[test]
    fn test_minter_cache_response_new() {
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let response = MinterCacheResponse::new(keys.clone());

        assert_eq!(response.cache_keys, keys);
        assert_eq!(response.len(), 3);
    }

    #[test]
    fn test_minter_cache_response_serialization() {
        let response = MinterCacheResponse::new(vec!["test_key".to_string()]);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("cache_keys"));

        let deserialized: MinterCacheResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cache_keys, vec!["test_key"]);
    }
}
