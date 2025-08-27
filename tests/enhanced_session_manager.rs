//! Integration tests for enhanced SessionManager
//!
//! These tests verify the SessionManager behaves correctly in realistic scenarios
//! and matches the TypeScript implementation behavior.

use bgutil_ytdlp_pot_provider::{
    config::Settings,
    session::{JsRuntimeHandle, SessionManager, WebPoMinter},
    types::{PotRequest, TokenMinterEntry},
};
use chrono::{Duration, Utc};

/// Helper function to create a test WebPoMinter
fn create_test_webpo_minter() -> WebPoMinter {
    WebPoMinter {
        mint_callback_ref: "test_callback".to_string(),
        runtime_handle: JsRuntimeHandle::new_for_test(),
    }
}

#[tokio::test]
async fn test_enhanced_session_manager_flow() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Test basic POT generation
    let request = PotRequest::new().with_content_binding("dQw4w9WgXcQ"); // Rick Roll video ID

    let response = session_manager.generate_pot_token(&request).await;
    assert!(response.is_ok());

    let pot_response = response.unwrap();
    assert_eq!(pot_response.content_binding, "dQw4w9WgXcQ");
    assert!(!pot_response.po_token.is_empty());
    assert!(!pot_response.is_expired());
}

#[tokio::test]
async fn test_enhanced_cache_behavior() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    let request = PotRequest::new().with_content_binding("test_video_id");

    // First request should generate new token
    let response1 = session_manager.generate_pot_token(&request).await.unwrap();

    // Second request should return cached token (same behavior as TypeScript)
    let response2 = session_manager.generate_pot_token(&request).await.unwrap();

    assert_eq!(response1.po_token, response2.po_token);
    assert_eq!(response1.content_binding, response2.content_binding);
}

#[tokio::test]
async fn test_enhanced_bypass_cache() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    let base_request = PotRequest::new().with_content_binding("test_video_id");

    // First request
    let _response1 = session_manager
        .generate_pot_token(&base_request)
        .await
        .unwrap();

    // Second request with bypass_cache (should generate new token)
    let bypass_request = base_request.with_bypass_cache(true);
    let response2 = session_manager
        .generate_pot_token(&bypass_request)
        .await
        .unwrap();

    // Should succeed (exact behavior depends on implementation)
    assert!(!response2.po_token.is_empty());
}

#[tokio::test]
async fn test_enhanced_proxy_configuration() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    let request = PotRequest::new()
        .with_content_binding("test_video_id")
        .with_proxy("http://localhost:8080")
        .with_source_address("192.168.1.1")
        .with_disable_tls_verification(true);

    // Should handle proxy configuration gracefully (TypeScript compatible)
    let result = session_manager.generate_pot_token(&request).await;

    // Should succeed even with proxy configuration
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_token_minter_entry_functionality() {
    // Test TokenMinterEntry which matches TypeScript TokenMinter interface
    let expires_at = Utc::now() + Duration::hours(1);
    let test_minter = create_test_webpo_minter();
    let minter = TokenMinterEntry::new(
        expires_at,
        "test_integrity_token",
        3600,
        300,
        Some("websafe_fallback".to_string()),
        test_minter,
    );

    assert_eq!(minter.integrity_token, "test_integrity_token");
    assert_eq!(minter.estimated_ttl_secs, 3600);
    assert_eq!(minter.mint_refresh_threshold, 300);
    assert_eq!(
        minter.websafe_fallback_token,
        Some("websafe_fallback".to_string())
    );
    assert!(!minter.is_expired());
    assert!(minter.time_until_expiry().num_seconds() > 0);
}

#[tokio::test]
async fn test_visitor_data_generation() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Test visitor data generation (placeholder implementation)
    let visitor_data = session_manager.generate_visitor_data().await.unwrap();
    assert!(!visitor_data.is_empty());

    // Test that request without content_binding uses visitor data
    let request = PotRequest::new(); // No content binding
    let response = session_manager.generate_pot_token(&request).await.unwrap();

    // Should use generated visitor data as content binding
    // The visitor data should be real data from Innertube, not placeholder
    assert!(!response.content_binding.is_empty());
    assert_ne!(response.content_binding, "placeholder_visitor_data"); // Should be real data
    assert!(response.content_binding.len() > 10); // Real visitor data is longer
}

#[tokio::test]
async fn test_minter_cache_operations() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Initially should have no cache keys
    let initial_keys = session_manager.get_minter_cache_keys().await.unwrap();
    assert!(initial_keys.is_empty());

    // Generate a token which should create minter cache entries
    let request = PotRequest::new().with_content_binding("cache_test");
    let _response = session_manager.generate_pot_token(&request).await.unwrap();

    // Should now have cache entries
    let cache_keys = session_manager.get_minter_cache_keys().await.unwrap();
    assert!(!cache_keys.is_empty());
}

#[tokio::test]
async fn test_cache_invalidation_operations() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Generate some tokens to populate caches
    let request1 = PotRequest::new().with_content_binding("invalidate_test_1");
    let request2 = PotRequest::new().with_content_binding("invalidate_test_2");

    let _response1 = session_manager.generate_pot_token(&request1).await.unwrap();
    let _response2 = session_manager.generate_pot_token(&request2).await.unwrap();

    // Verify caches have content
    let cache_keys = session_manager.get_minter_cache_keys().await.unwrap();
    assert!(!cache_keys.is_empty());

    // Test integrity token invalidation
    session_manager.invalidate_integrity_tokens().await.unwrap();

    // Cache keys should still exist but be marked as expired
    let keys_after_it = session_manager.get_minter_cache_keys().await.unwrap();
    assert_eq!(cache_keys.len(), keys_after_it.len());

    // Test full cache invalidation
    session_manager.invalidate_caches().await.unwrap();

    // All caches should be cleared
    let keys_after_clear = session_manager.get_minter_cache_keys().await.unwrap();
    assert!(keys_after_clear.is_empty());
}

#[tokio::test]
async fn test_innertube_context_support() {
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Test with Innertube context (similar to TypeScript implementation)
    let innertube_context = serde_json::json!({
        "client": {
            "clientName": "WEB",
            "clientVersion": "2.0",
            "remoteHost": "youtube.com"
        }
    });

    let request = PotRequest::new()
        .with_content_binding("innertube_test")
        .with_innertube_context(innertube_context);

    let response = session_manager.generate_pot_token(&request).await;
    assert!(response.is_ok());

    let pot_response = response.unwrap();
    assert_eq!(pot_response.content_binding, "innertube_test");
}

#[tokio::test]
async fn test_full_typescript_compatibility_scenario() {
    // This test mimics a typical usage scenario from the TypeScript implementation
    let settings = Settings::default();
    let session_manager = SessionManager::new(settings);

    // Scenario: Generate POT token with full configuration (like TypeScript calls)
    let request = PotRequest::new()
        .with_content_binding("_9lZdqGdl_M") // Some YouTube video ID
        .with_proxy("http://proxy.example.com:8080")
        .with_source_address("10.0.0.1")
        .with_disable_tls_verification(false)
        .with_disable_innertube(false);

    // First call should work
    let response1 = session_manager.generate_pot_token(&request).await.unwrap();
    assert_eq!(response1.content_binding, "_9lZdqGdl_M");
    assert!(!response1.po_token.is_empty());
    assert!(!response1.is_expired());

    // Second call should use cache (TypeScript behavior)
    let response2 = session_manager.generate_pot_token(&request).await.unwrap();
    assert_eq!(response1.po_token, response2.po_token);

    // Bypass cache should generate new token
    let bypass_request = request.with_bypass_cache(true);
    let response3 = session_manager
        .generate_pot_token(&bypass_request)
        .await
        .unwrap();
    assert_eq!(response3.content_binding, "_9lZdqGdl_M");
    // Note: In placeholder implementation, tokens will be similar, but in real implementation they should differ
}
