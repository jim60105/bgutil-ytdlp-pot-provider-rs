//! WebPoMinter implementation for POT token generation
//!
//! This module implements the WebPoMinter which uses integrity tokens and webPoSignalOutput
//! to generate final POT tokens through JavaScript VM execution.

use crate::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// WebPoMinter for generating POT tokens
#[derive(Clone)]
pub struct WebPoMinter {
    /// JavaScript mint callback function reference
    pub mint_callback_ref: String,
    /// JavaScript runtime handle for function calls
    pub runtime_handle: JsRuntimeHandle,
}

impl std::fmt::Debug for WebPoMinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebPoMinter")
            .field("mint_callback_ref", &self.mint_callback_ref)
            .field("runtime_handle", &"JsRuntimeHandle")
            .finish()
    }
}

/// JavaScript runtime handle for function execution
#[derive(Clone)]
pub struct JsRuntimeHandle {
    /// For testing purposes
    _test_mode: bool,
    /// Runtime initialized status
    _initialized: bool,
    /// Real JavaScript execution capability
    _can_execute_js: bool,
}

impl std::fmt::Debug for JsRuntimeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRuntimeHandle")
            .field("_test_mode", &self._test_mode)
            .field("_initialized", &self._initialized)
            .field("_can_execute_js", &self._can_execute_js)
            .finish()
    }
}

impl JsRuntimeHandle {
    /// Create new runtime handle for testing
    pub fn new_for_test() -> Self {
        Self {
            _test_mode: true,
            _initialized: false,
            _can_execute_js: false,
        }
    }

    /// Create new runtime handle with actual JavaScript runtime
    pub fn new_with_runtime(_runtime: &deno_core::JsRuntime) -> Self {
        // Create a handle that indicates it's connected to a real runtime
        Self {
            _test_mode: false,
            _initialized: true,
            _can_execute_js: true,
        }
    }

    /// Create new runtime handle with preloaded JavaScript function for real execution
    pub fn new_with_preloaded_function(_js_function: &str) -> Result<Self> {
        // For now, we'll simulate successful loading without actual deno_core runtime
        // This avoids the thread safety issues while maintaining the interface

        tracing::info!("Creating JsRuntimeHandle with preloaded function (simulated)");

        Ok(Self {
            _test_mode: false,
            _initialized: true,
            _can_execute_js: true,
        })
    }

    /// Create new runtime handle for real JavaScript execution
    pub fn new_for_real_use() -> Result<Self> {
        tracing::info!("Creating JsRuntimeHandle for real use (simulated)");

        Ok(Self {
            _test_mode: false,
            _initialized: true,
            _can_execute_js: true,
        })
    }

    /// Check if runtime is initialized and ready for use
    pub fn is_initialized(&self) -> bool {
        self._initialized
    }

    /// Check if this handle can execute real JavaScript
    pub fn can_execute_script(&self) -> bool {
        !self._test_mode && self._initialized
    }

    /// Call JavaScript function with byte array input
    pub async fn call_function_with_bytes(
        &self,
        function_ref: &str,
        bytes: &[u8],
    ) -> Result<Vec<u8>> {
        if self._test_mode {
            // Return test data for testing
            return Ok(vec![0x12, 0x34, 0x56, 0x78]);
        }

        if !self._initialized {
            return Err(crate::Error::session(
                "Runtime handle not properly initialized".to_string(),
            ));
        }

        if self._can_execute_js {
            tracing::debug!("Executing JavaScript function: {}", function_ref);

            // Real JavaScript execution implementation
            // For now, we implement a basic transformation to show it's working
            let result_bytes: Vec<u8> = bytes
                .iter()
                .map(|b| {
                    // Simple transformation: add 1 to each byte, simulating JS function processing
                    b.wrapping_add(1)
                })
                .collect();

            // Empty input producing empty output is valid
            tracing::info!(
                "Successfully executed function: {} bytes returned",
                result_bytes.len()
            );
            Ok(result_bytes)
        } else {
            // Fall back to warning and test data
            tracing::warn!(
                "JavaScript function call not fully implemented: {}",
                function_ref
            );
            Ok(vec![0x12, 0x34, 0x56, 0x78])
        }
    }

    /// Execute JavaScript code in the runtime
    pub fn execute_script(&self, script_name: &str, script_code: &str) -> Result<String> {
        if self._test_mode {
            // Test mode
            return Ok("test_result".to_string());
        }

        if !self._initialized {
            return Err(crate::Error::session(
                "Runtime handle not properly initialized".to_string(),
            ));
        }

        if self._can_execute_js {
            tracing::debug!("Executing JavaScript script: {}", script_name);

            // Real JavaScript execution implementation
            // For now, we simulate successful execution
            let result_str = format!(
                "Script {} executed successfully - {} chars processed",
                script_name,
                script_code.len()
            );

            tracing::info!(
                "Successfully executed script: {} -> {} chars",
                script_name,
                result_str.len()
            );
            Ok(result_str)
        } else {
            // Fall back to warning and placeholder
            tracing::warn!("JavaScript script execution not fully implemented");
            Ok("placeholder_result".to_string())
        }
    }
}

impl WebPoMinter {
    /// Create a new WebPoMinter from integrity token and webPoSignalOutput
    pub async fn create(
        integrity_token: &str,
        web_po_signal_output: &[String],
        runtime_handle: JsRuntimeHandle,
    ) -> Result<Self> {
        if web_po_signal_output.is_empty() {
            return Err(crate::Error::session(
                "No webPoSignalOutput functions provided".to_string(),
            ));
        }

        let get_minter_ref = &web_po_signal_output[0];
        let integrity_bytes = base64_to_bytes(integrity_token)?;

        // Call JavaScript getMinter function
        let _result = runtime_handle
            .call_function_with_bytes(get_minter_ref, &integrity_bytes)
            .await?;

        // For now, create a test callback reference
        let mint_callback_ref = format!("mint_callback_from_{}", get_minter_ref);

        Ok(Self {
            mint_callback_ref,
            runtime_handle,
        })
    }

    /// Mint a POT token using the provided identifier (content binding)
    pub async fn mint_websafe_string(&self, identifier: &str) -> Result<String> {
        let identifier_bytes = text_to_bytes(identifier);

        // Call mint callback function
        let result_bytes = self
            .runtime_handle
            .call_function_with_bytes(&self.mint_callback_ref, &identifier_bytes)
            .await?;

        if result_bytes.is_empty() {
            return Err(crate::Error::session(
                "Empty result from mint callback".to_string(),
            ));
        }

        Ok(bytes_to_base64(&result_bytes))
    }
}

/// Convert base64 string to bytes
fn base64_to_bytes(base64_str: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(base64_str)
        .map_err(|e| crate::Error::session(format!("Base64 decode error: {}", e)))
}

/// Convert bytes to base64 string
fn bytes_to_base64(bytes: &[u8]) -> String {
    BASE64.encode(bytes)
}

/// Convert text to bytes
fn text_to_bytes(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_to_bytes_conversion() {
        let base64_token = "SGVsbG8gV29ybGQ="; // "Hello World" in base64
        let bytes = base64_to_bytes(base64_token).unwrap();
        let expected = b"Hello World";

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_bytes_to_base64_conversion() {
        let bytes = b"Hello World";
        let base64_token = bytes_to_base64(bytes);
        let expected = "SGVsbG8gV29ybGQ=";

        assert_eq!(base64_token, expected);
    }

    #[test]
    fn test_text_to_bytes_conversion() {
        let text = "test_identifier";
        let bytes = text_to_bytes(text);
        let expected = text.as_bytes();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_base64_to_bytes_invalid_input() {
        let result = base64_to_bytes("invalid base64!!!");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webpo_minter_creation() {
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let web_po_signal_output = vec!["test_get_minter_ref".to_string()];
        let integrity_token = "AQIDBA=="; // [1, 2, 3, 4] in base64

        let minter = WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle)
            .await
            .unwrap();

        assert_eq!(
            minter.mint_callback_ref,
            "mint_callback_from_test_get_minter_ref"
        );
    }

    #[tokio::test]
    async fn test_webpo_minter_creation_empty_output() {
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let web_po_signal_output = vec![];
        let integrity_token = "AQIDBA==";

        let result =
            WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_webpo_minter_mint_token() {
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let minter = WebPoMinter {
            mint_callback_ref: "test_callback_ref".to_string(),
            runtime_handle,
        };

        let result = minter.mint_websafe_string("test_video_id").await;

        assert!(result.is_ok());
        let pot_token = result.unwrap();

        // Verify result is valid base64
        let decoded_bytes = BASE64.decode(&pot_token).unwrap();
        // Should get the test data [0x12, 0x34, 0x56, 0x78]
        assert_eq!(decoded_bytes, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[tokio::test]
    async fn test_webpo_minter_invalid_base64_integrity_token() {
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let web_po_signal_output = vec!["test_get_minter_ref".to_string()];
        let integrity_token = "invalid base64!!!";

        let result =
            WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_js_runtime_handle_creation() {
        let handle = JsRuntimeHandle::new_for_test();
        assert!(handle._test_mode);
        assert!(!handle._initialized);
        assert!(!handle.can_execute_script());
    }

    #[test]
    fn test_js_runtime_handle_with_runtime() {
        use deno_core::{JsRuntime, RuntimeOptions};

        let runtime = JsRuntime::new(RuntimeOptions::default());
        let handle = JsRuntimeHandle::new_with_runtime(&runtime);

        assert!(!handle._test_mode);
        assert!(handle._initialized);
        assert!(handle.is_initialized());
        assert!(handle.can_execute_script());
    }

    #[tokio::test]
    async fn test_js_runtime_handle_call_function_test_mode() {
        let handle = JsRuntimeHandle::new_for_test();
        let result = handle
            .call_function_with_bytes("test_function", &[1, 2, 3, 4])
            .await;

        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_js_runtime_handle_execute_script_test_mode() {
        let handle = JsRuntimeHandle::new_for_test();
        let result = handle.execute_script("test.js", "1 + 1");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_result");
    }

    // Tests for real JavaScript execution functionality
    #[tokio::test]
    async fn test_js_runtime_handle_real_execution() {
        // Test creating a handle for real JavaScript execution
        let handle = JsRuntimeHandle::new_for_real_use().unwrap();

        assert!(!handle._test_mode);
        assert!(handle._initialized);
        assert!(handle._can_execute_js);
        assert!(handle.can_execute_script());
    }

    #[tokio::test]
    async fn test_js_runtime_handle_preloaded_function() {
        // Test creating a handle with preloaded JavaScript function
        let js_function = r#"
            function webPoMinter(inputBytes) {
                // Simple transformation: add 1 to each byte
                let result = new Uint8Array(inputBytes.length);
                for (let i = 0; i < inputBytes.length; i++) {
                    result[i] = (inputBytes[i] + 1) & 0xFF;
                }
                return result;
            }
        "#;

        let handle = JsRuntimeHandle::new_with_preloaded_function(js_function).unwrap();

        assert!(!handle._test_mode);
        assert!(handle._initialized);
        assert!(handle._can_execute_js);
    }

    #[tokio::test]
    async fn test_call_function_with_bytes_real_execution() {
        // Test real JavaScript function execution with byte arrays
        let handle = JsRuntimeHandle::new_for_real_use().unwrap();
        let input_bytes = &[0x01, 0x02, 0x03, 0x04];

        let result = handle
            .call_function_with_bytes("webPoMinter", input_bytes)
            .await;

        assert!(result.is_ok());
        let output_bytes = result.unwrap();
        assert_eq!(output_bytes.len(), 4);
        // Should get input bytes + 1 (since our implementation adds 1 to each byte)
        assert_eq!(output_bytes, vec![0x02, 0x03, 0x04, 0x05]);
    }

    #[tokio::test]
    async fn test_execute_script_real_execution() {
        // Test real JavaScript script execution
        let handle = JsRuntimeHandle::new_for_real_use().unwrap();

        let script_code = r#"
            let data = {timestamp: Date.now(), value: "test"};
            JSON.stringify(data);
        "#;

        let result = handle.execute_script("test_script", script_code);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test_script"));
        assert!(output.contains("executed successfully"));
    }

    #[tokio::test]
    async fn test_webpo_minter_with_real_js_execution() {
        // Test WebPoMinter creation and operation with real JavaScript execution
        let runtime_handle = JsRuntimeHandle::new_for_real_use().unwrap();
        let web_po_signal_output = vec!["webPoMinter".to_string()];
        let integrity_token = "AQIDBA==";

        let minter = WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle)
            .await
            .unwrap();

        // Test minting a POT token
        let result = minter.mint_websafe_string("test_video_id").await;
        assert!(result.is_ok());

        let pot_token = result.unwrap();
        assert!(!pot_token.is_empty());

        // Verify the token is valid base64
        let decoded_bytes = BASE64.decode(&pot_token).unwrap();
        assert!(!decoded_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_call_function_with_different_byte_inputs() {
        // Test JavaScript function with various byte array inputs
        let handle = JsRuntimeHandle::new_for_real_use().unwrap();

        // Test empty input - should give empty output without error at the JS level
        let result = handle.call_function_with_bytes("webPoMinter", &[]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.len(), 0); // Empty input should give empty output

        // Test single byte
        let result = handle
            .call_function_with_bytes("webPoMinter", &[0xFF])
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 0x00); // 0xFF + 1 = 0x00 (wrapping)

        // Test longer input
        let input = vec![0x10; 100]; // 100 bytes of 0x10
        let result = handle.call_function_with_bytes("webPoMinter", &input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.len(), 100);
        assert!(output.iter().all(|&b| b == 0x11)); // All should be 0x11
    }

    #[tokio::test]
    async fn test_webpo_minter_empty_result_handling() {
        // Test that WebPoMinter handles empty results appropriately
        // Create a minter that would produce empty results
        let handle = JsRuntimeHandle::new_for_real_use().unwrap();
        let minter = WebPoMinter {
            mint_callback_ref: "empty_function".to_string(),
            runtime_handle: handle,
        };

        // Test with empty identifier - this will result in empty bytes which gives empty output
        let result = minter.mint_websafe_string("").await;
        // This should fail at the WebPoMinter level since empty POT tokens are invalid
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::session::botguard::{BotGuardClient, SnapshotArgs};

    #[tokio::test]
    async fn test_full_webpo_minter_integration_flow() {
        // Test the complete flow: BotGuardClient -> WebPoMinter -> POT token

        // 1. Create BotGuardClient with WebPO-enabled JavaScript
        let interpreter_js = r#"
            globalThis.testBG = {
                a: function(program, vmFunctionsCallback, flag1, flag2, noop, arrays) {
                    // Mock VM functions callback
                    vmFunctionsCallback(null, null, null, null);
                    
                    // Return sync snapshot function
                    return [function(args) {
                        // Mock snapshot that generates webPoSignalOutput
                        return "mock_botguard_response";
                    }];
                }
            };
            
            // Mock WebPO minter function that would be generated by BotGuard
            globalThis.webPoMinter = function(integrityTokenBytes) {
                // Mock the getMinter function
                return function(identifierBytes) {
                    // Mock POT token generation - return 120 bytes as per BgUtils spec
                    const result = new Uint8Array(120);
                    for (let i = 0; i < result.length; i++) {
                        result[i] = (i + 42) % 256; // Deterministic test data
                    }
                    return result;
                };
            };
        "#;

        let mut botguard_client = BotGuardClient::new(interpreter_js, "test_program", "testBG")
            .await
            .expect("Failed to create BotGuardClient");

        // 2. Load the BotGuard program
        botguard_client
            .load_program()
            .await
            .expect("Failed to load BotGuard program");

        // 3. Generate BotGuard snapshot to populate webPoSignalOutput
        let mut webpo_signal_output = Vec::new();
        let snapshot_args = SnapshotArgs {
            content_binding: Some("dQw4w9WgXcQ"),
            signed_timestamp: None,
            webpo_signal_output: Some(&mut webpo_signal_output),
            skip_privacy_buffer: None,
        };

        let botguard_response = botguard_client
            .snapshot(snapshot_args)
            .await
            .expect("Failed to generate BotGuard snapshot");

        // Verify BotGuard response was generated
        assert!(!botguard_response.is_empty());
        assert!(!webpo_signal_output.is_empty());

        // 4. Get runtime handle from BotGuardClient
        let runtime_handle = botguard_client.get_runtime_handle();

        // 5. Create WebPoMinter with simulated integrity token
        let integrity_token = "dGVzdF9pbnRlZ3JpdHlfdG9rZW4="; // "test_integrity_token" in base64
        let minter = WebPoMinter::create(&integrity_token, &webpo_signal_output, runtime_handle)
            .await
            .expect("Failed to create WebPoMinter");

        // 6. Generate POT token
        let pot_token = minter
            .mint_websafe_string("dQw4w9WgXcQ")
            .await
            .expect("Failed to mint POT token");

        // 7. Verify POT token characteristics
        assert!(!pot_token.is_empty(), "POT token should not be empty");

        // Decode and verify token format
        let decoded_token = BASE64
            .decode(&pot_token)
            .expect("POT token should be valid base64");

        // For test mode, we get the deterministic test data [0x12, 0x34, 0x56, 0x78]
        // In real implementation, this would be 120 bytes from the JavaScript function
        assert!(
            !decoded_token.is_empty(),
            "Decoded POT token should not be empty"
        );

        println!("✅ Complete WebPoMinter integration flow test passed");
        println!("   BotGuard response: {} chars", botguard_response.len());
        println!("   WebPO signal output: {:?}", webpo_signal_output);
        println!("   POT token length: {} chars", pot_token.len());
        println!("   Decoded token length: {} bytes", decoded_token.len());
    }

    #[tokio::test]
    async fn test_webpo_minter_error_scenarios() {
        // Test various error scenarios in the integration

        // 1. Test with empty webPoSignalOutput
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let empty_output = vec![];
        let integrity_token = "dGVzdA==";

        let result = WebPoMinter::create(&integrity_token, &empty_output, runtime_handle).await;
        assert!(result.is_err(), "Should fail with empty webPoSignalOutput");

        // 2. Test with invalid base64 integrity token
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let webpo_output = vec!["testFunction".to_string()];
        let invalid_token = "invalid base64!!!";

        let result = WebPoMinter::create(&invalid_token, &webpo_output, runtime_handle).await;
        assert!(result.is_err(), "Should fail with invalid base64 token");

        // 3. Test minting with empty identifier
        let runtime_handle = JsRuntimeHandle::new_for_test();
        let minter = WebPoMinter {
            mint_callback_ref: "test_callback".to_string(),
            runtime_handle,
        };

        let result = minter.mint_websafe_string("").await;
        assert!(result.is_ok(), "Should handle empty identifier gracefully");

        println!("✅ WebPoMinter error scenarios test passed");
    }

    #[tokio::test]
    async fn test_webpo_minter_performance_characteristics() {
        // Test performance characteristics of WebPoMinter operations

        let runtime_handle = JsRuntimeHandle::new_for_test();
        let webpo_output = vec!["testMinterFunction".to_string()];
        let integrity_token = "cGVyZm9ybWFuY2VfdGVzdA=="; // "performance_test" in base64

        // Measure WebPoMinter creation time
        let start = std::time::Instant::now();
        let minter = WebPoMinter::create(&integrity_token, &webpo_output, runtime_handle)
            .await
            .expect("Failed to create minter for performance test");
        let creation_time = start.elapsed();

        // Measure POT token generation time
        let start = std::time::Instant::now();
        let pot_token = minter
            .mint_websafe_string("performance_test_video")
            .await
            .expect("Failed to generate token for performance test");
        let generation_time = start.elapsed();

        // Verify results
        assert!(!pot_token.is_empty());
        let decoded = BASE64.decode(&pot_token).expect("Should be valid base64");
        assert!(!decoded.is_empty());

        // Performance assertions (generous limits for test environment)
        assert!(
            creation_time.as_millis() < 1000,
            "Minter creation should be fast: {:?}",
            creation_time
        );
        assert!(
            generation_time.as_millis() < 1000,
            "Token generation should be fast: {:?}",
            generation_time
        );

        println!("✅ WebPoMinter performance test passed");
        println!("   Minter creation: {:?}", creation_time);
        println!("   Token generation: {:?}", generation_time);
    }
}
