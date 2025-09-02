//! Network configuration and proxy support
//!
//! This module handles HTTP client configuration, proxy settings,
//! and network-related functionality.

use crate::Result;
use reqwest::{Client, Proxy};
use std::collections::HashMap;
use std::time::Duration;

/// Proxy specification for network requests matching TypeScript ProxySpec
#[derive(Debug, Clone, Default)]
pub struct ProxySpec {
    /// Proxy URL
    pub proxy_url: Option<String>,
    /// Source address
    pub source_address: Option<String>,
    /// Disable TLS verification
    pub disable_tls_verification: bool,
    /// IP family (4 or 6)
    pub ip_family: Option<u8>,
}

impl ProxySpec {
    /// Create new proxy specification
    pub fn new() -> Self {
        Self::default()
    }

    /// Set proxy URL
    pub fn with_proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Set source address
    pub fn with_source_address(mut self, source_address: impl Into<String>) -> Self {
        let addr = source_address.into();
        self.ip_family = if addr.contains(':') { Some(6) } else { Some(4) };
        self.source_address = Some(addr);
        self
    }

    /// Set TLS verification flag
    pub fn with_disable_tls_verification(mut self, disable: bool) -> Self {
        self.disable_tls_verification = disable;
        self
    }

    /// Generate cache key for minter cache
    /// Corresponds to TypeScript CacheSpec.key
    pub fn cache_key(&self, remote_host: Option<&str>) -> String {
        let ip = remote_host.map(|h| h.to_string());
        if let Some(ip) = ip {
            serde_json::to_string(&ip).unwrap_or_else(|_| "null".to_string())
        } else {
            let data = vec![
                self.proxy_url.as_deref().unwrap_or("null"),
                self.source_address.as_deref().unwrap_or("null"),
            ];
            serde_json::to_string(&data).unwrap_or_else(|_| "[]".to_string())
        }
    }
}

/// Network manager for HTTP requests
#[derive(Debug, Clone)]
pub struct NetworkManager {
    /// Base HTTP client
    client: Client,
}

impl NetworkManager {
    /// Create new network manager with proxy configuration
    pub fn new(proxy_spec: &ProxySpec) -> Result<Self> {
        let mut client_builder = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(Duration::from_secs(30));

        // Configure proxy if specified
        if let Some(proxy_url) = &proxy_spec.proxy_url {
            let proxy = Proxy::all(proxy_url).map_err(|e| {
                crate::Error::proxy(proxy_url, &format!("Invalid proxy URL: {}", e))
            })?;
            client_builder = client_builder.proxy(proxy);
        }

        // Configure TLS verification
        if proxy_spec.disable_tls_verification {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder.build().map_err(|e| {
            crate::Error::proxy(
                "client_builder",
                &format!("Failed to create HTTP client: {}", e),
            )
        })?;

        Ok(Self { client })
    }

    /// Get the configured HTTP client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Perform HTTP request with retry logic
    ///
    /// Corresponds to TypeScript: `getFetch` method (L438-483)
    pub async fn fetch_with_retry(
        &self,
        url: &str,
        options: RequestOptions,
        max_retries: u32,
        interval_ms: u64,
    ) -> Result<reqwest::Response> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self.perform_request(url, &options).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| crate::Error::internal("No error recorded during retries")))
    }

    /// Perform single HTTP request
    async fn perform_request(
        &self,
        url: &str,
        options: &RequestOptions,
    ) -> Result<reqwest::Response> {
        let mut request = match options.method.as_str() {
            "GET" => self.client.get(url),
            "POST" => {
                let mut req = self.client.post(url);
                if let Some(body) = &options.body {
                    req = req.body(body.clone());
                }
                req
            }
            _ => return Err(crate::Error::internal("Unsupported HTTP method")),
        };

        // Add headers
        for (key, value) in &options.headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| crate::Error::internal(format!("HTTP request failed: {}", e)))?;

        Ok(response)
    }
}

/// HTTP request options
#[derive(Debug, Clone)]
pub struct RequestOptions {
    /// HTTP method
    pub method: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Option<String>,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

impl RequestOptions {
    /// Create new request options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set HTTP method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    /// Add header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set request body
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_current_cache_key_behavior() {
        // Test case 1: no proxy, no source_address, no remote_host
        let spec1 = ProxySpec::new();
        let key1 = spec1.cache_key(None);
        println!("Default spec cache key: '{}'", key1);

        // Test case 2: with proxy
        let spec2 = ProxySpec::new().with_proxy("http://proxy:8080");
        let key2 = spec2.cache_key(None);
        println!("With proxy cache key: '{}'", key2);

        // Test case 3: with remote host
        let spec3 = ProxySpec::new();
        let key3 = spec3.cache_key(Some("192.168.1.100"));
        println!("With remote host cache key: '{}'", key3);

        // Test case 4: with proxy and source address
        let spec4 = ProxySpec::new()
            .with_proxy("http://proxy:8080")
            .with_source_address("192.168.1.1");
        let key4 = spec4.cache_key(None);
        println!("With proxy and source address cache key: '{}'", key4);

        // Let's see the actual JSON format
        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
        assert!(!key3.is_empty());
        assert!(!key4.is_empty());
    }

    #[test]
    fn test_proxy_spec_creation() {
        let spec = ProxySpec::new();
        assert!(spec.proxy_url.is_none());
        assert!(spec.source_address.is_none());
        assert!(!spec.disable_tls_verification);
        assert!(spec.ip_family.is_none());
    }

    #[test]
    fn test_proxy_spec_builder() {
        let spec = ProxySpec::new()
            .with_proxy("http://proxy:8080")
            .with_source_address("192.168.1.1")
            .with_disable_tls_verification(true);

        assert_eq!(spec.proxy_url, Some("http://proxy:8080".to_string()));
        assert_eq!(spec.source_address, Some("192.168.1.1".to_string()));
        assert!(spec.disable_tls_verification);
        assert_eq!(spec.ip_family, Some(4));
    }

    #[test]
    fn test_proxy_spec_ipv6() {
        let spec = ProxySpec::new().with_source_address("2001:db8::1");

        assert_eq!(spec.ip_family, Some(6));
    }

    #[test]
    fn test_proxy_spec_cache_key() {
        let spec = ProxySpec::new()
            .with_proxy("http://proxy:8080")
            .with_source_address("192.168.1.1");

        let key1 = spec.cache_key(None);
        let key2 = spec.cache_key(Some("youtube.com"));

        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_request_options_builder() {
        let options = RequestOptions::new()
            .with_method("POST")
            .with_header("Content-Type", "application/json")
            .with_body(r#"{"test": "data"}"#);

        assert_eq!(options.method, "POST");
        assert_eq!(
            options.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(options.body, Some(r#"{"test": "data"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_network_manager_creation() {
        let spec = ProxySpec::new();
        let manager = NetworkManager::new(&spec);

        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_network_manager_with_proxy() {
        let spec = ProxySpec::new().with_proxy("http://proxy:8080");

        // This might fail if proxy URL is invalid format, but should handle gracefully
        let result = NetworkManager::new(&spec);
        // We accept either success or a proxy error for this test
        match result {
            Ok(_) => {}                                         // Success
            Err(e) => assert!(e.to_string().contains("proxy")), // Expected proxy error
        }
    }
}
