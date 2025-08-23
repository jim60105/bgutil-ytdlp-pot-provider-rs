//! Type definitions for POT provider
//!
//! This module contains the main data structures used for requests and responses.

pub mod request;
pub mod response;

pub use request::PotRequest;
pub use response::{ErrorResponse, PingResponse, PotResponse};
