//! Request type definitions
//!
//! Defines the structure for POT token generation requests.

use serde::{Deserialize, Serialize};

/// BotGuard challenge data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Challenge {
    /// Challenge as a string (legacy format or parsed from webpage)
    String(String),
    /// Challenge as structured data (from yt-dlp or Innertube API)
    Data(ChallengeData),
}

/// Structured challenge data from BotGuard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeData {
    /// Interpreter URL wrapper
    #[serde(rename = "interpreterUrl")]
    pub interpreter_url: InterpreterUrl,

    /// Hash of the interpreter
    #[serde(rename = "interpreterHash")]
    pub interpreter_hash: String,

    /// BotGuard program code
    pub program: String,

    /// Global name for the BotGuard instance
    #[serde(rename = "globalName")]
    pub global_name: String,

    /// Client experiments state blob
    #[serde(rename = "clientExperimentsStateBlob")]
    pub client_experiments_state_blob: String,
}

/// Interpreter URL wrapper (Google's trusted resource URL format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterUrl {
    /// The actual URL wrapped in Google's trusted resource format
    #[serde(rename = "privateDoNotAccessOrElseTrustedResourceUrlWrappedValue")]
    pub private_do_not_access_or_else_trusted_resource_url_wrapped_value: String,
}

/// Request for POT token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotRequest {
    /// Content binding for the token (video ID, visitor data, etc.)
    pub content_binding: Option<String>,

    /// Proxy configuration for requests
    pub proxy: Option<String>,

    /// Whether to bypass cache and generate fresh token
    pub bypass_cache: Option<bool>,

    /// BotGuard challenge from Innertube (can be string or structured data)
    pub challenge: Option<Challenge>,

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

    /// Set challenge data as string
    pub fn with_challenge(mut self, challenge: impl Into<String>) -> Self {
        self.challenge = Some(Challenge::String(challenge.into()));
        self
    }

    /// Set challenge data as structured data
    pub fn with_challenge_data(mut self, challenge: ChallengeData) -> Self {
        self.challenge = Some(Challenge::Data(challenge));
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
        assert!(matches!(request.challenge, Some(Challenge::String(_))));
        if let Some(Challenge::String(s)) = request.challenge {
            assert_eq!(s, "test_challenge");
        }
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

    #[test]
    fn test_challenge_string_variant() {
        let challenge = Challenge::String("test_string".to_string());

        // Test serialization
        let json = serde_json::to_string(&challenge).unwrap();
        assert_eq!(json, "\"test_string\"");

        // Test deserialization
        let deserialized: Challenge = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Challenge::String(_)));
        if let Challenge::String(s) = deserialized {
            assert_eq!(s, "test_string");
        }
    }

    #[test]
    fn test_challenge_data_variant() {
        let challenge_data = ChallengeData {
            interpreter_url: InterpreterUrl {
                private_do_not_access_or_else_trusted_resource_url_wrapped_value:
                    "//www.google.com/js/test.js".to_string(),
            },
            interpreter_hash: "test_hash".to_string(),
            program: "test_program".to_string(),
            global_name: "test_global".to_string(),
            client_experiments_state_blob: "test_blob".to_string(),
        };

        let challenge = Challenge::Data(challenge_data);

        // Test serialization
        let json = serde_json::to_string(&challenge).unwrap();
        assert!(json.contains("interpreterUrl"));
        assert!(json.contains("test_hash"));
        assert!(json.contains("test_program"));

        // Test deserialization
        let deserialized: Challenge = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Challenge::Data(_)));
        if let Challenge::Data(data) = deserialized {
            assert_eq!(data.interpreter_hash, "test_hash");
            assert_eq!(data.program, "test_program");
            assert_eq!(data.global_name, "test_global");
        }
    }

    #[test]
    fn test_challenge_data_deserialization_from_ytdlp_format() {
        // Test with actual yt-dlp format
        let json = r#"{
            "interpreterUrl": {
                "privateDoNotAccessOrElseTrustedResourceUrlWrappedValue": "//www.google.com/js/th/test.js"
            },
            "interpreterHash": "test_hash_123",
            "program": "program_data",
            "globalName": "global_name",
            "clientExperimentsStateBlob": "blob_data"
        }"#;

        let challenge: Challenge = serde_json::from_str(json).unwrap();
        assert!(matches!(challenge, Challenge::Data(_)));

        if let Challenge::Data(data) = challenge {
            assert_eq!(
                data.interpreter_url
                    .private_do_not_access_or_else_trusted_resource_url_wrapped_value,
                "//www.google.com/js/th/test.js"
            );
            assert_eq!(data.interpreter_hash, "test_hash_123");
            assert_eq!(data.program, "program_data");
            assert_eq!(data.global_name, "global_name");
            assert_eq!(data.client_experiments_state_blob, "blob_data");
        }
    }

    #[test]
    fn test_pot_request_with_challenge_data() {
        let challenge_data = ChallengeData {
            interpreter_url: InterpreterUrl {
                private_do_not_access_or_else_trusted_resource_url_wrapped_value:
                    "//www.google.com/js/test.js".to_string(),
            },
            interpreter_hash: "hash123".to_string(),
            program: "program123".to_string(),
            global_name: "global123".to_string(),
            client_experiments_state_blob: "blob123".to_string(),
        };

        let request = PotRequest::new()
            .with_challenge_data(challenge_data)
            .with_content_binding("test_video");

        assert!(matches!(request.challenge, Some(Challenge::Data(_))));
        assert_eq!(request.content_binding, Some("test_video".to_string()));

        // Test serialization round-trip
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: PotRequest = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.challenge, Some(Challenge::Data(_))));
        if let Some(Challenge::Data(data)) = deserialized.challenge {
            assert_eq!(data.interpreter_hash, "hash123");
        }
    }

    #[test]
    fn test_pot_request_deserialize_with_challenge_object() {
        // Simulate yt-dlp request with challenge object
        let json = r#"{
            "content_binding": "test_id",
            "bypass_cache": false,
            "challenge": {
                "interpreterUrl": {
                    "privateDoNotAccessOrElseTrustedResourceUrlWrappedValue": "//www.google.com/js/test.js"
                },
                "interpreterHash": "test_hash",
                "program": "test_program",
                "globalName": "test_global",
                "clientExperimentsStateBlob": "test_blob"
            },
            "disable_innertube": false,
            "disable_tls_verification": false
        }"#;

        let request: PotRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.content_binding, Some("test_id".to_string()));
        assert!(matches!(request.challenge, Some(Challenge::Data(_))));
        assert_eq!(request.bypass_cache, Some(false));
    }

    #[test]
    fn test_pot_request_deserialize_with_challenge_string() {
        // Test backwards compatibility with string challenge
        let json = r#"{
            "content_binding": "test_id",
            "challenge": "simple_string_challenge"
        }"#;

        let request: PotRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.content_binding, Some("test_id".to_string()));
        assert!(matches!(request.challenge, Some(Challenge::String(_))));

        if let Some(Challenge::String(s)) = request.challenge {
            assert_eq!(s, "simple_string_challenge");
        }
    }

    #[test]
    fn test_interpreter_url_serialization() {
        let url = InterpreterUrl {
            private_do_not_access_or_else_trusted_resource_url_wrapped_value:
                "//www.google.com/js/test.js".to_string(),
        };

        let json = serde_json::to_string(&url).unwrap();
        assert!(json.contains("privateDoNotAccessOrElseTrustedResourceUrlWrappedValue"));
        assert!(json.contains("//www.google.com/js/test.js"));

        let deserialized: InterpreterUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.private_do_not_access_or_else_trusted_resource_url_wrapped_value,
            "//www.google.com/js/test.js"
        );
    }
}
