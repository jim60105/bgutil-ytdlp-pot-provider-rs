//! Basic integration tests
//!
//! Tests the basic functionality of the POT provider.

use bgutil_ytdlp_pot_provider::{
    config::Settings,
    session::SessionManager,
    types::PotRequest,
};

mod common;
use common::helpers;

#[tokio::test]
async fn test_basic_token_generation() {
    let session_manager = helpers::create_test_session_manager();
    
    let request = PotRequest::new()
        .with_content_binding("integration_test_video");
    
    let response = session_manager.generate_pot_token(&request).await;
    assert!(response.is_ok());
    
    let pot_response = response.unwrap();
    assert_eq!(pot_response.content_binding, "integration_test_video");
    assert!(!pot_response.po_token.is_empty());
    assert!(!pot_response.is_expired());
}

#[tokio::test] 
async fn test_visitor_data_generation() {
    let session_manager = helpers::create_test_session_manager();
    
    let visitor_data = session_manager.generate_visitor_data().await;
    assert!(visitor_data.is_ok());
    
    let data = visitor_data.unwrap();
    assert!(!data.is_empty());
}

#[tokio::test]
async fn test_cache_invalidation() {
    let session_manager = helpers::create_test_session_manager();
    
    // Generate a token to populate cache
    let request = PotRequest::new()
        .with_content_binding("cache_test_video");
    
    let _response = session_manager.generate_pot_token(&request).await.unwrap();
    
    // Invalidate caches should not fail
    session_manager.invalidate_caches().await;
    
    // Should be able to generate new token after invalidation
    let response2 = session_manager.generate_pot_token(&request).await;
    assert!(response2.is_ok());
}

#[test]
fn test_settings_creation() {
    let settings = Settings::default();
    assert_eq!(settings.server.port, 4416);
    assert_eq!(settings.server.host, "::");
    assert_eq!(settings.token.ttl_hours, 6);
}

#[test]
fn test_custom_settings() {
    let settings = helpers::create_test_settings(8080);
    assert_eq!(settings.server.port, 8080);
}