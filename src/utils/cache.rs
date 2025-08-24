//! File-based cache management for script mode
//!
//! Implements persistent storage for session data using JSON files,
//! following XDG Base Directory Specification.

use crate::{Result, session::manager::SessionDataCaches, types::SessionData};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, warn};

/// File-based cache manager
#[derive(Debug)]
pub struct FileCache {
    /// Path to cache file
    cache_path: PathBuf,
}

/// Serializable cache entry for file storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// POT token
    #[serde(rename = "poToken")]
    po_token: String,
    /// Content binding  
    #[serde(rename = "contentBinding")]
    content_binding: String,
    /// Expiration timestamp (ISO 8601 format)
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

impl FileCache {
    /// Create new file cache manager
    pub fn new(cache_path: PathBuf) -> Self {
        Self { cache_path }
    }

    /// Load cache from file
    ///
    /// Corresponds to TypeScript cache loading logic (L75-105)
    pub async fn load_cache(&self) -> Result<SessionDataCaches> {
        if !self.cache_path.exists() {
            debug!("Cache file does not exist: {:?}", self.cache_path);
            return Ok(SessionDataCaches::new());
        }

        match fs::read_to_string(&self.cache_path).await {
            Ok(content) => {
                debug!("Loading cache from: {:?}", self.cache_path);
                self.parse_cache_content(&content)
            }
            Err(e) => {
                warn!("Failed to read cache file {:?}: {}", self.cache_path, e);
                Ok(SessionDataCaches::new())
            }
        }
    }

    /// Save cache to file
    ///
    /// Corresponds to TypeScript cache saving logic (L117-127)
    pub async fn save_cache(&self, caches: SessionDataCaches) -> Result<()> {
        let cache_entries = self.convert_to_cache_entries(caches);
        let content = serde_json::to_string_pretty(&cache_entries)?;

        // Ensure parent directory exists
        if let Some(parent) = self.cache_path.parent()
            && let Err(e) = fs::create_dir_all(parent).await
        {
            error!("Failed to create cache directory {:?}: {}", parent, e);
            return Err(crate::Error::cache(
                "directory_creation",
                &format!("Directory creation failed: {}", e),
            ));
        }

        match fs::write(&self.cache_path, content).await {
            Ok(_) => {
                debug!("Cache saved to: {:?}", self.cache_path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to write cache file {:?}: {}", self.cache_path, e);
                Err(crate::Error::cache(
                    "file_write",
                    &format!("Write failed: {}", e),
                ))
            }
        }
    }

    /// Parse cache content from JSON
    fn parse_cache_content(&self, content: &str) -> Result<SessionDataCaches> {
        let cache_entries: std::collections::HashMap<String, CacheEntry> =
            match serde_json::from_str(content) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Error parsing cache: {}", e);
                    return Ok(SessionDataCaches::new());
                }
            };

        let mut session_caches = SessionDataCaches::new();

        for (content_binding, entry) in cache_entries {
            match self.parse_cache_entry(&content_binding, entry) {
                Ok(session_data) => {
                    session_caches.insert(content_binding, session_data);
                }
                Err(e) => {
                    warn!("Ignored cache entry for '{}': {}", content_binding, e);
                }
            }
        }

        debug!("Loaded {} cache entries", session_caches.len());
        Ok(session_caches)
    }

    /// Parse individual cache entry
    fn parse_cache_entry(&self, content_binding: &str, entry: CacheEntry) -> Result<SessionData> {
        let expires_at = DateTime::parse_from_rfc3339(&entry.expires_at)
            .map_err(|e| {
                crate::Error::cache("date_parse", &format!("Invalid expiration date: {}", e))
            })?
            .with_timezone(&Utc);

        // Validate that the entry hasn't expired
        if expires_at <= Utc::now() {
            return Err(crate::Error::cache("validation", "Entry has expired"));
        }

        Ok(SessionData::new(
            entry.po_token,
            content_binding,
            expires_at,
        ))
    }

    /// Convert session data to cache entries for serialization
    fn convert_to_cache_entries(
        &self,
        caches: SessionDataCaches,
    ) -> std::collections::HashMap<String, CacheEntry> {
        caches
            .into_iter()
            .map(|(content_binding, session_data)| {
                let entry = CacheEntry {
                    po_token: session_data.po_token,
                    content_binding: session_data.content_binding.clone(),
                    expires_at: session_data.expires_at.to_rfc3339(),
                };
                (content_binding, entry)
            })
            .collect()
    }
}

/// Get cache directory path following XDG Base Directory Specification
///
/// Corresponds to TypeScript implementation (L8-30)
pub fn get_cache_path() -> anyhow::Result<PathBuf> {
    let cache_dir = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg_cache).join("bgutil-ytdlp-pot-provider")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache").join("bgutil-ytdlp-pot-provider")
    } else {
        // Fallback to current directory if home is not available
        warn!("Could not determine home directory, using current directory for cache");
        std::env::current_dir()?.join(".cache")
    };

    Ok(cache_dir.join("cache.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_save_and_load_cache() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().to_path_buf();
        let file_cache = FileCache::new(cache_path);

        // Create test session data
        let mut session_caches = SessionDataCaches::new();
        let expires_at = Utc::now() + Duration::hours(6);
        session_caches.insert(
            "test_video_id".to_string(),
            SessionData::new("test_token", "test_video_id", expires_at),
        );

        // Save cache
        file_cache.save_cache(session_caches.clone()).await.unwrap();

        // Load cache
        let loaded_caches = file_cache.load_cache().await.unwrap();

        assert_eq!(loaded_caches.len(), 1);
        let loaded_entry = loaded_caches.get("test_video_id").unwrap();
        assert_eq!(loaded_entry.po_token, "test_token");
        assert_eq!(loaded_entry.content_binding, "test_video_id");
    }

    #[tokio::test]
    async fn test_load_nonexistent_cache() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().with_extension("nonexistent");
        let file_cache = FileCache::new(cache_path);

        let result = file_cache.load_cache().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_expired_entries_filtered() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().to_path_buf();

        // Create cache file with expired entry
        let expired_entry = CacheEntry {
            po_token: "expired_token".to_string(),
            content_binding: "expired_video".to_string(),
            expires_at: (Utc::now() - Duration::hours(1)).to_rfc3339(), // Expired
        };

        let valid_entry = CacheEntry {
            po_token: "valid_token".to_string(),
            content_binding: "valid_video".to_string(),
            expires_at: (Utc::now() + Duration::hours(1)).to_rfc3339(), // Valid
        };

        let mut cache_entries = std::collections::HashMap::new();
        cache_entries.insert("expired_video".to_string(), expired_entry);
        cache_entries.insert("valid_video".to_string(), valid_entry);

        let content = serde_json::to_string(&cache_entries).unwrap();
        tokio::fs::write(&cache_path, content).await.unwrap();

        // Load cache
        let file_cache = FileCache::new(cache_path);
        let loaded_caches = file_cache.load_cache().await.unwrap();

        // Only valid entry should be loaded
        assert_eq!(loaded_caches.len(), 1);
        assert!(loaded_caches.contains_key("valid_video"));
        assert!(!loaded_caches.contains_key("expired_video"));
    }

    #[tokio::test]
    async fn test_malformed_cache_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().to_path_buf();

        // Write malformed JSON
        tokio::fs::write(&cache_path, "invalid json content")
            .await
            .unwrap();

        let file_cache = FileCache::new(cache_path);
        let result = file_cache.load_cache().await.unwrap();

        // Should return empty cache on parse error
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_cache_path_with_xdg() {
        unsafe {
            std::env::set_var("XDG_CACHE_HOME", "/tmp/test_cache");
        }

        let cache_path = get_cache_path().unwrap();

        assert!(
            cache_path
                .to_string_lossy()
                .contains("bgutil-ytdlp-pot-provider")
        );
        assert!(cache_path.to_string_lossy().ends_with("cache.json"));

        unsafe {
            std::env::remove_var("XDG_CACHE_HOME");
        }
    }
}
