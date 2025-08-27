//! Innertube API integration for visitor data generation
//!
//! This module handles communication with YouTube's internal Innertube API
//! to generate visitor data and retrieve challenge information.

use crate::Result;
use reqwest::Client;

/// Trait for Innertube API operations to enable testing with mocks
#[async_trait::async_trait]
pub trait InnertubeProvider {
    /// Generate visitor data from YouTube's Innertube API
    async fn generate_visitor_data(&self) -> Result<String>;

    /// Get challenge data from Innertube /att/get endpoint
    async fn get_challenge(
        &self,
        context: &crate::types::InnertubeContext,
    ) -> crate::Result<crate::types::ChallengeData>;
}

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

    /// Create new Innertube client with custom base URL (for testing)
    pub fn new_with_base_url(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }
}

#[async_trait::async_trait]
impl InnertubeProvider for InnertubeClient {
    /// Generate visitor data
    ///
    /// Corresponds to TypeScript: `generateVisitorData` method (L230-241)
    async fn generate_visitor_data(&self) -> Result<String> {
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

    /// Get challenge data from Innertube /att/get endpoint
    ///
    /// Corresponds to TypeScript: POST to /youtubei/v1/att/get in getDescrambledChallenge method
    async fn get_challenge(
        &self,
        context: &crate::types::InnertubeContext,
    ) -> crate::Result<crate::types::ChallengeData> {
        use serde_json::json;

        tracing::debug!("Getting challenge from Innertube /att/get endpoint");

        let request_body = json!({
            "context": context,
            "engagementType": "ENGAGEMENT_TYPE_UNBOUND"
        });

        let response = self
            .client
            .post(format!("{}/att/get?prettyPrint=false", self.base_url))
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send request to Innertube att/get: {}", e);
                crate::Error::network(format!("Network request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!("Innertube att/get returned error status: {}", status);
            return Err(crate::Error::network(format!(
                "API request failed with status: {}",
                status
            )));
        }

        let json_response: serde_json::Value = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse Innertube att/get response: {}", e);
            crate::Error::network(format!("Failed to parse JSON response: {}", e))
        })?;

        // Extract bgChallenge from response
        let bg_challenge = json_response.get("bgChallenge").ok_or_else(|| {
            tracing::error!("bgChallenge not found in Innertube att/get response");
            crate::Error::challenge("innertube", "bgChallenge not found in API response")
        })?;

        // Parse the challenge data
        let interpreter_url_value = bg_challenge
            .get("interpreterUrl")
            .and_then(|url| url.get("privateDoNotAccessOrElseTrustedResourceUrlWrappedValue"))
            .and_then(|val| val.as_str())
            .ok_or_else(|| {
                crate::Error::challenge("innertube", "interpreterUrl not found in bgChallenge")
            })?;

        let interpreter_hash = bg_challenge
            .get("interpreterHash")
            .and_then(|hash| hash.as_str())
            .ok_or_else(|| {
                crate::Error::challenge("innertube", "interpreterHash not found in bgChallenge")
            })?;

        let program = bg_challenge
            .get("program")
            .and_then(|prog| prog.as_str())
            .ok_or_else(|| {
                crate::Error::challenge("innertube", "program not found in bgChallenge")
            })?;

        let global_name = bg_challenge
            .get("globalName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                crate::Error::challenge("innertube", "globalName not found in bgChallenge")
            })?;

        let client_experiments_state_blob = bg_challenge
            .get("clientExperimentsStateBlob")
            .and_then(|blob| blob.as_str())
            .map(|s| s.to_string());

        let challenge_data = crate::types::ChallengeData {
            interpreter_url: crate::types::TrustedResourceUrl::new(interpreter_url_value),
            interpreter_hash: interpreter_hash.to_string(),
            program: program.to_string(),
            global_name: global_name.to_string(),
            client_experiments_state_blob,
        };

        tracing::debug!("Successfully retrieved challenge data from Innertube");
        Ok(challenge_data)
    }
}

impl InnertubeClient {
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
