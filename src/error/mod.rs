//! Error handling for the POT provider
//!
//! This module defines error types and handling patterns used throughout the application.

pub mod formatting;
pub mod types;

pub use formatting::{
    format_error, format_error_for_api, format_error_for_logging, format_error_with_update,
};
pub use types::{Error, Result};
