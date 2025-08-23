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

    // Validate deprecated fields
    // Note: The TypeScript version checks for data_sync_id and visitor_data
    // We should implement similar validation here if needed

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
                Json(ErrorResponse::new(e.to_string())),
            ))
        }
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

/// Get minter cache keys endpoint
///
/// GET /minter_cache
///
/// Returns the current minter cache keys for debugging.
pub async fn minter_cache(State(_state): State<AppState>) -> Json<Vec<String>> {
    // TODO: Implement actual minter cache key retrieval
    tracing::debug!("Returning minter cache keys");
    Json(vec!["placeholder_key".to_string()])
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
    async fn test_minter_cache_handler() {
        let state = create_test_state();
        let response = minter_cache(State(state)).await;
        assert!(!response.is_empty());
    }
}
