//! Contract compatibility tests
//!
//! These tests ensure API compatibility between Rust and TypeScript implementations.

use bgutil_ytdlp_pot_provider::types::*;
use chrono::Utc;
use pretty_assertions::assert_eq;
use serde_json;

mod common;

#[test]
fn test_pot_request_schema_compatibility() {
    // Verify POT request format is compatible with TypeScript version
    let request = PotRequest::new().with_content_binding("test_video");

    let json = serde_json::to_value(&request).unwrap();

    // Check required fields exist
    assert!(json.get("content_binding").is_some());

    // Check field types
    assert!(json["content_binding"].is_string());
}

#[test]
fn test_pot_response_schema_compatibility() {
    // Verify POT response format compatibility
    let response = PotResponse::new("test_token", "test_video", Utc::now());

    let json = serde_json::to_value(&response).unwrap();

    // Check TypeScript version expected fields
    assert!(json.get("poToken").is_some());
    assert!(json.get("expiresAt").is_some());
    assert!(json.get("contentBinding").is_some());
}

#[test]
fn test_error_response_schema_compatibility() {
    // Verify error response format compatibility
    let error_response = ErrorResponse::new("test_error");
    let json = serde_json::to_value(&error_response).unwrap();

    assert!(json.get("error").is_some());
}

#[test]
fn test_minter_cache_response_schema_compatibility() {
    // Verify minter cache response compatibility
    let cache_response = MinterCacheResponse::new(vec![
        "test_cache_key1".to_string(),
        "test_cache_key2".to_string(),
    ]);

    let json = serde_json::to_value(&cache_response).unwrap();

    // Check structure matches TypeScript expectations
    assert!(json.get("cache_keys").is_some());
    assert!(json["cache_keys"].is_array());

    if let Some(cache_array) = json["cache_keys"].as_array() {
        assert_eq!(cache_array.len(), 2);
        assert_eq!(cache_array[0], "test_cache_key1");
        assert_eq!(cache_array[1], "test_cache_key2");
    }
}

#[test]
fn test_ping_response_schema_compatibility() {
    // Verify ping response matches expected format
    let ping_response = PingResponse::new(12345, "1.0.0");
    let json = serde_json::to_value(&ping_response).unwrap();

    assert!(json.get("server_uptime").is_some());
    assert!(json.get("version").is_some());
    assert_eq!(json["server_uptime"], 12345);
    assert_eq!(json["version"], "1.0.0");
}

#[test]
fn test_json_serialization_consistency() {
    // Test round-trip serialization consistency
    let original_request = PotRequest::new().with_content_binding("test_video_456");

    // Serialize to JSON
    let json_str = serde_json::to_string(&original_request).unwrap();

    // Deserialize back
    let deserialized_request: PotRequest = serde_json::from_str(&json_str).unwrap();

    // Verify consistency
    assert_eq!(
        original_request.content_binding,
        deserialized_request.content_binding
    );
}

#[test]
fn test_response_json_field_names() {
    // Ensure JSON field names match TypeScript expectations exactly
    let response = PotResponse::new("token_123", "video_456", Utc::now());

    let json_str = serde_json::to_string(&response).unwrap();

    // Check for exact field names that TypeScript version expects
    assert!(json_str.contains("\"poToken\""));
    assert!(json_str.contains("\"expiresAt\""));
    assert!(json_str.contains("\"contentBinding\""));

    // Ensure no unexpected field names
    assert!(!json_str.contains("\"po_token\"")); // Should be "poToken", not "po_token"
    assert!(!json_str.contains("\"expires_at\"")); // Should be "expiresAt", not "expires_at"
    assert!(!json_str.contains("\"content_binding\"")); // Should be "contentBinding", not "content_binding"
}

#[test]
fn test_pot_request_optional_fields_compatibility() {
    // Test that optional fields are properly handled
    let request = PotRequest::new()
        .with_content_binding("test_video")
        .with_proxy("http://proxy:8080")
        .with_bypass_cache(true);

    let json = serde_json::to_value(&request).unwrap();

    assert_eq!(json["content_binding"], "test_video");
    assert_eq!(json["proxy"], "http://proxy:8080");
    assert_eq!(json["bypass_cache"], true);
}
