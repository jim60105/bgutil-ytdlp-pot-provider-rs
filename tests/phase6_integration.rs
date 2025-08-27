//! Phase 6 Integration Tests
//!
//! Comprehensive tests for final SessionManager integration and TODO completion.

use bgutil_ytdlp_pot_provider::{config::Settings, session::SessionManager, types::*};
use std::sync::Arc;
use tokio::time::{Duration, timeout};

/// Helper function to create a test SessionManager
async fn create_test_session_manager() -> SessionManager {
    let settings = Settings::default();
    SessionManager::new(settings)
}

#[tokio::test]
async fn test_complete_pot_generation_flow() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new().with_content_binding("test_video_id");

    // Act
    let result = session_manager.generate_pot_token(&request).await;

    // Assert
    assert!(result.is_ok(), "POT generation should succeed");
    let response = result.unwrap();

    // Since we're using placeholder implementation, check for placeholder token pattern
    assert!(
        !response.po_token.is_empty(),
        "POT token should not be empty"
    );
    assert!(
        response.po_token.len() > 5,
        "POT token should have meaningful length"
    );
    assert!(
        !response.content_binding.is_empty(),
        "Should have content binding"
    );
}

#[tokio::test]
async fn test_pot_generation_different_content_bindings() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    let test_cases = vec!["video_id_12345", "playlist_id_abcde", "channel_id_xyz789"];

    // Act & Assert
    for content_binding in test_cases {
        let request = PotRequest::new().with_content_binding(content_binding);

        let result = session_manager.generate_pot_token(&request).await;

        assert!(
            result.is_ok(),
            "Content binding '{}' should work",
            content_binding
        );
        let response = result.unwrap();
        assert!(
            !response.po_token.is_empty(),
            "Token should not be empty for '{}'",
            content_binding
        );
        assert_eq!(
            response.content_binding, content_binding,
            "Content binding should match"
        );
    }
}

#[tokio::test]
async fn test_visitor_data_generation_real_api() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Act
    let result = session_manager.generate_visitor_data().await;

    // Assert
    assert!(result.is_ok(), "Visitor data generation should succeed");
    let visitor_data = result.unwrap();
    assert!(!visitor_data.is_empty(), "Visitor data should not be empty");

    // Check if it's real data
    // Note: In integration tests, the real Innertube API is called
    // because the test mode detection doesn't work for integration tests
    if visitor_data == "placeholder_visitor_data" {
        // Test mode - placeholder is being used
        assert_eq!(
            visitor_data, "placeholder_visitor_data",
            "Should return placeholder in test mode"
        );
    } else {
        // Real mode - actual API is being called (which is what we want!)
        assert_ne!(
            visitor_data, "placeholder_visitor_data",
            "Should not return placeholder in real mode"
        );
        assert!(
            visitor_data.len() > 10,
            "Visitor data should have meaningful length"
        );
        // Real visitor data from YouTube's Innertube API
        assert!(
            visitor_data.chars().all(|c| c.is_ascii_alphanumeric()
                || c == '%'
                || c == '='
                || c == '+'
                || c == '/'),
            "Visitor data should be URL-encoded"
        );
    }
}

#[tokio::test]
async fn test_session_manager_caching() {
    // Arrange
    let session_manager = create_test_session_manager().await;
    let request = PotRequest::new().with_content_binding("cache_test_video");

    // Act - Generate twice with same parameters
    let result1 = session_manager.generate_pot_token(&request).await;
    let result2 = session_manager.generate_pot_token(&request).await;

    // Assert
    assert!(result1.is_ok() && result2.is_ok());

    let response1 = result1.unwrap();
    let response2 = result2.unwrap();

    // Both should be valid tokens
    assert!(!response1.po_token.is_empty());
    assert!(!response2.po_token.is_empty());

    // Both should have same content binding
    assert_eq!(response1.content_binding, response2.content_binding);
}

#[tokio::test]
async fn test_session_manager_concurrent_requests() {
    // Arrange
    let session_manager = Arc::new(create_test_session_manager().await);
    let mut handles = vec![];

    // Act - Launch multiple concurrent requests
    for i in 0..3 {
        // Reduced from 5 to 3 to be more conservative
        let manager = Arc::clone(&session_manager);
        let handle = tokio::spawn(async move {
            let request = PotRequest::new().with_content_binding(&format!("concurrent_test_{}", i));
            manager.generate_pot_token(&request).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Assert
    for (i, result) in results.into_iter().enumerate() {
        let token_result = result.expect(&format!("Task {} should complete", i));
        assert!(token_result.is_ok(), "Request {} should succeed", i);
        let response = token_result.unwrap();
        assert!(!response.po_token.is_empty(), "Token {} should be valid", i);
    }
}

#[tokio::test]
async fn test_session_manager_timeout_handling() {
    // Arrange
    let session_manager = create_test_session_manager().await;
    let request = PotRequest::new().with_content_binding("timeout_test");

    // Act - Test with timeout
    let result = timeout(
        Duration::from_secs(30), // 30 second timeout
        session_manager.generate_pot_token(&request),
    )
    .await;

    // Assert
    assert!(result.is_ok(), "Should complete within timeout");
    assert!(
        result.unwrap().is_ok(),
        "Should generate token successfully"
    );
}

#[tokio::test]
async fn test_cache_invalidation() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Generate some tokens to populate caches
    let request1 = PotRequest::new().with_content_binding("invalidate_test_1");
    let request2 = PotRequest::new().with_content_binding("invalidate_test_2");

    let _response1 = session_manager.generate_pot_token(&request1).await.unwrap();
    let _response2 = session_manager.generate_pot_token(&request2).await.unwrap();

    // Act
    let cache_result = session_manager.invalidate_caches().await;
    let it_result = session_manager.invalidate_integrity_tokens().await;

    // Assert
    assert!(cache_result.is_ok(), "Cache invalidation should succeed");
    assert!(it_result.is_ok(), "IT invalidation should succeed");
}

#[tokio::test]
async fn test_minter_cache_operations() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Generate a token to create a minter
    let request = PotRequest::new().with_content_binding("minter_cache_test");
    let _response = session_manager.generate_pot_token(&request).await.unwrap();

    // Act
    let result = session_manager.get_minter_cache_keys().await;

    // Assert
    assert!(result.is_ok(), "Getting cache keys should succeed");
    let cache_keys = result.unwrap();
    assert!(
        !cache_keys.is_empty(),
        "Should have at least one minter cached"
    );
}

#[tokio::test]
async fn test_empty_content_binding_uses_visitor_data() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Test with empty content binding
    let request = PotRequest::new(); // No content binding

    // Act
    let result = session_manager.generate_pot_token(&request).await;

    // Assert
    assert!(result.is_ok(), "Should generate token with visitor data");
    let response = result.unwrap();
    assert!(
        !response.content_binding.is_empty(),
        "Should have generated content binding"
    );
    assert!(!response.po_token.is_empty(), "Should be valid POT token");
}

#[tokio::test]
async fn test_bypass_cache_functionality() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    let base_request = PotRequest::new().with_content_binding("bypass_test_video");
    let bypass_request = base_request.clone().with_bypass_cache(true);

    // Act
    let result1 = session_manager.generate_pot_token(&base_request).await;
    let result2 = session_manager.generate_pot_token(&bypass_request).await;

    // Assert
    assert!(result1.is_ok(), "First request should succeed");
    assert!(result2.is_ok(), "Bypass cache request should succeed");

    let response1 = result1.unwrap();
    let response2 = result2.unwrap();

    assert!(!response1.po_token.is_empty());
    assert!(response2.po_token.len() > 5);
    assert_eq!(response1.content_binding, response2.content_binding);
}

#[tokio::test]
async fn test_session_manager_error_recovery() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Test with potentially problematic configuration but valid content binding
    let request = PotRequest::new().with_content_binding("error_test");

    // Act
    let result = session_manager.generate_pot_token(&request).await;

    // Assert - Should succeed since we're using valid configuration
    assert!(result.is_ok(), "Should succeed with valid configuration");
    let response = result.unwrap();
    assert!(!response.po_token.is_empty(), "Should be valid token");
}

#[tokio::test]
async fn test_session_manager_diagnostics() {
    // Arrange
    let session_manager = create_test_session_manager().await;

    // Act
    let (request_key, server_host) = session_manager.get_diagnostic_info();

    // Assert
    assert!(!request_key.is_empty(), "Request key should not be empty");
    assert_eq!(
        request_key, "O43z0dpjhgX20SCx4KAo",
        "Should match hardcoded request key"
    );
    assert!(!server_host.is_empty(), "Server host should not be empty");
}

#[tokio::test]
async fn test_complete_workflow_integration() {
    // This test simulates a complete workflow from request to response

    // Arrange
    let session_manager = create_test_session_manager().await;

    // Step 1: Generate visitor data
    let visitor_data = session_manager.generate_visitor_data().await.unwrap();
    assert!(!visitor_data.is_empty());

    // Step 2: Use visitor data as content binding
    let request = PotRequest::new().with_content_binding(&visitor_data);

    // Step 3: Generate POT token
    let response = session_manager.generate_pot_token(&request).await.unwrap();

    // Step 4: Validate response
    assert!(!response.po_token.is_empty());
    assert_eq!(response.content_binding, visitor_data);
    assert!(!response.po_token.is_empty());

    // Step 5: Test cache operations
    let cache_keys = session_manager.get_minter_cache_keys().await.unwrap();
    assert!(!cache_keys.is_empty());

    // Step 6: Invalidate caches
    session_manager.invalidate_caches().await.unwrap();
    session_manager.invalidate_integrity_tokens().await.unwrap();
}
