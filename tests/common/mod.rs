//! Common test utilities and helpers
//!
//! This module provides shared utilities for integration tests.

#![allow(dead_code)]

use bgutil_ytdlp_pot_provider::{config::Settings, session::SessionManager, types::*};
use serde_json;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test helper functions
pub mod helpers {
    use super::*;

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

/// Test configuration factory
pub struct TestConfig;

impl TestConfig {
    /// Create minimal test configuration
    pub fn minimal() -> Settings {
        let mut settings = Settings::default();
        settings.server.port = 0; // Use random port
        settings.logging.level = "debug".to_string();
        settings.network.connect_timeout = 5;
        settings.network.request_timeout = 10;
        settings.token.ttl_hours = 1; // Short TTL for testing
        settings
    }

    /// Create configuration with proxy support
    pub fn with_proxy(proxy_url: &str) -> Settings {
        let mut settings = Self::minimal();
        settings.network.https_proxy = Some(proxy_url.to_string());
        settings
    }

    /// Create offline test configuration
    pub fn offline() -> Settings {
        let mut settings = Self::minimal();
        settings.botguard.disable_innertube = true;
        settings.network.max_retries = 0;
        settings
    }
}

/// Test data factory
pub struct MockData;

impl MockData {
    /// Generate sample POT request
    pub fn pot_request() -> PotRequest {
        PotRequest::new().with_content_binding("test_video_123")
    }

    /// Generate sample POT response
    pub fn pot_response() -> PotResponse {
        PotResponse::new(
            "pot_token_12345",
            "test_video_123",
            chrono::Utc::now() + chrono::Duration::hours(6),
        )
    }

    /// Generate BotGuard challenge data
    pub fn botguard_challenge() -> serde_json::Value {
        serde_json::json!({
            "challenge": "Y2hhbGxlbmdl", // base64 "challenge"
            "difficulty": 1,
            "interpreterJavascript": {
                "functionName": "BGChallenge",
                "script": "function BGChallenge(){return 'test';}"
            }
        })
    }
}

/// Mock server factory
pub struct MockServerFactory;

impl MockServerFactory {
    /// Create new mock server
    pub async fn new() -> MockServer {
        MockServer::start().await
    }

    /// Setup BotGuard challenge endpoint
    pub async fn setup_botguard_challenge(server: &MockServer) {
        Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/challenge"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockData::botguard_challenge()))
            .mount(server)
            .await;
    }

    /// Setup error responses for failure testing
    pub async fn setup_error_responses(server: &MockServer) {
        Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/challenge"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(server)
            .await;
    }
}

/// Test utilities
pub struct TestUtils;

impl TestUtils {
    /// Initialize test logging
    pub fn init_logger() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("debug")
            .try_init();
    }

    /// Wait for async condition
    pub async fn wait_for_condition<F, Fut>(
        condition: F,
        timeout: std::time::Duration,
    ) -> anyhow::Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        use tokio::time::{sleep, timeout as tokio_timeout};

        tokio_timeout(timeout, async {
            loop {
                if condition().await {
                    return Ok(());
                }
                sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("Wait condition timeout"))?
    }
}
