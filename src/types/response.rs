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
}
