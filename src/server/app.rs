//! Axum application setup
//!
//! Creates and configures the Axum application with routes and middleware.

use crate::{config::Settings, session::SessionManager};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Session manager for token generation
    pub session_manager: Arc<SessionManager>,
    /// Application settings
    pub settings: Arc<Settings>,
    /// Server start time for uptime calculation
    pub start_time: std::time::Instant,
}

/// Create the main Axum application with routes and middleware
pub fn create_app(settings: Settings) -> Router {
    let session_manager = Arc::new(SessionManager::new(settings.clone()));

    let state = AppState {
        session_manager,
        settings: Arc::new(settings),
        start_time: std::time::Instant::now(),
    };

    Router::new()
        .route("/get_pot", post(super::handlers::generate_pot))
        .route("/ping", get(super::handlers::ping))
        .route(
            "/invalidate_caches",
            post(super::handlers::invalidate_caches),
        )
        .route("/minter_cache", get(super::handlers::minter_cache))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_app() {
        let settings = Settings::default();
        let _app = create_app(settings);

        // The app should be created successfully
        // More detailed testing would require setting up a test server
        assert!(true); // Placeholder assertion
    }
}
