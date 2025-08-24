//! Error type definitions
//!
//! Defines the main error types used throughout the POT provider application.

use thiserror::Error;

/// Main error type for the POT provider
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// HTTP server errors
    #[error("Server error: {0}")]
    Server(String),

    /// Session management errors
    #[error("Session error: {0}")]
    Session(String),

    /// Token generation errors
    #[error("Token generation error: {0}")]
    TokenGeneration(String),

    /// BotGuard related errors
    #[error("BotGuard error: {message}")]
    BotGuard { message: String },

    /// Cache operation errors
    #[error("Cache error: {operation}")]
    Cache { operation: String },

    /// Integrity token errors
    #[error("Integrity token error: {details}")]
    IntegrityToken { details: String },

    /// Visitor data generation errors
    #[error("Visitor data generation failed: {reason}")]
    VisitorData { reason: String },

    /// Challenge processing errors
    #[error("Challenge processing failed: {stage}")]
    Challenge { stage: String },

    /// Proxy configuration errors
    #[error("Proxy error: {config}")]
    Proxy { config: String },

    /// Network/HTTP client errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Date/time parsing errors
    #[error("Date parsing error: {0}")]
    DateParse(#[from] chrono::ParseError),

    /// Generic errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new server error
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Create a new session error
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create a new token generation error
    pub fn token_generation(msg: impl Into<String>) -> Self {
        Self::TokenGeneration(msg.into())
    }

    /// Create a BotGuard error
    pub fn botguard(message: impl Into<String>) -> Self {
        Self::BotGuard {
            message: message.into(),
        }
    }

    /// Create a cache error
    pub fn cache(operation: impl Into<String>) -> Self {
        Self::Cache {
            operation: operation.into(),
        }
    }

    /// Create an integrity token error
    pub fn integrity_token(details: impl Into<String>) -> Self {
        Self::IntegrityToken {
            details: details.into(),
        }
    }

    /// Create a visitor data error
    pub fn visitor_data(reason: impl Into<String>) -> Self {
        Self::VisitorData {
            reason: reason.into(),
        }
    }

    /// Create a challenge error
    pub fn challenge(stage: impl Into<String>) -> Self {
        Self::Challenge {
            stage: stage.into(),
        }
    }

    /// Create a proxy error
    pub fn proxy(config: impl Into<String>) -> Self {
        Self::Proxy {
            config: config.into(),
        }
    }

    /// Create a new internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test config error");
        assert!(matches!(err, Error::Config(_)));
        assert_eq!(err.to_string(), "Configuration error: test config error");
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());

        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::Json(_)));
    }

    #[test]
    fn test_botguard_error() {
        let err = Error::botguard("Test BotGuard error");
        assert!(matches!(err, Error::BotGuard { .. }));
        assert!(err.to_string().contains("BotGuard error"));
    }

    #[test]
    fn test_cache_error() {
        let err = Error::cache("clear operation failed");
        assert!(matches!(err, Error::Cache { .. }));
        assert!(err.to_string().contains("Cache error"));
    }

    #[test]
    fn test_integrity_token_error() {
        let err = Error::integrity_token("Token validation failed");
        assert!(matches!(err, Error::IntegrityToken { .. }));
        assert!(err.to_string().contains("Integrity token error"));
    }

    #[test]
    fn test_visitor_data_error() {
        let err = Error::visitor_data("Generation failed");
        assert!(matches!(err, Error::VisitorData { .. }));
        assert!(err.to_string().contains("Visitor data generation failed"));
    }

    #[test]
    fn test_challenge_error() {
        let err = Error::challenge("Processing failed");
        assert!(matches!(err, Error::Challenge { .. }));
        assert!(err.to_string().contains("Challenge processing failed"));
    }

    #[test]
    fn test_proxy_error() {
        let err = Error::proxy("Invalid proxy config");
        assert!(matches!(err, Error::Proxy { .. }));
        assert!(err.to_string().contains("Proxy error"));
    }

    #[test]
    fn test_date_parse_error() {
        let date_err = chrono::DateTime::parse_from_rfc3339("invalid date");
        assert!(date_err.is_err());

        let err: Error = date_err.unwrap_err().into();
        assert!(matches!(err, Error::DateParse(_)));
    }
}
