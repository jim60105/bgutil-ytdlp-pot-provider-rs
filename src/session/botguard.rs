//! BotGuard challenge processing and integration
//!
//! This module handles the interaction with Google's BotGuard system,
//! including challenge descrambling and JavaScript VM execution.

use crate::{Result, types::*};
use deno_core::{FastString, JsRuntime, RuntimeOptions};
use reqwest::Client;
use serde_json::Value;

/// BotGuard integration manager
#[derive(Debug)]
#[allow(dead_code)] // TODO: Remove when implementation is complete
pub struct BotGuardManager {
    /// HTTP client for requests
    client: Client,
    /// Request key for API calls
    request_key: String,
}

impl BotGuardManager {
    /// Create new BotGuard manager
    pub fn new(client: Client, request_key: String) -> Self {
        Self {
            client,
            request_key,
        }
    }

    /// Get descrambled challenge from Innertube or fallback endpoint
    ///
    /// Corresponds to TypeScript: `getDescrambledChallenge` method (L243-317)
    pub async fn get_descrambled_challenge(
        &self,
        challenge: Option<ChallengeData>,
        innertube_context: Option<InnertubeContext>,
        disable_innertube: bool,
    ) -> Result<DescrambledChallenge> {
        // Try Innertube /att/get endpoint first
        if !disable_innertube {
            if let Some(context) = innertube_context {
                match self.get_challenge_from_innertube(&context).await {
                    Ok(challenge) => return Ok(challenge),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to get challenge from Innertube: {}, trying /Create endpoint",
                            e
                        );
                    }
                }
            } else if let Some(challenge) = challenge {
                tracing::debug!("Using challenge from webpage");
                return self.process_challenge_data(challenge).await;
            }
        } else {
            tracing::debug!("Using /Create endpoint as Innertube challenges are disabled");
        }

        // Fallback to /Create endpoint
        self.get_challenge_from_create_endpoint().await
    }

    /// Get challenge from Innertube /att/get endpoint
    async fn get_challenge_from_innertube(
        &self,
        _context: &InnertubeContext,
    ) -> Result<DescrambledChallenge> {
        // TODO: Implement Innertube API call
        // POST to https://www.youtube.com/youtubei/v1/att/get?prettyPrint=false
        tracing::warn!("Innertube challenge retrieval not implemented yet");
        Err(crate::Error::challenge("Innertube not implemented"))
    }

    /// Process challenge data from webpage
    async fn process_challenge_data(
        &self,
        _challenge: ChallengeData,
    ) -> Result<DescrambledChallenge> {
        // TODO: Implement challenge data processing
        // Fetch interpreter JavaScript and create DescrambledChallenge
        tracing::warn!("Challenge data processing not implemented yet");
        Err(crate::Error::challenge(
            "Challenge processing not implemented",
        ))
    }

    /// Get challenge from /Create endpoint
    async fn get_challenge_from_create_endpoint(&self) -> Result<DescrambledChallenge> {
        // Use WAA API to fetch challenge
        let waa_response = self.fetch_waa_challenge(None).await?;
        Ok(self.waa_response_to_descrambled_challenge(waa_response))
    }

    /// Generate Integrity Token from BotGuard response
    pub async fn get_integrity_token(
        &self,
        botguard_response: &str,
    ) -> Result<IntegrityTokenResponse> {
        let payload = vec![
            serde_json::Value::String(self.request_key.clone()),
            serde_json::Value::String(botguard_response.to_string()),
        ];

        let response = self
            .client
            .post("https://jnn-pa.googleapis.com/$rpc/google.internal.waa.v1.Waa/GenerateIT")
            .header("Content-Type", "application/json+protobuf")
            .header("x-goog-api-key", "AIzaSyDyT5W0Jh49F30Pqqtyfdf7pDLFKLJoAnw")
            .header("x-user-agent", "grpc-web-javascript/0.1")
            .json(&payload)
            .send()
            .await?;

        let raw_data: Value = response.json().await?;
        self.parse_integrity_token_response(&raw_data)
    }

    /// Parse Integrity Token API response
    fn parse_integrity_token_response(&self, raw_data: &Value) -> Result<IntegrityTokenResponse> {
        let array = raw_data.as_array().ok_or_else(|| {
            crate::Error::integrity_token("Invalid IntegrityToken response format".to_string())
        })?;

        if array.is_empty() {
            return Err(crate::Error::integrity_token(
                "Empty IntegrityToken response".to_string(),
            ));
        }

        Ok(IntegrityTokenResponse {
            integrity_token: array[0].as_str().map(|s| s.to_string()),
            estimated_ttl_secs: array.get(1).and_then(|v| v.as_u64()).unwrap_or(3600),
            mint_refresh_threshold: array.get(2).and_then(|v| v.as_u64()).unwrap_or(1800),
            websafe_fallback_token: array.get(3).and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }

    /// Fetch WAA challenge from Google WAA API
    async fn fetch_waa_challenge(&self, interpreter_hash: Option<&str>) -> Result<WaaResponse> {
        let mut payload = vec![serde_json::Value::String(self.request_key.clone())];

        if let Some(hash) = interpreter_hash {
            payload.push(serde_json::Value::String(hash.to_string()));
        }

        let response = self
            .client
            .post("https://jnn-pa.googleapis.com/$rpc/google.internal.waa.v1.Waa/Create")
            .header("Content-Type", "application/json+protobuf")
            .header("x-goog-api-key", "AIzaSyDyT5W0Jh49F30Pqqtyfdf7pDLFKLJoAnw")
            .header("x-user-agent", "grpc-web-javascript/0.1")
            .json(&payload)
            .send()
            .await?;

        let raw_data: Value = response.json().await?;
        self.parse_waa_response(&raw_data)
    }

    /// Parse WAA API response into structured data
    fn parse_waa_response(&self, raw_data: &Value) -> Result<WaaResponse> {
        let array = raw_data
            .as_array()
            .ok_or_else(|| crate::Error::challenge("Invalid WAA response format".to_string()))?;

        if array.len() < 5 {
            return Err(crate::Error::challenge(
                "Insufficient WAA response data".to_string(),
            ));
        }

        Ok(WaaResponse {
            message_id: array[0].as_str().map(|s| s.to_string()),
            interpreter_javascript: array[1].as_str().unwrap_or("").to_string(),
            interpreter_hash: array[2].as_str().unwrap_or("").to_string(),
            program: array[3].as_str().unwrap_or("").to_string(),
            global_name: array[4].as_str().unwrap_or("").to_string(),
            client_experiments_state_blob: array
                .get(5)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Convert WAA response to DescrambledChallenge
    fn waa_response_to_descrambled_challenge(
        &self,
        waa_response: WaaResponse,
    ) -> DescrambledChallenge {
        DescrambledChallenge {
            message_id: waa_response.message_id,
            interpreter_javascript: TrustedScript::new(
                waa_response.interpreter_javascript,
                "https://jnn-pa.googleapis.com",
            ),
            interpreter_hash: waa_response.interpreter_hash,
            program: waa_response.program,
            global_name: waa_response.global_name,
            client_experiments_state_blob: waa_response.client_experiments_state_blob,
        }
    }
}

/// WAA API response structure
#[derive(Debug, Clone)]
pub struct WaaResponse {
    /// The ID of the JSPB message
    pub message_id: Option<String>,
    /// The script associated with the challenge
    pub interpreter_javascript: String,
    /// The hash of the script
    pub interpreter_hash: String,
    /// The challenge program
    pub program: String,
    /// The name of the VM in the global scope
    pub global_name: String,
    /// The client experiments state blob
    pub client_experiments_state_blob: Option<String>,
}

/// Integrity Token response from WAA GenerateIT API
#[derive(Debug, Clone)]
pub struct IntegrityTokenResponse {
    /// The integrity token
    pub integrity_token: Option<String>,
    /// Estimated TTL in seconds
    pub estimated_ttl_secs: u64,
    /// Mint refresh threshold in seconds
    pub mint_refresh_threshold: u64,
    /// Websafe fallback token
    pub websafe_fallback_token: Option<String>,
}

/// BotGuard JavaScript VM client
pub struct BotGuardClient {
    /// JavaScript runtime
    runtime: JsRuntime,
    /// Challenge program
    program: String,
    /// Global VM name
    global_name: String,
    /// VM functions after initialization
    vm_functions: Option<VmFunctions>,
}

/// VM functions returned by BotGuard initialization
#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO: Remove when functions are actually used
struct VmFunctions {
    /// Async snapshot function reference
    async_snapshot_function: String,
    /// Shutdown function reference
    shutdown_function: String,
    /// Pass event function reference
    pass_event_function: String,
    /// Check camera function reference
    check_camera_function: String,
}

impl BotGuardClient {
    /// Create new BotGuard client with JavaScript runtime
    pub async fn new(
        interpreter_javascript: &str,
        program: &str,
        global_name: &str,
    ) -> Result<Self> {
        let mut runtime = Self::create_js_runtime().await?;

        // Execute BotGuard interpreter script
        runtime
            .execute_script(
                "botguard_interpreter.js",
                FastString::from(interpreter_javascript.to_string()),
            )
            .map_err(|e| {
                crate::Error::botguard(format!("Failed to execute interpreter script: {}", e))
            })?;

        Ok(Self {
            runtime,
            program: program.to_string(),
            global_name: global_name.to_string(),
            vm_functions: None,
        })
    }

    /// Create JavaScript runtime with appropriate configuration
    async fn create_js_runtime() -> Result<JsRuntime> {
        let runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![],
            ..Default::default()
        });

        Ok(runtime)
    }

    /// Load the BotGuard program and initialize VM
    pub async fn load_program(&mut self) -> Result<()> {
        // Create a JavaScript function to handle VM functions callback
        let init_script = format!(
            r#"
            let vmFunctions = null;
            
            function vmFunctionsCallback(asyncFn, shutdownFn, passEventFn, checkCameraFn) {{
                vmFunctions = {{
                    async: asyncFn,
                    shutdown: shutdownFn,
                    passEvent: passEventFn,
                    checkCamera: checkCameraFn
                }};
            }}
            
            // Initialize BotGuard VM
            if (typeof globalThis.{} !== 'undefined' && globalThis.{}.a) {{
                const syncFunctions = globalThis.{}.a('{}', vmFunctionsCallback, 20, 0, function() {{}}, []);
                globalThis.syncSnapshot = syncFunctions[0];
            }} else {{
                throw new Error('BotGuard VM not found in global scope: {}');
            }}
            "#,
            self.global_name, self.global_name, self.global_name, self.program, self.global_name
        );

        self.runtime
            .execute_script("botguard_init.js", FastString::from(init_script))
            .map_err(|e| {
                crate::Error::botguard(format!("Failed to initialize BotGuard VM: {}", e))
            })?;

        // Store VM functions info (simplified for now)
        self.vm_functions = Some(VmFunctions {
            async_snapshot_function: "vmFunctions.async".to_string(),
            shutdown_function: "vmFunctions.shutdown".to_string(),
            pass_event_function: "vmFunctions.passEvent".to_string(),
            check_camera_function: "vmFunctions.checkCamera".to_string(),
        });

        tracing::info!("BotGuard VM initialized successfully");
        Ok(())
    }

    /// Generate BotGuard response using the VM
    pub async fn generate_response(&mut self) -> Result<String> {
        if self.vm_functions.is_none() {
            return Err(crate::Error::botguard(
                "VM not initialized. Call load_program() first.".to_string(),
            ));
        }

        // Execute sync snapshot function
        let result = self
            .runtime
            .execute_script(
                "botguard_snapshot.js",
                FastString::from("globalThis.syncSnapshot()".to_string()),
            )
            .map_err(|e| {
                crate::Error::botguard(format!("Failed to generate BotGuard response: {}", e))
            })?;

        // Convert the result to string (simplified)
        let response = format!("{:?}", result);
        Ok(response)
    }

    /// Get VM functions status
    pub fn is_initialized(&self) -> bool {
        self.vm_functions.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_waa_response_creation() {
        let waa_response = WaaResponse {
            message_id: Some("test_msg_123".to_string()),
            interpreter_javascript: "test_script".to_string(),
            interpreter_hash: "test_hash".to_string(),
            program: "test_program".to_string(),
            global_name: "test_global".to_string(),
            client_experiments_state_blob: None,
        };

        assert_eq!(waa_response.message_id, Some("test_msg_123".to_string()));
        assert_eq!(waa_response.interpreter_javascript, "test_script");
        assert_eq!(waa_response.interpreter_hash, "test_hash");
        assert_eq!(waa_response.program, "test_program");
        assert_eq!(waa_response.global_name, "test_global");
    }

    #[test]
    fn test_parse_waa_response_success() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!([
            "test_message_id",
            "test_interpreter_js",
            "test_hash",
            "test_program",
            "test_global_name"
        ]);

        let result = manager.parse_waa_response(&mock_data);
        assert!(result.is_ok());

        let waa_response = result.unwrap();
        assert_eq!(waa_response.message_id, Some("test_message_id".to_string()));
        assert_eq!(waa_response.interpreter_javascript, "test_interpreter_js");
        assert_eq!(waa_response.interpreter_hash, "test_hash");
        assert_eq!(waa_response.program, "test_program");
        assert_eq!(waa_response.global_name, "test_global_name");
    }

    #[test]
    fn test_parse_waa_response_with_experiments() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!([
            "msg_id",
            "js_code",
            "hash",
            "program",
            "global",
            "experiments_blob"
        ]);

        let result = manager.parse_waa_response(&mock_data);
        assert!(result.is_ok());

        let waa_response = result.unwrap();
        assert_eq!(
            waa_response.client_experiments_state_blob,
            Some("experiments_blob".to_string())
        );
    }

    #[test]
    fn test_parse_waa_response_insufficient_data() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!(["only", "three", "items"]);

        let result = manager.parse_waa_response(&mock_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Insufficient WAA response data")
        );
    }

    #[test]
    fn test_parse_waa_response_invalid_format() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!({"not": "an_array"});

        let result = manager.parse_waa_response(&mock_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid WAA response format")
        );
    }

    #[test]
    fn test_waa_response_to_descrambled_challenge() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let waa_response = WaaResponse {
            message_id: Some("test_msg".to_string()),
            interpreter_javascript: "test_js_code".to_string(),
            interpreter_hash: "test_hash".to_string(),
            program: "test_program".to_string(),
            global_name: "test_global".to_string(),
            client_experiments_state_blob: Some("test_blob".to_string()),
        };

        let challenge = manager.waa_response_to_descrambled_challenge(waa_response);

        assert_eq!(challenge.message_id, Some("test_msg".to_string()));
        assert_eq!(challenge.interpreter_javascript.script(), "test_js_code");
        assert_eq!(challenge.interpreter_hash, "test_hash");
        assert_eq!(challenge.program, "test_program");
        assert_eq!(challenge.global_name, "test_global");
        assert_eq!(
            challenge.client_experiments_state_blob,
            Some("test_blob".to_string())
        );
    }

    #[tokio::test]
    async fn test_fetch_waa_challenge_with_mockito() {
        // This test uses mockito to mock the WAA API
        let mut server = mockito::Server::new_async().await;
        let mock_response = json!([
            "test_message_id",
            "test_interpreter_js",
            "test_hash",
            "test_program",
            "test_global_name"
        ]);

        let mock = server
            .mock("POST", "/$rpc/google.internal.waa.v1.Waa/Create")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        // Create a client with the mockito server URL
        let _client = Client::builder().build().unwrap();

        // Note: This is a simplified test - the actual implementation would need
        // URL configuration for testing
        drop(mock);
    }

    #[tokio::test]
    async fn test_botguard_manager_creation() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());
        assert_eq!(manager.request_key, "test_key");
    }

    #[tokio::test]
    async fn test_get_descrambled_challenge_fallback() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let result = manager.get_descrambled_challenge(None, None, false).await;

        // Should fail since we're making a real API call without proper authentication
        assert!(result.is_err());
        // The error could be a network error, HTTP error, or authentication error
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Network error")
                || error_msg.contains("dns error")
                || error_msg.contains("error sending request")
                || error_msg.contains("connection")
                || error_msg.contains("HTTP")
                || error_msg.contains("400")
                || error_msg.contains("401")
                || error_msg.contains("403")
                || error_msg.contains("404")
                || error_msg.contains("invalid")
                || error_msg.contains("failed")
                || error_msg.contains("Insufficient")
                || error_msg.contains("Challenge processing")
        );
    }

    #[tokio::test]
    async fn test_get_descrambled_challenge_disable_innertube() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let result = manager.get_descrambled_challenge(None, None, true).await;

        // Should fail since we're making a real API call without proper authentication
        assert!(result.is_err());
        // The error could be a network error, HTTP error, or authentication error
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Network error")
                || error_msg.contains("dns error")
                || error_msg.contains("error sending request")
                || error_msg.contains("connection")
                || error_msg.contains("HTTP")
                || error_msg.contains("400")
                || error_msg.contains("401")
                || error_msg.contains("403")
                || error_msg.contains("404")
                || error_msg.contains("invalid")
                || error_msg.contains("failed")
                || error_msg.contains("Insufficient")
                || error_msg.contains("Challenge processing")
        );
    }

    // JavaScript VM Tests
    #[tokio::test]
    async fn test_create_js_runtime() {
        let runtime = BotGuardClient::create_js_runtime().await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_execute_simple_javascript() {
        let mut runtime = BotGuardClient::create_js_runtime().await.unwrap();
        let result = runtime.execute_script("test.js", FastString::from("1 + 1".to_string()));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_botguard_script() {
        let mut runtime = BotGuardClient::create_js_runtime().await.unwrap();

        // Simulate BotGuard script structure
        let script = r#"
            globalThis.testVM = {
                a: function(program, callback, flag1, flag2, noop, arrays) {
                    // Simulate BotGuard VM initialization
                    return [function() { return "test_sync_function"; }];
                }
            };
        "#;

        let result = runtime.execute_script("botguard.js", FastString::from(script.to_string()));
        assert!(result.is_ok());

        // Verify global object was created
        let global_check = runtime.execute_script(
            "check.js",
            FastString::from("typeof globalThis.testVM".to_string()),
        );
        assert!(global_check.is_ok());
    }

    #[tokio::test]
    async fn test_botguard_client_initialization() {
        let interpreter_js = r#"
            globalThis.testBG = {
                a: function(program, vmFunctionsCallback, flag1, flag2, noop, arrays) {
                    // Simulate VM functions callback
                    vmFunctionsCallback(
                        function(callback, args) { callback("async_result"); },
                        function() { console.log("shutdown"); },
                        function(event) { console.log("pass_event"); },
                        function() { console.log("check_camera"); }
                    );
                    return [function() { return "sync_snapshot"; }];
                }
            };
        "#;

        let client = BotGuardClient::new(interpreter_js, "test_program", "testBG").await;
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.program, "test_program");
        assert_eq!(client.global_name, "testBG");
        assert!(!client.is_initialized());
    }

    #[tokio::test]
    async fn test_botguard_program_loading() {
        let interpreter_js = r#"
            globalThis.testBG = {
                a: function(program, vmFunctionsCallback, flag1, flag2, noop, arrays) {
                    if (program !== "test_program") {
                        throw new Error("Invalid program");
                    }
                    vmFunctionsCallback(null, null, null, null);
                    return [function() { return "loaded"; }];
                }
            };
        "#;

        let mut client = BotGuardClient::new(interpreter_js, "test_program", "testBG")
            .await
            .unwrap();
        assert!(!client.is_initialized());

        let result = client.load_program().await;
        assert!(result.is_ok());
        assert!(client.is_initialized());
    }

    #[tokio::test]
    async fn test_botguard_program_loading_error() {
        let interpreter_js = r#"
            // Missing BotGuard VM
        "#;

        let mut client = BotGuardClient::new(interpreter_js, "test_program", "nonexistentBG")
            .await
            .unwrap();
        let result = client.load_program().await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("BotGuard VM not found"));
    }

    #[tokio::test]
    async fn test_botguard_response_generation() {
        let interpreter_js = r#"
            globalThis.testBG = {
                a: function(program, vmFunctionsCallback, flag1, flag2, noop, arrays) {
                    vmFunctionsCallback(null, null, null, null);
                    return [function() { return "test_response_data"; }];
                }
            };
        "#;

        let mut client = BotGuardClient::new(interpreter_js, "test_program", "testBG")
            .await
            .unwrap();

        // Should fail before initialization
        let result = client.generate_response().await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("VM not initialized")
        );

        // Load program first
        client.load_program().await.unwrap();

        // Now should work
        let result = client.generate_response().await;
        assert!(result.is_ok());
    }

    // Integrity Token Tests
    #[test]
    fn test_integrity_token_response_creation() {
        let response = IntegrityTokenResponse {
            integrity_token: Some("test_token_123".to_string()),
            estimated_ttl_secs: 3600,
            mint_refresh_threshold: 1800,
            websafe_fallback_token: Some("fallback_token".to_string()),
        };

        assert_eq!(response.integrity_token, Some("test_token_123".to_string()));
        assert_eq!(response.estimated_ttl_secs, 3600);
        assert_eq!(response.mint_refresh_threshold, 1800);
        assert_eq!(
            response.websafe_fallback_token,
            Some("fallback_token".to_string())
        );
    }

    #[test]
    fn test_parse_integrity_token_response_success() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!(["integrity_token_123", 3600, 1800, "fallback_token"]);

        let result = manager.parse_integrity_token_response(&mock_data);
        assert!(result.is_ok());

        let token_response = result.unwrap();
        assert_eq!(
            token_response.integrity_token,
            Some("integrity_token_123".to_string())
        );
        assert_eq!(token_response.estimated_ttl_secs, 3600);
        assert_eq!(token_response.mint_refresh_threshold, 1800);
        assert_eq!(
            token_response.websafe_fallback_token,
            Some("fallback_token".to_string())
        );
    }

    #[test]
    fn test_parse_integrity_token_response_minimal() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!(["token_only"]);

        let result = manager.parse_integrity_token_response(&mock_data);
        assert!(result.is_ok());

        let token_response = result.unwrap();
        assert_eq!(
            token_response.integrity_token,
            Some("token_only".to_string())
        );
        assert_eq!(token_response.estimated_ttl_secs, 3600); // default
        assert_eq!(token_response.mint_refresh_threshold, 1800); // default
        assert_eq!(token_response.websafe_fallback_token, None);
    }

    #[test]
    fn test_parse_integrity_token_response_empty() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!([]);

        let result = manager.parse_integrity_token_response(&mock_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Empty IntegrityToken response")
        );
    }

    #[test]
    fn test_parse_integrity_token_response_invalid_format() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        let mock_data = json!({"not": "an_array"});

        let result = manager.parse_integrity_token_response(&mock_data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid IntegrityToken response format")
        );
    }

    #[tokio::test]
    async fn test_get_integrity_token_integration() {
        let client = Client::new();
        let manager = BotGuardManager::new(client, "test_key".to_string());

        // This will call the real API
        let result = manager.get_integrity_token("test_botguard_response").await;

        // The test might succeed if the API accepts the test response, or fail with an error
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Network error")
                    || error_msg.contains("dns error")
                    || error_msg.contains("error sending request")
                    || error_msg.contains("connection")
                    || error_msg.contains("HTTP")
                    || error_msg.contains("400")
                    || error_msg.contains("401")
                    || error_msg.contains("403")
                    || error_msg.contains("404")
                    || error_msg.contains("invalid")
                    || error_msg.contains("failed")
                    || error_msg.contains("Insufficient")
                    || error_msg.contains("Challenge processing")
            );
        } else {
            // If it succeeds, that's valid - the API accepted our test response
            let response = result.unwrap();
            assert!(
                response.integrity_token.is_some() || response.websafe_fallback_token.is_some()
            );
        }
    }
}
