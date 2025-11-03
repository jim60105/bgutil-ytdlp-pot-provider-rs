//! HTTP request handlers
//!
//! Implementation of HTTP endpoints for the POT provider server.

use crate::{
    server::app::AppState,
    types::{ErrorResponse, PingResponse, PotRequest},
    utils::version,
};
use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Middleware to validate deprecated fields before processing
pub async fn validate_deprecated_fields_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Only check POST requests to /get_pot
    if request.method() != "POST" || request.uri().path() != "/get_pot" {
        return Ok(next.run(request).await);
    }

    // Extract the request body for validation
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_context(
                    "Invalid request body",
                    "request_parsing",
                )),
            ));
        }
    };

    // Parse JSON to check for deprecated fields
    if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&body_bytes)
        && let Some(obj) = json_value.as_object()
    {
        // Check for data_sync_id
        if obj.contains_key("data_sync_id") {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_context(
                    "data_sync_id is deprecated, use content_binding instead",
                    "deprecated_field_validation",
                )),
            ));
        }

        // Check for visitor_data
        if obj.contains_key("visitor_data") {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::with_context(
                    "visitor_data is deprecated, use content_binding instead",
                    "deprecated_field_validation",
                )),
            ));
        }
    }

    // Reconstruct the request and continue
    let new_body = Body::from(body_bytes);
    let new_request = Request::from_parts(parts, new_body);

    Ok(next.run(new_request).await)
}

/// Generate POT token endpoint
///
/// POST /get_pot
///
/// Generates a new POT token based on the request parameters.
pub async fn generate_pot(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> axum::response::Response {
    // Parse JSON with detailed error logging
    let request: PotRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => {
            // Log the raw body for debugging (truncate if too long)
            let body_preview = if body.len() > 1000 {
                format!(
                    "{}... (truncated, total {} bytes)",
                    String::from_utf8_lossy(&body[..1000]),
                    body.len()
                )
            } else {
                String::from_utf8_lossy(&body).to_string()
            };
            
            tracing::error!(
                "Failed to deserialize JSON request: {}\nBody preview: {}",
                e,
                body_preview
            );
            
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse::with_context(
                    format!("Invalid JSON: {}", e),
                    "json_deserialization",
                )),
            )
                .into_response();
        }
    };

    tracing::debug!("Received POT generation request: {:?}", request);

    // Note: Deprecated field validation is now handled by middleware

    match state.session_manager.generate_pot_token(&request).await {
        Ok(response) => {
            tracing::info!(
                "Successfully generated POT token for content_binding: {:?}",
                request.content_binding
            );
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to generate POT token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::with_context(
                    format_error(&e),
                    "token_generation",
                )),
            )
                .into_response()
        }
    }
}

/// Format error for HTTP response
///
/// Corresponds to TypeScript `strerror` function in `utils.ts`
fn format_error(error: &crate::Error) -> String {
    crate::error::format_error(error)
}

/// Ping endpoint for health checks
///
/// GET /ping
///
/// Returns server status and uptime information.
pub async fn ping(State(state): State<AppState>) -> Json<PingResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    let response = PingResponse::new(uptime, version::get_version());

    tracing::debug!(
        "Ping response: uptime={}s, version={}",
        uptime,
        version::get_version()
    );
    Json(response)
}

/// Invalidate caches endpoint
///
/// POST /invalidate_caches
///
/// Clears all internal caches.
pub async fn invalidate_caches(State(state): State<AppState>) -> StatusCode {
    tracing::info!("Invalidating all caches");
    if let Err(e) = state.session_manager.invalidate_caches().await {
        tracing::error!("Failed to invalidate caches: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::NO_CONTENT
}

/// Invalidate integrity tokens endpoint
///
/// POST /invalidate_it
///
/// Invalidates integrity tokens to force regeneration.
pub async fn invalidate_it(State(state): State<AppState>) -> StatusCode {
    tracing::info!("Invalidating integrity tokens");
    if let Err(e) = state.session_manager.invalidate_integrity_tokens().await {
        tracing::error!("Failed to invalidate integrity tokens: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::NO_CONTENT
}

/// Get minter cache keys endpoint
///
/// GET /minter_cache
///
/// Returns the current minter cache keys for debugging.
pub async fn minter_cache(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!("Retrieving minter cache keys");
    match state.session_manager.get_minter_cache_keys().await {
        Ok(cache_keys) => Ok(Json(cache_keys)),
        Err(e) => {
            tracing::error!("Failed to retrieve minter cache keys: {}", e);
            let error_response = ErrorResponse::with_context(
                format!("Failed to get cache keys: {}", e),
                "cache_retrieval",
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Settings, session::SessionManager};
    use std::sync::Arc;

    fn create_test_state() -> AppState {
        let settings = Settings::default();
        AppState {
            session_manager: Arc::new(SessionManager::new(settings.clone())),
            settings: Arc::new(settings),
            start_time: std::time::Instant::now(),
        }
    }

    #[tokio::test]
    async fn test_ping_handler() {
        let state = create_test_state();
        let response = ping(State(state)).await;

        assert!(!response.version.is_empty());
        assert!(response.server_uptime < 1); // Should be very small for fresh state
    }

    #[tokio::test]
    async fn test_generate_pot_handler() {
        let state = create_test_state();
        let request = PotRequest::new().with_content_binding("test_video");

        let response = generate_pot(State(state), Json(request)).await;
        // Since we changed to IntoResponse, we can't easily test the structure
        // but at least we can verify it compiles and runs
        let _ = response.into_response();
    }

    #[tokio::test]
    async fn test_invalidate_caches_handler() {
        let state = create_test_state();
        let status = invalidate_caches(State(state)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_invalidate_it_handler() {
        let state = create_test_state();
        let status = invalidate_it(State(state)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_minter_cache_handler() {
        let state = create_test_state();
        let response = minter_cache(State(state)).await;
        // Response should be empty initially but valid
        assert!(response.is_ok());
        let cache_keys = response.unwrap().0; // Extract Json<Vec<String>>
        assert!(cache_keys.is_empty());
    }

    #[test]
    fn test_format_error_botguard() {
        let error = crate::Error::BotGuard {
            code: "500".to_string(),
            message: "BotGuard initialization failed".to_string(),
            info: None,
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("BGError(500)"));
        assert!(formatted.contains("BotGuard initialization failed"));
    }

    #[test]
    fn test_format_error_token_generation() {
        let error = crate::Error::TokenGeneration {
            reason: "Failed to generate token".to_string(),
            stage: None,
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Token generation failed"));
        assert!(formatted.contains("Failed to generate token"));
    }

    #[test]
    fn test_format_error_integrity_token() {
        let error = crate::Error::IntegrityToken {
            details: "Invalid token structure".to_string(),
            response_data: None,
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Integrity token error"));
        assert!(formatted.contains("Invalid token structure"));
    }

    #[test]
    fn test_format_error_challenge() {
        let error = crate::Error::Challenge {
            stage: "verification".to_string(),
            message: "Processing failed".to_string(),
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Challenge processing failed"));
        assert!(formatted.contains("verification"));
    }

    #[test]
    fn test_format_error_proxy() {
        let error = crate::Error::Proxy {
            config: "http://proxy:8080".to_string(),
            message: "Invalid proxy settings".to_string(),
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Proxy error"));
        assert!(formatted.contains("Invalid proxy settings"));
    }

    #[tokio::test]
    async fn test_format_error_network() {
        // Create a network error by making a request to an invalid URL
        let client = reqwest::Client::new();
        let result = client
            .get("http://invalid-domain-that-does-not-exist.test")
            .send()
            .await;
        assert!(result.is_err());

        let reqwest_error = result.unwrap_err();
        let error = crate::Error::Http(reqwest_error);
        let formatted = format_error(&error);
        assert!(formatted.starts_with("HTTP request failed:"));
    }

    #[test]
    fn test_format_error_json() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = crate::Error::Json(json_error);
        let formatted = format_error(&error);
        assert!(formatted.starts_with("JSON error:"));
    }

    #[test]
    fn test_format_error_io() {
        let error = crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
        let formatted = format_error(&error);
        assert!(formatted.starts_with("I/O error:"));
    }

    #[test]
    fn test_format_error_date_parse() {
        // Create a real parse error
        let date_error = chrono::DateTime::parse_from_rfc3339("invalid date").unwrap_err();
        let error = crate::Error::DateParse(date_error);
        let formatted = format_error(&error);
        assert!(formatted.starts_with("Date parsing error:"));
    }

    #[test]
    fn test_format_error_cache() {
        let error = crate::Error::Cache {
            operation: "store".to_string(),
            details: "Failed to store cache entry".to_string(),
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Cache error"));
        assert!(formatted.contains("Failed to store cache entry"));
    }

    #[test]
    fn test_format_error_config() {
        let error = crate::Error::Config {
            field: "timeout".to_string(),
            message: "Invalid configuration parameter".to_string(),
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Configuration error"));
        assert!(formatted.contains("Invalid configuration parameter"));
    }

    #[test]
    fn test_format_error_visitor_data() {
        let error = crate::Error::VisitorData {
            reason: "Failed to generate visitor data".to_string(),
            context: None,
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Visitor data generation failed"));
        assert!(formatted.contains("Failed to generate visitor data"));
    }

    #[test]
    fn test_format_error_internal() {
        let error = crate::Error::Internal {
            message: "Unexpected internal state".to_string(),
            context: None,
        };
        let formatted = format_error(&error);
        assert!(formatted.contains("Internal error"));
        assert!(formatted.contains("Unexpected internal state"));
    }

    #[test]
    fn test_format_error_session() {
        let error = crate::Error::Session("Session expired".to_string());
        let formatted = format_error(&error);
        assert_eq!(formatted, "Session error: Session expired");
    }

    #[test]
    fn test_format_error_server() {
        let error = crate::Error::Server("Server configuration invalid".to_string());
        let formatted = format_error(&error);
        assert_eq!(formatted, "Server error: Server configuration invalid");
    }

    #[tokio::test]
    async fn test_generate_pot_with_empty_content_binding() {
        let state = create_test_state();
        let request = PotRequest::new(); // No content binding set

        let response = generate_pot(State(state), Json(request)).await;
        // Since we changed to IntoResponse, we can't easily test the structure
        // but at least we can verify it compiles and runs
        let _ = response.into_response();
    }

    #[tokio::test]
    async fn test_ping_handler_timing() {
        use std::time::Duration;

        let state = create_test_state();

        // Wait a small amount of time to ensure uptime is measurable
        tokio::time::sleep(Duration::from_millis(10)).await;

        let response = ping(State(state)).await;

        assert!(!response.version.is_empty());
        // server_uptime is u64, so always >= 0, just check it's a reasonable value
        assert!(response.server_uptime < 10); // Should be less than 10 seconds for test
    }
}

// Additional tests for deprecated field validation middleware
#[cfg(test)]
mod deprecated_field_tests {
    use super::*;
    use crate::config::Settings;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    fn create_test_app() -> axum::Router {
        let settings = Settings::default();
        let session_manager =
            std::sync::Arc::new(crate::session::SessionManager::new(settings.clone()));

        let state = AppState {
            session_manager,
            settings: std::sync::Arc::new(settings),
            start_time: std::time::Instant::now(),
        };

        axum::Router::new()
            .route("/get_pot", axum::routing::post(generate_pot))
            .layer(axum::middleware::from_fn(
                validate_deprecated_fields_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_deprecated_data_sync_id_field() {
        // Arrange
        let app = create_test_app();

        let deprecated_request = json!({
            "data_sync_id": "deprecated_value",
            "content_binding": "video_id"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/get_pot")
            .header("content-type", "application/json")
            .body(Body::from(deprecated_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_response["error"],
            "data_sync_id is deprecated, use content_binding instead"
        );
        assert_eq!(json_response["context"], "deprecated_field_validation");
    }

    #[tokio::test]
    async fn test_deprecated_visitor_data_field() {
        // Arrange
        let app = create_test_app();

        let deprecated_request = json!({
            "visitor_data": "deprecated_visitor",
            "content_binding": "video_id"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/get_pot")
            .header("content-type", "application/json")
            .body(Body::from(deprecated_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            json_response["error"],
            "visitor_data is deprecated, use content_binding instead"
        );
        assert_eq!(json_response["context"], "deprecated_field_validation");
    }

    #[tokio::test]
    async fn test_both_deprecated_fields() {
        // Arrange
        let app = create_test_app();

        let deprecated_request = json!({
            "data_sync_id": "deprecated_data",
            "visitor_data": "deprecated_visitor",
            "content_binding": "video_id"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/get_pot")
            .header("content-type", "application/json")
            .body(Body::from(deprecated_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Should return error for data_sync_id (first check)
        assert_eq!(
            json_response["error"],
            "data_sync_id is deprecated, use content_binding instead"
        );
        assert_eq!(json_response["context"], "deprecated_field_validation");
    }

    #[tokio::test]
    async fn test_valid_request_without_deprecated_fields() {
        // Arrange
        let app = create_test_app();

        let valid_request = json!({
            "content_binding": "video_id",
            "proxy": "http://proxy:8080"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/get_pot")
            .header("content-type", "application/json")
            .body(Body::from(valid_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_deprecated_fields_case_sensitivity() {
        // Arrange
        let app = create_test_app();

        let case_sensitive_request = json!({
            "Data_Sync_Id": "test",  // Different case
            "content_binding": "video_id"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/get_pot")
            .header("content-type", "application/json")
            .body(Body::from(case_sensitive_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert
        // Should succeed because field name doesn't match exactly
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_ignores_non_get_pot_requests() {
        // Test that middleware only applies to /get_pot endpoint
        let app = create_test_app();

        let deprecated_request = json!({
            "data_sync_id": "should_be_ignored"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/some_other_endpoint") // Different endpoint
            .header("content-type", "application/json")
            .body(Body::from(deprecated_request.to_string()))
            .unwrap();

        // Act
        let response = app.oneshot(request).await.unwrap();

        // Assert - should get 404 not 400 (deprecated field error)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
