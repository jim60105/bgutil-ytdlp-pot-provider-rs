//! Request type definitions
//!
//! Defines the structure for POT token generation requests.

use serde::{Deserialize, Serialize};

/// Request for POT token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotRequest {
    /// Content binding for the token (video ID, visitor data, etc.)
    pub content_binding: Option<String>,

    /// Proxy configuration for requests
    pub proxy: Option<String>,

    /// Whether to bypass cache and generate fresh token
    pub bypass_cache: Option<bool>,

    /// BotGuard challenge from Innertube
    pub challenge: Option<String>,

    /// Whether to disable challenges from Innertube
    pub disable_innertube: Option<bool>,

    /// Whether to disable TLS certificate verification
    pub disable_tls_verification: Option<bool>,

    /// Innertube context object
    pub innertube_context: Option<serde_json::Value>,

    /// Client-side IP address to bind to
    pub source_address: Option<String>,
}

/// Challenge invalidation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateRequest {
    /// Type of invalidation
    pub invalidate_type: InvalidationType,
}

impl InvalidateRequest {
    /// Create a new invalidate request
    pub fn new(invalidate_type: InvalidationType) -> Self {
        Self { invalidate_type }
    }

    /// Create a cache invalidation request
    pub fn caches() -> Self {
        Self::new(InvalidationType::Caches)
    }

    /// Create an integrity token invalidation request
    pub fn integrity_token() -> Self {
        Self::new(InvalidationType::IntegrityToken)
    }
}

/// Type of invalidation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationType {
    /// Invalidate cached tokens
    Caches,
    /// Invalidate integrity token
    #[serde(rename = "IT")]
    IntegrityToken,
}

impl Default for PotRequest {
    fn default() -> Self {
        Self {
            content_binding: None,
            proxy: None,
            bypass_cache: Some(false),
            challenge: None,
            disable_innertube: Some(false),
            disable_tls_verification: Some(false),
            innertube_context: None,
            source_address: None,
        }
    }
}

impl PotRequest {
    /// Create a new request with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content binding
    pub fn with_content_binding(mut self, content_binding: impl Into<String>) -> Self {
        self.content_binding = Some(content_binding.into());
        self
    }

    /// Set proxy configuration
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set bypass cache flag
    pub fn with_bypass_cache(mut self, bypass_cache: bool) -> Self {
        self.bypass_cache = Some(bypass_cache);
        self
    }

    /// Set source address
    pub fn with_source_address(mut self, source_address: impl Into<String>) -> Self {
        self.source_address = Some(source_address.into());
        self
    }

    /// Set TLS verification flag
    pub fn with_disable_tls_verification(mut self, disable: bool) -> Self {
        self.disable_tls_verification = Some(disable);
        self
    }

    /// Set challenge data
    pub fn with_challenge(mut self, challenge: impl Into<String>) -> Self {
        self.challenge = Some(challenge.into());
        self
    }

    /// Set disable Innertube flag
    pub fn with_disable_innertube(mut self, disable: bool) -> Self {
        self.disable_innertube = Some(disable);
        self
    }

    /// Set Innertube context
    pub fn with_innertube_context(mut self, context: serde_json::Value) -> Self {
        self.innertube_context = Some(context);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pot_request_default() {
        let request = PotRequest::default();
        assert_eq!(request.content_binding, None);
        assert_eq!(request.bypass_cache, Some(false));
        assert_eq!(request.disable_innertube, Some(false));
    }

    #[test]
    fn test_pot_request_builder() {
        let request = PotRequest::new()
            .with_content_binding("test_video_id")
            .with_proxy("http://proxy:8080")
            .with_bypass_cache(true)
            .with_source_address("192.168.1.1")
            .with_disable_tls_verification(true)
            .with_challenge("test_challenge")
            .with_disable_innertube(true);

        assert_eq!(request.content_binding, Some("test_video_id".to_string()));
        assert_eq!(request.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(request.bypass_cache, Some(true));
        assert_eq!(request.source_address, Some("192.168.1.1".to_string()));
        assert_eq!(request.disable_tls_verification, Some(true));
        assert_eq!(request.challenge, Some("test_challenge".to_string()));
        assert_eq!(request.disable_innertube, Some(true));
    }

    #[test]
    fn test_pot_request_serialization() {
        let request = PotRequest::new().with_content_binding("test");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test"));

        let deserialized: PotRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content_binding, Some("test".to_string()));
    }

    #[test]
    fn test_invalidate_request_creation() {
        let cache_request = InvalidateRequest::caches();
        assert!(matches!(
            cache_request.invalidate_type,
            InvalidationType::Caches
        ));

        let it_request = InvalidateRequest::integrity_token();
        assert!(matches!(
            it_request.invalidate_type,
            InvalidationType::IntegrityToken
        ));
    }

    #[test]
    fn test_invalidate_request_serialization() {
        let request = InvalidateRequest::caches();
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: InvalidateRequest = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            deserialized.invalidate_type,
            InvalidationType::Caches
        ));
    }

    #[test]
    fn test_invalidation_type_serialization() {
        let caches = InvalidationType::Caches;
        let json = serde_json::to_string(&caches).unwrap();
        assert_eq!(json, "\"Caches\"");

        let it = InvalidationType::IntegrityToken;
        let json = serde_json::to_string(&it).unwrap();
        assert_eq!(json, "\"IT\"");
    }
}
