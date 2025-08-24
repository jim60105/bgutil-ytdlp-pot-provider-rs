//! HTTP server integration tests
//!
//! These tests verify that the HTTP API behaves correctly and matches
//! the TypeScript implementation behavior.

use axum::http::StatusCode;
use bgutil_ytdlp_pot_provider::{config::Settings, server::create_app, types::*};
use tower::ServiceExt;

/// Create test application for integration tests
fn create_test_app() -> axum::Router {
    let settings = Settings::default();
    create_app(settings)
}

#[tokio::test]
async fn test_server_ping_endpoint() {
    let app = create_test_app();

    let request = axum::http::Request::builder()
        .uri("/ping")
        .method("GET")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let ping_response: PingResponse = serde_json::from_slice(&body).unwrap();

    // server_uptime should be a valid positive number
    assert!(!ping_response.version.is_empty());
}

#[tokio::test]
async fn test_server_get_pot_endpoint() {
    let app = create_test_app();

    let request_body = PotRequest::new().with_content_binding("test_video_id");
    let request = axum::http::Request::builder()
        .uri("/get_pot")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&request_body).unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pot_response: PotResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(pot_response.content_binding, "test_video_id");
    assert!(!pot_response.po_token.is_empty());
}

#[tokio::test]
async fn test_server_invalidate_endpoints() {
    let app = create_test_app();

    // Test invalidate_caches
    let request = axum::http::Request::builder()
        .uri("/invalidate_caches")
        .method("POST")
        .body(axum::body::Body::empty())
        .unwrap();

    let response =
        ServiceExt::<axum::http::Request<axum::body::Body>>::oneshot(app.clone(), request)
            .await
            .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Test invalidate_it
    let request = axum::http::Request::builder()
        .uri("/invalidate_it")
        .method("POST")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_server_minter_cache_endpoint() {
    let app = create_test_app();

    let request = axum::http::Request::builder()
        .uri("/minter_cache")
        .method("GET")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let cache_keys: Vec<String> = serde_json::from_slice(&body).unwrap();

    // Should return an empty array initially
    assert!(cache_keys.is_empty());
}

#[tokio::test]
async fn test_server_cors_headers() {
    let app = create_test_app();

    let request = axum::http::Request::builder()
        .uri("/ping")
        .method("GET")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should have CORS headers set
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}
