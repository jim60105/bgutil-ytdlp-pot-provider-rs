//! HTTP server implementation
//!
//! This module contains the HTTP server implementation using Axum framework.

pub mod app;
pub mod handlers;

pub use app::create_app;
