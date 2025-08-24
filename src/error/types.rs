//! Enhanced error handling system
//!
//! Provides comprehensive error classification and formatting
//! corresponding to TypeScript strerror function and error handling.

use thiserror::Error;

/// Main error type for the application
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML configuration parsing errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// URL parsing errors
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// BotGuard related errors (corresponds to BGError in TypeScript)
    #[error("BotGuard error ({code}): {message}")]
    BotGuard {
        code: String,
        message: String,
        info: Option<serde_json::Value>,
    },

    /// Token generation errors
    #[error("Token generation failed: {reason}")]
    TokenGeneration {
        reason: String,
        stage: Option<String>,
    },

    /// Cache operation errors
    #[error("Cache error during {operation}: {details}")]
    Cache { operation: String, details: String },

    /// Configuration errors
    #[error("Configuration error in {field}: {message}")]
    Config { field: String, message: String },

    /// Integrity token errors
    #[error("Integrity token error: {details}")]
    IntegrityToken {
        details: String,
        response_data: Option<serde_json::Value>,
    },

    /// Visitor data generation errors
    #[error("Visitor data generation failed: {reason}")]
    VisitorData {
        reason: String,
        context: Option<String>,
    },

    /// Challenge processing errors
    #[error("Challenge processing failed at stage '{stage}': {message}")]
    Challenge { stage: String, message: String },

    /// Proxy configuration errors
    #[error("Proxy error with config '{config}': {message}")]
    Proxy { config: String, message: String },

    /// Network/connection errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        retry_count: Option<u32>,
    },

    /// Timeout errors
    #[error("Operation timed out after {duration_secs} seconds: {operation}")]
    Timeout {
        operation: String,
        duration_secs: u64,
    },

    /// Authentication/authorization errors
    #[error("Authentication failed: {reason}")]
    Auth {
        reason: String,
        endpoint: Option<String>,
    },

    /// Rate limiting errors
    #[error("Rate limited: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },

    /// Validation errors
    #[error("Validation failed for {field}: {message}")]
    Validation {
        field: String,
        message: String,
        value: Option<String>,
    },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        context: Option<String>,
    },

    // Legacy error types for backward compatibility
    /// Configuration-related errors (legacy)
    #[error("Configuration error: {0}")]
    ConfigLegacy(String),

    /// HTTP server errors (legacy)
    #[error("Server error: {0}")]
    Server(String),

    /// Session management errors (legacy)
    #[error("Session error: {0}")]
    Session(String),

    /// Token generation errors (legacy)
    #[error("Token generation error: {0}")]
    TokenGenerationLegacy(String),

    /// BotGuard related errors (legacy)
    #[error("BotGuard error: {message}")]
    BotGuardLegacy { message: String },

    /// Cache operation errors (legacy)
    #[error("Cache error: {operation}")]
    CacheLegacy { operation: String },

    /// Integrity token errors (legacy)
    #[error("Integrity token error: {details}")]
    IntegrityTokenLegacy { details: String },

    /// Visitor data generation errors (legacy)
    #[error("Visitor data generation failed: {reason}")]
    VisitorDataLegacy { reason: String },

    /// Challenge processing errors (legacy)
    #[error("Challenge processing failed: {stage}")]
    ChallengeLegacy { stage: String },

    /// Proxy configuration errors (legacy)
    #[error("Proxy error: {config}")]
    ProxyLegacy { config: String },

    /// Date/time parsing errors
    #[error("Date parsing error: {0}")]
    DateParse(#[from] chrono::ParseError),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a BotGuard error (corresponds to BGError in TypeScript)
    pub fn botguard<S: Into<String>>(code: S, message: S) -> Self {
        Self::BotGuard {
            code: code.into(),
            message: message.into(),
            info: None,
        }
    }

    /// Create a BotGuard error with additional info
    pub fn botguard_with_info<S: Into<String>>(
        code: S,
        message: S,
        info: serde_json::Value,
    ) -> Self {
        Self::BotGuard {
            code: code.into(),
            message: message.into(),
            info: Some(info),
        }
    }

    /// Create a token generation error
    pub fn token_generation<S: Into<String>>(reason: S) -> Self {
        Self::TokenGeneration {
            reason: reason.into(),
            stage: None,
        }
    }

    /// Create a token generation error with stage info
    pub fn token_generation_at_stage<S: Into<String>>(reason: S, stage: S) -> Self {
        Self::TokenGeneration {
            reason: reason.into(),
            stage: Some(stage.into()),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(operation: S, details: S) -> Self {
        Self::Cache {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(field: S, message: S) -> Self {
        Self::Config {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an integrity token error
    pub fn integrity_token<S: Into<String>>(details: S) -> Self {
        Self::IntegrityToken {
            details: details.into(),
            response_data: None,
        }
    }

    /// Create a challenge error
    pub fn challenge<S: Into<String>>(stage: S, message: S) -> Self {
        Self::Challenge {
            stage: stage.into(),
            message: message.into(),
        }
    }

    /// Create a proxy error
    pub fn proxy<S: Into<String>>(config: S, message: S) -> Self {
        Self::Proxy {
            config: config.into(),
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            retry_count: None,
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S, duration_secs: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_secs,
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(field: S, message: S) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
            value: None,
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
            context: None,
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Network { .. } => true,
            Error::Timeout { .. } => true,
            Error::Http(e) => e.is_timeout() || e.is_connect(),
            Error::RateLimit { .. } => true,
            _ => false,
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Http(..) => "http",
            Error::Json(..) => "json",
            Error::Toml(..) => "toml",
            Error::Url(..) => "url",
            Error::Io(..) => "io",
            Error::BotGuard { .. } => "botguard",
            Error::TokenGeneration { .. } => "token_generation",
            Error::Cache { .. } => "cache",
            Error::Config { .. } => "config",
            Error::IntegrityToken { .. } => "integrity_token",
            Error::VisitorData { .. } => "visitor_data",
            Error::Challenge { .. } => "challenge",
            Error::Proxy { .. } => "proxy",
            Error::Network { .. } => "network",
            Error::Timeout { .. } => "timeout",
            Error::Auth { .. } => "auth",
            Error::RateLimit { .. } => "rate_limit",
            Error::Validation { .. } => "validation",
            Error::Internal { .. } => "internal",
            // Legacy variants
            Error::ConfigLegacy(..) => "config",
            Error::Server(..) => "server",
            Error::Session(..) => "session",
            Error::TokenGenerationLegacy(..) => "token_generation",
            Error::BotGuardLegacy { .. } => "botguard",
            Error::CacheLegacy { .. } => "cache",
            Error::IntegrityTokenLegacy { .. } => "integrity_token",
            Error::VisitorDataLegacy { .. } => "visitor_data",
            Error::ChallengeLegacy { .. } => "challenge",
            Error::ProxyLegacy { .. } => "proxy",
            Error::DateParse(..) => "date_parse",
        }
    }

    // Legacy constructor methods for backward compatibility
    /// Create a new configuration error (legacy)
    pub fn config_legacy(msg: impl Into<String>) -> Self {
        Self::ConfigLegacy(msg.into())
    }

    /// Create a new server error
    pub fn server(msg: impl Into<String>) -> Self {
        Self::Server(msg.into())
    }

    /// Create a new session error
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create a new token generation error (legacy)
    pub fn token_generation_legacy(msg: impl Into<String>) -> Self {
        Self::TokenGenerationLegacy(msg.into())
    }

    /// Create a BotGuard error (legacy)
    pub fn botguard_legacy(message: impl Into<String>) -> Self {
        Self::BotGuardLegacy {
            message: message.into(),
        }
    }

    /// Create a cache error (legacy)
    pub fn cache_legacy(operation: impl Into<String>) -> Self {
        Self::CacheLegacy {
            operation: operation.into(),
        }
    }

    /// Create an integrity token error (legacy)
    pub fn integrity_token_legacy(details: impl Into<String>) -> Self {
        Self::IntegrityTokenLegacy {
            details: details.into(),
        }
    }

    /// Create a visitor data error (legacy)
    pub fn visitor_data_legacy(reason: impl Into<String>) -> Self {
        Self::VisitorDataLegacy {
            reason: reason.into(),
        }
    }

    /// Create a challenge error (legacy)
    pub fn challenge_legacy(stage: impl Into<String>) -> Self {
        Self::ChallengeLegacy {
            stage: stage.into(),
        }
    }

    /// Create a proxy error (legacy)
    pub fn proxy_legacy(config: impl Into<String>) -> Self {
        Self::ProxyLegacy {
            config: config.into(),
        }
    }

    /// Create a new internal error (legacy)
    pub fn internal_legacy(msg: impl Into<String>) -> Self {
        Self::Internal {
            message: msg.into(),
            context: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("field", "test config error");
        assert!(matches!(err, Error::Config { .. }));
        assert_eq!(
            err.to_string(),
            "Configuration error in field: test config error"
        );
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
        let err = Error::botguard("403", "Test BotGuard error");
        assert!(matches!(err, Error::BotGuard { .. }));
        assert!(err.to_string().contains("BotGuard error"));
    }

    #[test]
    fn test_cache_error() {
        let err = Error::cache("clear", "operation failed");
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
        let err = Error::VisitorData {
            reason: "Generation failed".to_string(),
            context: None,
        };
        assert!(matches!(err, Error::VisitorData { .. }));
        assert!(err.to_string().contains("Visitor data generation failed"));
    }

    #[test]
    fn test_challenge_error() {
        let err = Error::challenge("processing", "Processing failed");
        assert!(matches!(err, Error::Challenge { .. }));
        assert!(err.to_string().contains("Challenge processing failed"));
    }

    #[test]
    fn test_proxy_error() {
        let err = Error::proxy("http://proxy:8080", "Invalid proxy config");
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
