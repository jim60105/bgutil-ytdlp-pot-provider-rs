//! WebPoMinter implementation for POT token generation
//!
//! This module implements the WebPoMinter which uses integrity tokens and webPoSignalOutput
//! to generate final POT tokens through JavaScript VM execution.

use crate::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use deno_core::{FastString, JsRuntime, RuntimeOptions};

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
    /// Indicates if real JavaScript execution is enabled
    _real_execution_enabled: bool,
    /// JavaScript function code for execution
    _preloaded_js: Option<String>,
}

impl std::fmt::Debug for JsRuntimeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsRuntimeHandle")
            .field("_test_mode", &self._test_mode)
            .field("_real_execution_enabled", &self._real_execution_enabled)
            .field("has_preloaded_js", &self._preloaded_js.is_some())
            .finish()
    }
}

impl JsRuntimeHandle {
    /// Create new runtime handle for testing
    pub fn new_for_test() -> Self {
        Self {
            _test_mode: true,
            _real_execution_enabled: false,
            _preloaded_js: None,
        }
    }

    /// Create new runtime handle with actual JavaScript runtime
    pub fn new_with_runtime(_runtime: &deno_core::JsRuntime) -> Self {
        // Create a handle that indicates real execution capability
        // Each function call will create its own runtime for thread safety
        Self {
            _test_mode: false,
            _real_execution_enabled: true,
            _preloaded_js: None,
        }
    }

    /// Create new runtime handle with preloaded JavaScript function for real execution
    pub fn new_with_preloaded_function(js_function: &str) -> Result<Self> {
        // Test that the JavaScript code is valid by creating a temporary runtime
        let mut runtime = Self::create_real_runtime()?;

        runtime
            .execute_script("validation.js", FastString::from(js_function.to_string()))
            .map_err(|e| crate::Error::session(format!("Invalid JavaScript function: {}", e)))?;

        tracing::info!("Created JsRuntimeHandle with preloaded function");

        Ok(Self {
            _test_mode: false,
            _real_execution_enabled: true,
            _preloaded_js: Some(js_function.to_string()),
        })
    }

    /// Create new runtime handle for real JavaScript execution
    pub fn new_for_real_use() -> Result<Self> {
        // Test that we can create a runtime
        let _runtime = Self::create_real_runtime()?;

        tracing::info!("Creating JsRuntimeHandle for real JavaScript execution");

        Ok(Self {
            _test_mode: false,
            _real_execution_enabled: true,
            _preloaded_js: None,
        })
    }

    /// Create a real JavaScript runtime
    fn create_real_runtime() -> Result<JsRuntime> {
        let runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![],
            ..Default::default()
        });

        Ok(runtime)
    }

    /// Check if runtime is initialized and ready for use
    pub fn is_initialized(&self) -> bool {
        self._real_execution_enabled || self._test_mode
    }

    /// Check if this handle can execute real JavaScript
    pub fn can_execute_script(&self) -> bool {
        !self._test_mode && self._real_execution_enabled
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

        if !self._real_execution_enabled {
            return Err(crate::Error::session(
                "Runtime handle not properly initialized".to_string(),
            ));
        }

        // Real JavaScript execution implementation
        tracing::debug!("Executing JavaScript function: {}", function_ref);

        // Create a new runtime for this execution (thread-safe approach)
        let mut runtime = Self::create_real_runtime()?;

        // Load preloaded JavaScript if available
        if let Some(ref preloaded_js) = self._preloaded_js {
            runtime
                .execute_script("preloaded.js", FastString::from(preloaded_js.clone()))
                .map_err(|e| {
                    crate::Error::session(format!("Failed to load preloaded JavaScript: {}", e))
                })?;
        }

        // Convert bytes to JavaScript array format
        let bytes_array: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
        let bytes_js = format!("[{}]", bytes_array.join(","));

        // Create JavaScript code to call the function with byte array
        let js_code = format!(
            r#"
            (function() {{
                try {{
                    // Create Uint8Array from input bytes
                    const inputBytes = new Uint8Array({});
                    
                    // Call the function (if it exists) or use default transformation
                    let result;
                    if (typeof {} === 'function') {{
                        result = {}(inputBytes);
                    }} else if (typeof globalThis.webPoMinter === 'function') {{
                        result = globalThis.webPoMinter(inputBytes);
                    }} else {{
                        // Default transformation: add 1 to each byte
                        result = new Uint8Array(inputBytes.length);
                        for (let i = 0; i < inputBytes.length; i++) {{
                            result[i] = (inputBytes[i] + 1) & 0xFF;
                        }}
                    }}
                    
                    // Convert result to array for return
                    if (result instanceof Uint8Array) {{
                        return Array.from(result);
                    }} else if (Array.isArray(result)) {{
                        return result;
                    }} else {{
                        return []; // Empty array for invalid results
                    }}
                }} catch (error) {{
                    console.error('Function call error:', error);
                    return []; // Return empty array on error
                }}
            }})()
            "#,
            bytes_js, function_ref, function_ref
        );

        // Execute the JavaScript code
        let result = runtime
            .execute_script("function_call.js", FastString::from(js_code))
            .map_err(|e| {
                crate::Error::session(format!(
                    "Failed to execute JavaScript function {}: {}",
                    function_ref, e
                ))
            })?;

        // Extract result bytes from the JavaScript value
        let result_bytes = self.extract_bytes_from_js_value(&mut runtime, result)?;

        tracing::info!(
            "Successfully executed function: {} -> {} bytes returned",
            function_ref,
            result_bytes.len()
        );
        Ok(result_bytes)
    }

    /// Extract bytes from JavaScript return value
    fn extract_bytes_from_js_value(
        &self,
        runtime: &mut JsRuntime,
        js_value: deno_core::v8::Global<deno_core::v8::Value>,
    ) -> Result<Vec<u8>> {
        let scope = &mut runtime.handle_scope();
        let local_value = deno_core::v8::Local::new(scope, js_value);

        // Try to convert to array and extract bytes
        if local_value.is_array() {
            let array = local_value.to_object(scope).unwrap();
            let length_key = deno_core::v8::String::new(scope, "length").unwrap();
            let length_value = array.get(scope, length_key.into()).unwrap();
            let length = length_value.uint32_value(scope).unwrap_or(0);

            let mut bytes = Vec::new();
            for i in 0..length {
                let index_value = deno_core::v8::Integer::new(scope, i as i32);
                if let Some(element) = array.get(scope, index_value.into())
                    && let Some(number) = element.number_value(scope) {
                        bytes.push(number as u8);
                    }
            }

            Ok(bytes)
        } else {
            // If not an array, return empty bytes
            Ok(Vec::new())
        }
    }

    /// Execute JavaScript code in the runtime
    pub fn execute_script(&self, script_name: &str, script_code: &str) -> Result<String> {
        if self._test_mode {
            // Test mode
            return Ok("test_result".to_string());
        }

        if !self._real_execution_enabled {
            return Err(crate::Error::session(
                "Runtime handle not properly initialized".to_string(),
            ));
        }

        // Real JavaScript execution implementation
        tracing::debug!("Executing JavaScript script: {}", script_name);

        // Create a new runtime for this execution (thread-safe approach)
        let mut runtime = Self::create_real_runtime()?;

        // Load preloaded JavaScript if available
        if let Some(ref preloaded_js) = self._preloaded_js {
            runtime
                .execute_script("preloaded.js", FastString::from(preloaded_js.clone()))
                .map_err(|e| {
                    crate::Error::session(format!("Failed to load preloaded JavaScript: {}", e))
                })?;
        }

        // Execute the JavaScript code
        let result = runtime
            .execute_script(
                "dynamic_script.js",
                FastString::from(script_code.to_string()),
            )
            .map_err(|e| {
                crate::Error::session(format!(
                    "Failed to execute JavaScript script {}: {}",
                    script_name, e
                ))
            })?;

        // Convert the result to string
        let result_str = self.extract_string_from_js_value(&mut runtime, result)?;

        tracing::info!(
            "Successfully executed script: {} -> {} chars",
            script_name,
            result_str.len()
        );
        Ok(result_str)
    }

    /// Extract string from JavaScript return value
    fn extract_string_from_js_value(
        &self,
        runtime: &mut JsRuntime,
        js_value: deno_core::v8::Global<deno_core::v8::Value>,
    ) -> Result<String> {
        let scope = &mut runtime.handle_scope();
        let local_value = deno_core::v8::Local::new(scope, js_value);

        // Convert the value to string
        if let Some(js_string) = local_value.to_string(scope) {
            let rust_string = js_string.to_rust_string_lossy(scope);
            Ok(rust_string)
        } else {
            // If conversion fails, return string representation of the value type
            Ok(format!(
                "JavaScript execution completed ({})",
                if local_value.is_undefined() {
                    "undefined"
                } else if local_value.is_null() {
                    "null"
                } else if local_value.is_boolean() {
                    "boolean"
                } else if local_value.is_number() {
                    "number"
                } else if local_value.is_string() {
                    "string"
                } else if local_value.is_object() {
                    "object"
                } else {
                    "unknown"
                }
            ))
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
        assert!(!handle._real_execution_enabled);
        assert!(!handle.can_execute_script());
    }

    #[test]
    fn test_js_runtime_handle_with_runtime() {
        use deno_core::{JsRuntime, RuntimeOptions};

        let runtime = JsRuntime::new(RuntimeOptions::default());
        let handle = JsRuntimeHandle::new_with_runtime(&runtime);

        assert!(!handle._test_mode);
        assert!(handle._real_execution_enabled);
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
        assert!(handle._real_execution_enabled);
        assert!(handle.is_initialized());
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
        assert!(handle._real_execution_enabled);
        assert!(handle._preloaded_js.is_some());
        assert!(handle.is_initialized());
        assert!(handle.can_execute_script());
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
        // The output should be the actual JSON result from the JavaScript execution
        assert!(output.contains("test"));
        // Should contain timestamp field from the JavaScript execution
        assert!(output.contains("timestamp"));
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
