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
            .with_bypass_cache(true);

        assert_eq!(request.content_binding, Some("test_video_id".to_string()));
        assert_eq!(request.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(request.bypass_cache, Some(true));
    }

    #[test]
    fn test_pot_request_serialization() {
        let request = PotRequest::new().with_content_binding("test");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test"));

        let deserialized: PotRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content_binding, Some("test".to_string()));
    }
}
