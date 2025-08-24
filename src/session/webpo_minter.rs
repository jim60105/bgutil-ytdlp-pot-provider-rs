//! WebPoMinter implementation for POT token generation
//!
//! This module implements the WebPoMinter which uses integrity tokens and webPoSignalOutput
//! to generate final POT tokens through JavaScript VM execution.

use crate::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// WebPoMinter for generating POT tokens
#[derive(Debug, Clone)]
pub struct WebPoMinter {
    /// JavaScript mint callback function reference
    pub mint_callback_ref: String,
    /// JavaScript runtime handle for function calls
    pub runtime_handle: JsRuntimeHandle,
}

/// JavaScript runtime handle for function execution
#[derive(Debug, Clone)]
pub struct JsRuntimeHandle {
    // TODO: Add actual runtime reference when integrating with BotGuardClient
    _placeholder: String,
}

impl JsRuntimeHandle {
    /// Create new runtime handle for testing
    pub fn new_for_test() -> Self {
        Self {
            _placeholder: "test_handle".to_string(),
        }
    }

    /// Call JavaScript function with byte array input
    pub async fn call_function_with_bytes(
        &self,
        _function_ref: &str,
        _bytes: &[u8],
    ) -> Result<Vec<u8>> {
        // TODO: Implement actual JavaScript function call
        // For now, return test data
        Ok(vec![0x12, 0x34, 0x56, 0x78])
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

        let result = WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle)
            .await;

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

        let result = WebPoMinter::create(&integrity_token, &web_po_signal_output, runtime_handle)
            .await;

        assert!(result.is_err());
    }
}