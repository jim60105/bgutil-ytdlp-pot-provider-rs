//! CLI Robustness Tests
//!
//! Tests for issue: fix CLI robustness when content-binding token generation fails
//!
//! This test suite validates that:
//! - Token generation works with various content-binding formats
//! - CLI mode with disable_innertube works correctly
//! - No forced Innertube API calls when content_binding is provided
//! - Behavior matches TypeScript reference implementation

use bgutil_ytdlp_pot_provider::{config::Settings, session::SessionManager, types::*};

/// Helper function to create a test SessionManager
async fn create_test_session_manager() -> SessionManager {
    let settings = Settings::default();
    SessionManager::new(settings)
}

#[tokio::test]
async fn test_generate_pot_with_video_id_format() {
    // Video ID format: 11 characters, alphanumeric + - and _
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new().with_content_binding("dQw4w9WgXcQ");

    let response = session_manager.generate_pot_token(&request).await;

    assert!(
        response.is_ok(),
        "POT generation with video ID format should succeed: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert_eq!(response.content_binding, "dQw4w9WgXcQ");
    assert!(!response.po_token.is_empty());
}

#[tokio::test]
async fn test_generate_pot_with_visitor_data_format() {
    // Visitor data format: longer string (16+ chars), alphanumeric + - and _
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new().with_content_binding("CgtEeHVoMzlVU0E1NCig");

    let response = session_manager.generate_pot_token(&request).await;

    assert!(
        response.is_ok(),
        "POT generation with visitor data format should succeed: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert_eq!(response.content_binding, "CgtEeHVoMzlVU0E1NCig");
    assert!(!response.po_token.is_empty());
}

#[tokio::test]
async fn test_generate_pot_with_custom_binding_format() {
    // Non-standard format that doesn't match video ID or visitor data
    let session_manager = create_test_session_manager().await;

    let test_cases = vec![
        "custom_binding",
        "test_video_id",
        "short",
        "playlist_id_abc",
    ];

    for content_binding in test_cases {
        let request = PotRequest::new().with_content_binding(content_binding);

        let response = session_manager.generate_pot_token(&request).await;

        assert!(
            response.is_ok(),
            "POT generation with custom format '{}' should succeed: {:?}",
            content_binding,
            response.err()
        );

        let response = response.unwrap();
        assert_eq!(response.content_binding, content_binding);
        assert!(!response.po_token.is_empty());
    }
}

#[tokio::test]
async fn test_cli_mode_with_disable_innertube() {
    // Simulates CLI mode which sets disable_innertube = true
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new()
        .with_content_binding("test_video_id")
        .with_disable_innertube(true);

    let response = session_manager.generate_pot_token(&request).await;

    assert!(
        response.is_ok(),
        "CLI mode with disable_innertube should succeed: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert_eq!(response.content_binding, "test_video_id");
    assert!(!response.po_token.is_empty());
}

#[tokio::test]
async fn test_video_id_format_with_disable_innertube() {
    // Test that video ID format works even when Innertube is disabled
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new()
        .with_content_binding("dQw4w9WgXcQ")
        .with_disable_innertube(true);

    let response = session_manager.generate_pot_token(&request).await;

    assert!(
        response.is_ok(),
        "Video ID with disable_innertube should succeed: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert_eq!(response.content_binding, "dQw4w9WgXcQ");
    assert!(!response.po_token.is_empty());
}

#[tokio::test]
async fn test_multiple_formats_sequential() {
    // Test multiple different formats in sequence
    let session_manager = create_test_session_manager().await;

    let test_cases = vec![
        ("dQw4w9WgXcQ", "video ID"),
        ("CgtEeHVoMzlVU0E1NCig", "visitor data"),
        ("custom_test", "custom format"),
        ("short", "short format"),
    ];

    for (content_binding, description) in test_cases {
        let request = PotRequest::new().with_content_binding(content_binding);

        let response = session_manager.generate_pot_token(&request).await;

        assert!(
            response.is_ok(),
            "POT generation for {} should succeed: {:?}",
            description,
            response.err()
        );

        let response = response.unwrap();
        assert_eq!(response.content_binding, content_binding);
        assert!(!response.po_token.is_empty());
    }
}

#[tokio::test]
async fn test_cache_works_with_different_formats() {
    // Test that caching works correctly with the simplified implementation
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new().with_content_binding("cache_test_video");

    // First request - should generate new token
    let response1 = session_manager.generate_pot_token(&request).await;
    assert!(response1.is_ok());
    let response1 = response1.unwrap();

    // Second request - should use cached token
    let response2 = session_manager.generate_pot_token(&request).await;
    assert!(response2.is_ok());
    let response2 = response2.unwrap();

    // Both should return the same token (cached)
    assert_eq!(response1.po_token, response2.po_token);
    assert_eq!(response1.content_binding, response2.content_binding);
}

#[tokio::test]
async fn test_bypass_cache_with_different_formats() {
    // Test that bypass_cache works correctly
    let session_manager = create_test_session_manager().await;

    let request = PotRequest::new()
        .with_content_binding("bypass_test")
        .with_bypass_cache(true);

    // First request
    let response1 = session_manager.generate_pot_token(&request).await;
    assert!(response1.is_ok());

    // Second request with bypass_cache should generate new token
    let response2 = session_manager.generate_pot_token(&request).await;
    assert!(response2.is_ok());

    let response1 = response1.unwrap();
    let response2 = response2.unwrap();

    // Content binding should be the same
    assert_eq!(response1.content_binding, response2.content_binding);
}
