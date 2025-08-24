//! Version information utilities
//!
//! Provides version information for the application.

/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the current application version
pub fn get_version() -> &'static str {
    VERSION
}

/// Get detailed version information including git commit
pub fn get_detailed_version() -> String {
    let version = get_version();
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    let build_date = option_env!("BUILD_DATE").unwrap_or("unknown");

    format!("{} ({}@{})", version, git_hash, build_date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert_eq!(version, "0.1.0"); // Should match Cargo.toml
    }

    #[test]
    fn test_get_detailed_version() {
        let detailed = get_detailed_version();
        assert!(!detailed.is_empty());
        assert!(detailed.contains("0.1.0"));
    }
}
