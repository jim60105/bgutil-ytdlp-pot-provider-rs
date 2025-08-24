//! Configuration management for the POT provider
//!
//! This module handles loading and managing configuration settings
//! for both HTTP server and script modes.

pub mod loader;
pub mod settings;

pub use loader::ConfigLoader;
pub use settings::Settings;
