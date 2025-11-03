//! Custom serde deserializers for flexible type handling
//!
//! Provides custom deserializers to handle various input formats from different clients.

use serde::{Deserialize, Deserializer, de};

/// Deserialize a flexible boolean value that can be:
/// - JSON boolean: `true`, `false`
/// - Integer: `0` (false), any non-zero integer (true, negative integers treated as false for safety)
/// - String: `"0"`, `"1"`, `"false"`, `"true"` (case-insensitive)
///
/// This is needed because different client implementations may send boolean values
/// in different formats (e.g., Python might send 0/1 instead of true/false).
///
/// Note: Strings like "yes"/"no", "y"/"n", and empty strings are NOT supported
/// and will return an error to ensure explicit boolean semantics in the API.
pub fn deserialize_flexible_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    // Use an intermediate Value type to handle multiple input types
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FlexibleBool {
        Bool(bool),
        Int(i64),
        String(String),
    }

    // Deserialize into Option first to handle None/null values
    let value: Option<FlexibleBool> = Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(FlexibleBool::Bool(b)) => Ok(Some(b)),
        Some(FlexibleBool::Int(i)) => {
            // 0 is false, any non-zero positive integer is true
            // Negative integers are treated as false for safety
            Ok(Some(i > 0))
        }
        Some(FlexibleBool::String(s)) => {
            // Parse string representation - only accept explicit boolean strings
            let s_lower = s.trim().to_lowercase();
            match s_lower.as_str() {
                "true" | "1" => Ok(Some(true)),
                "false" | "0" => Ok(Some(false)),
                _ => Err(de::Error::custom(format!("invalid boolean string: {}", s))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(default, deserialize_with = "deserialize_flexible_bool")]
        value: Option<bool>,
    }

    #[test]
    fn test_deserialize_json_bool_true() {
        let json = json!({"value": true});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_json_bool_false() {
        let json = json!({"value": false});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_int_zero() {
        let json = json!({"value": 0});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_int_one() {
        let json = json!({"value": 1});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_int_positive() {
        let json = json!({"value": 42});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_int_negative() {
        let json = json!({"value": -1});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_string_true() {
        let json = json!({"value": "true"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_string_false() {
        let json = json!({"value": "false"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_string_true_case_insensitive() {
        let json = json!({"value": "True"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));

        let json = json!({"value": "TRUE"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_string_false_case_insensitive() {
        let json = json!({"value": "False"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));

        let json = json!({"value": "FALSE"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_string_one() {
        let json = json!({"value": "1"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_string_zero() {
        let json = json!({"value": "0"});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(false));
    }

    #[test]
    fn test_deserialize_string_whitespace() {
        let json = json!({"value": "  true  "});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, Some(true));
    }

    #[test]
    fn test_deserialize_null() {
        let json = json!({"value": null});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, None);
    }

    #[test]
    fn test_deserialize_missing_field() {
        let json = json!({});
        let result: TestStruct = serde_json::from_value(json).unwrap();
        assert_eq!(result.value, None);
    }

    #[test]
    fn test_deserialize_invalid_string() {
        let json = json!({"value": "invalid"});
        let result: Result<TestStruct, _> = serde_json::from_value(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid boolean string"));
    }

    #[test]
    fn test_deserialize_string_yes_rejected() {
        // "yes" should be rejected for API clarity
        let json = json!({"value": "yes"});
        let result: Result<TestStruct, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_string_empty_rejected() {
        // Empty string should be rejected for explicit validation
        let json = json!({"value": ""});
        let result: Result<TestStruct, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }
}
