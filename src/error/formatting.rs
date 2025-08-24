//! Error formatting utilities
//!
//! Provides error formatting functions that correspond to the TypeScript
//! strerror function for consistent error message formatting.

use crate::Error;
use serde_json;
use std::error::Error as StdError;

/// Format error for display (corresponds to TypeScript strerror function)
///
/// Provides detailed error formatting with nested error causes,
/// matching the behavior of the TypeScript implementation.
pub fn format_error(error: &Error) -> String {
    format_error_with_update(error, false)
}

/// Format error and optionally update the error message
///
/// Corresponds to TypeScript strerror(e, update) function
pub fn format_error_with_update(error: &Error, update: bool) -> String {
    let formatted = match error {
        Error::BotGuard {
            code,
            message,
            info,
        } => {
            let info_str = info
                .as_ref()
                .map(|i| format!(" (info: {})", serde_json::to_string(i).unwrap_or_default()))
                .unwrap_or_default();
            format!("BGError({}): {}{}", code, message, info_str)
        }

        Error::TokenGeneration { reason, stage } => match stage {
            Some(stage) => format!("Token generation failed at {}: {}", stage, reason),
            None => format!("Token generation failed: {}", reason),
        },

        Error::Cache { operation, details } => {
            format!("Cache error during {}: {}", operation, details)
        }

        Error::Config { field, message } => {
            format!("Configuration error in {}: {}", field, message)
        }

        Error::IntegrityToken {
            details,
            response_data,
        } => {
            let response_str = response_data
                .as_ref()
                .map(|r| {
                    format!(
                        " (response: {})",
                        serde_json::to_string(r).unwrap_or_default()
                    )
                })
                .unwrap_or_default();
            format!("Integrity token error: {}{}", details, response_str)
        }

        Error::Challenge { stage, message } => {
            format!("Challenge processing failed at {}: {}", stage, message)
        }

        Error::Proxy { config, message } => {
            format!("Proxy error with config '{}': {}", config, message)
        }

        Error::Network {
            message,
            retry_count,
        } => match retry_count {
            Some(count) => format!("Network error (attempt {}): {}", count, message),
            None => format!("Network error: {}", message),
        },

        Error::Timeout {
            operation,
            duration_secs,
        } => {
            format!(
                "Operation '{}' timed out after {} seconds",
                operation, duration_secs
            )
        }

        Error::Validation {
            field,
            message,
            value,
        } => match value {
            Some(val) => format!(
                "Validation failed for {} (value: '{}'): {}",
                field, val, message
            ),
            None => format!("Validation failed for {}: {}", field, message),
        },

        // For standard errors, use their Display implementation
        _ => error.to_string(),
    };

    // Handle nested error causes (like TypeScript version)
    let mut result = formatted;
    let mut source = error.source();

    while let Some(cause) = source {
        if !result.contains(&cause.to_string()) {
            result = format!("{} (caused by {})", result, cause);
        }
        source = cause.source();
    }

    if update {
        // In TypeScript, this modifies the error message
        // In Rust, we can't modify the error, so we just return the formatted version
        // The caller would need to handle the update differently
    }

    result
}

/// Format error for JSON API responses
pub fn format_error_for_api(error: &Error) -> serde_json::Value {
    serde_json::json!({
        "error": format_error(error),
        "category": error.category(),
        "retryable": error.is_retryable(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// Format error for logging with structured data
pub fn format_error_for_logging(error: &Error) -> serde_json::Value {
    let mut log_data = serde_json::json!({
        "message": format_error(error),
        "category": error.category(),
        "retryable": error.is_retryable(),
    });

    // Add specific error details
    match error {
        Error::BotGuard { code, info, .. } => {
            log_data["botguard_code"] = serde_json::Value::String(code.clone());
            if let Some(info_val) = info {
                log_data["botguard_info"] = info_val.clone();
            }
        }
        Error::Network {
            retry_count: Some(count),
            ..
        } => {
            log_data["retry_count"] = serde_json::Value::Number((*count).into());
        }
        Error::Timeout { duration_secs, .. } => {
            log_data["timeout_duration"] = serde_json::Value::Number((*duration_secs).into());
        }
        Error::RateLimit {
            retry_after: Some(after),
            ..
        } => {
            log_data["retry_after"] = serde_json::Value::Number((*after).into());
        }
        _ => {}
    }

    log_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_botguard_error_formatting() {
        let error = Error::botguard("403", "Access denied");
        let formatted = format_error(&error);

        assert!(formatted.contains("BGError(403)"));
        assert!(formatted.contains("Access denied"));
    }

    #[test]
    fn test_nested_error_formatting() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let wrapped_error = Error::Io(io_error);

        let formatted = format_error(&wrapped_error);
        assert!(formatted.contains("File not found"));
    }

    #[test]
    fn test_token_generation_error_with_stage() {
        let error = Error::token_generation_at_stage("VM execution failed", "botguard_init");
        let formatted = format_error(&error);

        assert!(formatted.contains("Token generation failed at botguard_init"));
        assert!(formatted.contains("VM execution failed"));
    }

    #[test]
    fn test_config_error_formatting() {
        let error = Error::config("proxy_url", "Invalid URL format");
        let formatted = format_error(&error);

        assert!(formatted.contains("Configuration error in proxy_url"));
        assert!(formatted.contains("Invalid URL format"));
    }

    #[test]
    fn test_api_error_formatting() {
        let error = Error::timeout("token_generation", 30);
        let api_response = format_error_for_api(&error);

        assert!(
            api_response["error"]
                .as_str()
                .unwrap()
                .contains("timed out")
        );
        assert_eq!(api_response["category"].as_str().unwrap(), "timeout");
        assert_eq!(api_response["retryable"].as_bool().unwrap(), true);
        assert!(api_response["timestamp"].is_string());
    }

    #[test]
    fn test_logging_error_formatting() {
        let error = Error::botguard_with_info(
            "500",
            "Server error",
            serde_json::json!({"details": "VM crash"}),
        );
        let log_data = format_error_for_logging(&error);

        assert!(
            log_data["message"]
                .as_str()
                .unwrap()
                .contains("BGError(500)")
        );
        assert_eq!(log_data["category"].as_str().unwrap(), "botguard");
        assert_eq!(log_data["botguard_code"].as_str().unwrap(), "500");
        assert!(log_data["botguard_info"].is_object());
    }
}
