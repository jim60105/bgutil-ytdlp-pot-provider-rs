//! Innertube API integration for visitor data generation
//!
//! This module handles communication with YouTube's internal Innertube API
//! to generate visitor data and retrieve challenge information.

use crate::{Result, types::*};
use reqwest::Client;

/// Innertube API client
#[derive(Debug)]
pub struct InnertubeClient {
    /// HTTP client
    client: Client,
    /// Base URL for Innertube API
    base_url: String,
}

impl InnertubeClient {
    /// Create new Innertube client
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://www.youtube.com/youtubei/v1".to_string(),
        }
    }

    /// Generate visitor data
    ///
    /// Corresponds to TypeScript: `generateVisitorData` method (L230-241)
    pub async fn generate_visitor_data(&self) -> Result<String> {
        use serde_json::json;

        let request_body = json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20240822.03.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": "FEwhat_to_watch"
        });

        let response = self
            .client
            .post(format!("{}/browse", self.base_url))
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send request to Innertube API: {}", e);
                crate::Error::VisitorData {
                    reason: format!("Network request failed: {}", e),
                    context: Some("innertube".to_string()),
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!("Innertube API returned error status: {}", status);
            return Err(crate::Error::VisitorData {
                reason: format!("API request failed with status: {}", status),
                context: Some("innertube".to_string()),
            });
        }

        let json_response: serde_json::Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse Innertube API response: {}", e);
            crate::Error::VisitorData {
                reason: format!("Failed to parse JSON response: {}", e),
                context: Some("innertube".to_string()),
            }
        })?;

        let visitor_data = json_response
            .get("responseContext")
            .and_then(|ctx| ctx.get("visitorData"))
            .and_then(|data| data.as_str())
            .ok_or_else(|| {
                tracing::error!("Visitor data not found in Innertube API response");
                crate::Error::VisitorData {
                    reason: "Visitor data not found in API response".to_string(),
                    context: Some("innertube".to_string()),
                }
            })?;

        tracing::debug!("Successfully generated visitor data: {}", visitor_data);
        Ok(visitor_data.to_string())
    }

    /// Get challenge from /att/get endpoint
    ///
    /// Note: Challenge retrieval from Innertube is handled separately by BotGuardManager.
    /// This method is kept for API completeness but may not be needed immediately.
    pub async fn get_challenge(&self, _context: &InnertubeContext) -> Result<ChallengeData> {
        // TODO: Evaluate if this is needed separate from BotGuardManager's implementation
        // Currently BotGuardManager handles Innertube challenge retrieval directly
        tracing::debug!("Challenge retrieval through InnertubeClient not currently needed");
        Err(crate::Error::challenge(
            "innertube",
            "Challenge retrieval handled by BotGuardManager",
        ))
    }

    /// Get client configuration for diagnostics
    pub fn get_client_info(&self) -> (String, bool) {
        (
            self.base_url.clone(),
            format!("{:?}", self.client).contains("Client"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_innertube_client_creation() {
        let client = Client::new();
        let innertube = InnertubeClient::new(client);
        assert_eq!(innertube.base_url, "https://www.youtube.com/youtubei/v1");
    }

    #[tokio::test]
    async fn test_generate_visitor_data_success() {
        // Arrange
        let mock_server = MockServer::start().await;
        let visitor_data = "CgtDZjBSbE5uZDJlQSij6bbFBjIKCgJVUxIEGgAgYA%3D%3D";

        let expected_request = json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20240822.03.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": "FEwhat_to_watch"
        });

        let mock_response = json!({
            "responseContext": {
                "visitorData": visitor_data
            }
        });

        Mock::given(method("POST"))
            .and(path("/youtubei/v1/browse"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let mut innertube = InnertubeClient::new(client);
        innertube.base_url = mock_server.uri() + "/youtubei/v1";

        // Act
        let result = innertube.generate_visitor_data().await;

        // Assert
        assert!(result.is_ok());
        let generated_visitor_data = result.unwrap();
        assert_eq!(generated_visitor_data, visitor_data);
        assert!(!generated_visitor_data.is_empty());
    }

    #[tokio::test]
    async fn test_generate_visitor_data_network_error() {
        // Arrange
        let client = Client::new();
        let mut innertube = InnertubeClient::new(client);
        innertube.base_url = "http://invalid-url-that-does-not-exist".to_string();

        // Act
        let result = innertube.generate_visitor_data().await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Check that it's a VisitorData error with network-related message
        let error_str = error.to_string();
        assert!(
            error_str.contains("Visitor data generation failed")
                || error_str.contains("Network request failed")
        );
    }

    #[tokio::test]
    async fn test_generate_visitor_data_api_error() {
        // Arrange
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/youtubei/v1/browse"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let mut innertube = InnertubeClient::new(client);
        innertube.base_url = mock_server.uri() + "/youtubei/v1";

        // Act
        let result = innertube.generate_visitor_data().await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_visitor_data_missing_visitor_data() {
        // Arrange
        let mock_server = MockServer::start().await;

        let mock_response = json!({
            "responseContext": {}
        });

        Mock::given(method("POST"))
            .and(path("/youtubei/v1/browse"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let mut innertube = InnertubeClient::new(client);
        innertube.base_url = mock_server.uri() + "/youtubei/v1";

        // Act
        let result = innertube.generate_visitor_data().await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = error.to_string();
        assert!(
            error_str.contains("Visitor data generation failed")
                || error_str.contains("not found in API response")
        );
    }

    #[tokio::test]
    async fn test_get_challenge() {
        let client = Client::new();
        let innertube = InnertubeClient::new(client);

        let context = InnertubeContext::default();
        let result = innertube.get_challenge(&context).await;

        // Should fail as challenge retrieval is handled by BotGuardManager
        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(
            error_str.contains("Challenge processing failed")
                || error_str.contains("BotGuardManager")
        );
    }

    #[tokio::test]
    async fn test_innertube_client_fields_usage() {
        let client = Client::new();
        let innertube = InnertubeClient::new(client);

        // Verify field accessibility through diagnostic method
        let (base_url, has_client) = innertube.get_client_info();
        assert!(!base_url.is_empty());
        assert!(base_url.contains("youtube.com"));
        assert!(has_client);
    }
}
