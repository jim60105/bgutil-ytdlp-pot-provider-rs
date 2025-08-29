//! WebPoMinter implementation for POT token generation
//!
//! DEPRECATED: This module provides deprecated placeholder implementations.
//! All POT token generation should now use BotGuardClient directly.

#![allow(deprecated)]

use crate::Result;

/// WebPoMinter for generating POT tokens
///
/// DEPRECATED: This is a placeholder implementation that was used during migration
/// from TypeScript. All POT token generation now uses BotGuardClient directly.
#[deprecated(
    since = "0.1.0",
    note = "Use BotGuardClient::generate_po_token instead. This struct is a legacy placeholder."
)]
#[derive(Clone, Debug)]
pub struct WebPoMinter {
    /// Placeholder for backward compatibility
    pub mint_callback_ref: String,
    /// Runtime handle for compatibility
    pub runtime_handle: JsRuntimeHandle,
}

impl WebPoMinter {
    /// Create new WebPoMinter instance
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient::generate_po_token instead. WebPoMinter is deprecated."
    )]
    pub fn new(mint_callback_ref: String, runtime_handle: JsRuntimeHandle) -> Self {
        Self {
            mint_callback_ref,
            runtime_handle,
        }
    }

    /// Generate POT token using the provided data
    ///
    /// DEPRECATED: This method always returns an error to encourage migration to BotGuardClient.
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient::generate_po_token instead. This method is deprecated."
    )]
    pub async fn generate_pot_token(&self, _data: &[u8]) -> Result<String> {
        Err(crate::Error::token_generation(
            "WebPoMinter is deprecated. Use BotGuardClient::generate_po_token instead.",
        ))
    }

    /// Mint websafe string (backward compatibility method)
    ///
    /// DEPRECATED: This method always returns an error to encourage migration to BotGuardClient.
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient::generate_po_token instead. This method is deprecated."
    )]
    pub async fn mint_websafe_string(&self, _identifier: &str) -> Result<String> {
        Err(crate::Error::token_generation(
            "WebPoMinter::mint_websafe_string is deprecated. Use BotGuardClient::generate_po_token instead.",
        ))
    }
}

/// JavaScript runtime handle for function execution
///
/// DEPRECATED: This is a placeholder implementation that was used during migration
/// from TypeScript. All POT token generation now uses BotGuardClient directly.
#[deprecated(
    since = "0.1.0",
    note = "Use BotGuardClient instead. This struct is a legacy placeholder from TypeScript migration."
)]
#[derive(Clone, Debug)]
pub struct JsRuntimeHandle {
    /// Test mode flag for backward compatibility
    _test_mode: bool,
    /// Real execution enabled flag  
    _real_execution_enabled: bool,
}

impl JsRuntimeHandle {
    /// Create new runtime handle for testing
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient instead. JsRuntimeHandle is deprecated."
    )]
    pub fn new_for_test() -> Self {
        Self {
            _test_mode: true,
            _real_execution_enabled: false,
        }
    }

    /// Create new runtime handle without deno_core dependency
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient instead. JsRuntimeHandle is deprecated."
    )]
    pub fn new_simplified() -> Self {
        Self {
            _test_mode: false,
            _real_execution_enabled: true,
        }
    }

    /// Check if the runtime is initialized
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient instead. JsRuntimeHandle is deprecated."
    )]
    pub fn is_initialized(&self) -> bool {
        true // Always initialized in simplified version
    }

    /// Check if can execute script
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient instead. JsRuntimeHandle is deprecated."
    )]
    pub fn can_execute_script(&self) -> bool {
        self._real_execution_enabled
    }

    /// Call function with bytes (simplified implementation)
    #[deprecated(
        since = "0.1.0",
        note = "Use BotGuardClient::generate_po_token instead. This method is deprecated."
    )]
    pub async fn call_function_with_bytes(
        &self,
        _function_name: &str,
        _bytes: &[u8],
    ) -> Result<Vec<u8>> {
        if self._test_mode {
            // Return test data for testing
            Ok(vec![0x12, 0x34, 0x56, 0x78])
        } else {
            Err(crate::Error::token_generation(
                "JsRuntimeHandle is deprecated. Use BotGuardClient::generate_po_token instead.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_runtime_handle_creation() {
        let handle = JsRuntimeHandle::new_for_test();
        assert!(handle._test_mode);
        assert!(!handle._real_execution_enabled);
        assert!(handle.is_initialized());
        assert!(!handle.can_execute_script());
    }

    #[test]
    fn test_js_runtime_handle_simplified() {
        let handle = JsRuntimeHandle::new_simplified();
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

    #[tokio::test]
    async fn test_js_runtime_handle_call_function_real_mode() {
        let handle = JsRuntimeHandle::new_simplified();
        let result = handle
            .call_function_with_bytes("test_function", &[1, 2, 3, 4])
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("deprecated"));
    }

    #[test]
    fn test_webpo_minter_creation() {
        let handle = JsRuntimeHandle::new_for_test();
        let minter = WebPoMinter::new("test_callback".to_string(), handle);

        assert_eq!(minter.mint_callback_ref, "test_callback");
        assert!(minter.runtime_handle._test_mode);
    }

    #[tokio::test]
    async fn test_webpo_minter_generate_pot_token() {
        let handle = JsRuntimeHandle::new_for_test();
        let minter = WebPoMinter::new("test_callback".to_string(), handle);

        let result = minter.generate_pot_token(&[1, 2, 3, 4]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("deprecated"));
    }
}
