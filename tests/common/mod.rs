//! Common test utilities and helpers
//!
//! This module provides shared utilities for integration tests.

/// Test helper functions
pub mod helpers {
    use bgutil_ytdlp_pot_provider::{config::Settings, session::SessionManager};
    use std::sync::Arc;

    /// Create a test session manager with default settings
    pub fn create_test_session_manager() -> SessionManager {
        let settings = Settings::default();
        SessionManager::new(settings)
    }

    /// Create test settings with custom values
    pub fn create_test_settings(port: u16) -> Settings {
        let mut settings = Settings::default();
        settings.server.port = port;
        settings
    }
}