//! HTTP request handlers
//!
//! Implementation of HTTP endpoints for the POT provider server.

use crate::{
    server::app::AppState,
    types::{ErrorResponse, PingResponse, PotRequest},
    utils::version,
};
use axum::{Json as RequestJson, extract::State, http::StatusCode, response::Json};

/// Generate POT token endpoint
///
/// POST /get_pot
///
/// Generates a new POT token based on the request parameters.
pub async fn generate_pot(
    State(state): State<AppState>,
    RequestJson(request): RequestJson<PotRequest>,
) -> Result<Json<crate::types::PotResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!("Received POT generation request: {:?}", request);

    // Validate deprecated fields (matching TypeScript validation)
    if let Err(error_response) = validate_deprecated_fields(&request) {
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    match state.session_manager.generate_pot_token(&request).await {
        Ok(response) => {
            tracing::info!(
                "Successfully generated POT token for content_binding: {:?}",
                request.content_binding
            );
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to generate POT token: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format_error(&e))),
            ))
        }
    }
}

/// Validate deprecated fields in the request
/// 
/// Checks for deprecated data_sync_id and visitor_data fields
fn validate_deprecated_fields(_request: &PotRequest) -> Result<(), ErrorResponse> {
    // Note: Since we're using a structured PotRequest, we need to check if the raw JSON
    // would contain these deprecated fields. For now, we'll implement this check in a simple way.
    // In a full implementation, this would require custom deserialization or middleware.
    
    // For now, return Ok since the structured request doesn't contain these fields
    // TODO: Implement proper JSON field validation for deprecated fields
    Ok(())
}

/// Format error for HTTP response
/// 
/// Corresponds to TypeScript `strerror` function in `utils.ts`
fn format_error(error: &crate::Error) -> String {
    match error {
        crate::Error::BotGuard { message } => format!("BotGuard error: {}", message),
        crate::Error::TokenGeneration(msg) => format!("Token generation failed: {}", msg),
        crate::Error::IntegrityToken { details } => format!("Integrity token error: {}", details),
        crate::Error::Challenge { stage } => format!("Challenge processing failed at {}", stage),
        crate::Error::Proxy { config } => format!("Proxy configuration error: {}", config),
        crate::Error::Network(e) => format!("Network error: {}", e),
        crate::Error::Json(e) => format!("JSON error: {}", e),
        crate::Error::Io(e) => format!("I/O error: {}", e),
        crate::Error::DateParse(e) => format!("Date parsing error: {}", e),
        crate::Error::Cache { operation } => format!("Cache operation failed: {}", operation),
        crate::Error::Config(msg) => format!("Configuration error: {}", msg),
        crate::Error::VisitorData { reason } => format!("Visitor data generation failed: {}", reason),
        crate::Error::Internal(msg) => format!("Internal error: {}", msg),
        crate::Error::Session(msg) => format!("Session error: {}", msg),
        crate::Error::Server(msg) => format!("Server error: {}", msg),
    }
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
    state.session_manager.invalidate_caches().await;
    StatusCode::NO_CONTENT
}

/// Invalidate integrity tokens endpoint
///
/// POST /invalidate_it
///
/// Invalidates integrity tokens to force regeneration.
pub async fn invalidate_it(State(state): State<AppState>) -> StatusCode {
    tracing::info!("Invalidating integrity tokens");
    state.session_manager.invalidate_integrity_tokens().await;
    StatusCode::NO_CONTENT
}

/// Get minter cache keys endpoint
///
/// GET /minter_cache
///
/// Returns the current minter cache keys for debugging.
pub async fn minter_cache(State(state): State<AppState>) -> Json<Vec<String>> {
    tracing::debug!("Retrieving minter cache keys");
    let cache_keys = state.session_manager.get_minter_cache_keys().await;
    Json(cache_keys)
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

        let result = generate_pot(State(state), RequestJson(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.content_binding, "test_video");
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
        assert!(response.is_empty());
    }
}
