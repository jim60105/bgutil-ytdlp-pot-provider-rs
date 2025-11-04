//! Comprehensive tests for optional configuration fields
//!
//! Tests that all configuration fields can be omitted and will use their default values
//! when not specified in the TOML configuration file.

use bgutil_ytdlp_pot_provider::config::Settings;
use std::io::Write;
use std::sync::Mutex;
use tempfile::NamedTempFile;

// Static mutex to ensure environment variable tests don't interfere with each other
static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_server_host_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.server.host, "127.0.0.1");
    assert_eq!(settings.server.port, 4416); // Default value
    assert_eq!(settings.server.timeout.as_secs(), 30); // Default value
    assert!(settings.server.enable_cors); // Default value
    assert_eq!(settings.server.max_body_size, 1024 * 1024); // Default value
}

#[test]
fn test_server_port_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
port = 8080
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.server.host, "::"); // Default value
    assert_eq!(settings.server.port, 8080);
    assert_eq!(settings.server.timeout.as_secs(), 30); // Default value
}

#[test]
fn test_server_timeout_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
timeout = 60
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.server.timeout.as_secs(), 60);
    assert_eq!(settings.server.host, "::"); // Default value
    assert_eq!(settings.server.port, 4416); // Default value
}

#[test]
fn test_server_enable_cors_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
enable_cors = false
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(!settings.server.enable_cors);
    assert_eq!(settings.server.host, "::"); // Default value
    assert_eq!(settings.server.port, 4416); // Default value
}

#[test]
fn test_server_max_body_size_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
max_body_size = 2097152
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.server.max_body_size, 2097152);
    assert_eq!(settings.server.host, "::"); // Default value
    assert_eq!(settings.server.port, 4416); // Default value
}

#[test]
fn test_server_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.server.host, "::");
    assert_eq!(settings.server.port, 4416);
    assert_eq!(settings.server.timeout.as_secs(), 30);
    assert!(settings.server.enable_cors);
    assert_eq!(settings.server.max_body_size, 1024 * 1024);
}

#[test]
fn test_token_ttl_hours_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
ttl_hours = 12
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.token.ttl_hours, 12);
    assert!(settings.token.enable_cache); // Default value
    assert_eq!(settings.token.max_cache_entries, 1000); // Default value
    assert_eq!(settings.token.cache_cleanup_interval, 60); // Default value
    assert_eq!(settings.token.pot_cache_duration, 1800); // Default value
    assert_eq!(settings.token.pot_generation_timeout, 30); // Default value
}

#[test]
fn test_token_enable_cache_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
enable_cache = false
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(!settings.token.enable_cache);
    assert_eq!(settings.token.ttl_hours, 6); // Default value
    assert_eq!(settings.token.max_cache_entries, 1000); // Default value
}

#[test]
fn test_token_max_cache_entries_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
max_cache_entries = 500
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.token.max_cache_entries, 500);
    assert_eq!(settings.token.ttl_hours, 6); // Default value
}

#[test]
fn test_token_cache_cleanup_interval_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
cache_cleanup_interval = 120
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.token.cache_cleanup_interval, 120);
    assert_eq!(settings.token.ttl_hours, 6); // Default value
}

#[test]
fn test_token_pot_cache_duration_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
pot_cache_duration = 3600
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.token.pot_cache_duration, 3600);
    assert_eq!(settings.token.ttl_hours, 6); // Default value
}

#[test]
fn test_token_pot_generation_timeout_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
pot_generation_timeout = 60
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.token.pot_generation_timeout, 60);
    assert_eq!(settings.token.ttl_hours, 6); // Default value
}

#[test]
fn test_token_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[token]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.token.ttl_hours, 6);
    assert!(settings.token.enable_cache);
    assert_eq!(settings.token.max_cache_entries, 1000);
    assert_eq!(settings.token.cache_cleanup_interval, 60);
    assert_eq!(settings.token.pot_cache_duration, 1800);
    assert_eq!(settings.token.pot_generation_timeout, 30);
}

#[test]
fn test_logging_level_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[logging]
level = "debug"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.logging.level, "debug");
    assert!(!settings.logging.verbose); // Default value
    assert_eq!(settings.logging.format, "text"); // Default value
    assert!(settings.logging.log_requests); // Default value
}

#[test]
fn test_logging_verbose_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[logging]
verbose = true
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(settings.logging.verbose);
    assert_eq!(settings.logging.level, "info"); // Default value
}

#[test]
fn test_logging_format_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[logging]
format = "json"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.logging.format, "json");
    assert_eq!(settings.logging.level, "info"); // Default value
}

#[test]
fn test_logging_log_requests_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[logging]
log_requests = false
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(!settings.logging.log_requests);
    assert_eq!(settings.logging.level, "info"); // Default value
}

#[test]
fn test_logging_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[logging]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.logging.level, "info");
    assert!(!settings.logging.verbose);
    assert_eq!(settings.logging.format, "text");
    assert!(settings.logging.log_requests);
}

#[test]
fn test_network_https_proxy_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
https_proxy = "https://proxy.example.com:8080"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.network.https_proxy,
        Some("https://proxy.example.com:8080".to_string())
    );
    assert_eq!(settings.network.http_proxy, None); // Default value
    assert_eq!(settings.network.connect_timeout, 30); // Default value
}

#[test]
fn test_network_http_proxy_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
http_proxy = "http://proxy.example.com:8080"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.network.http_proxy,
        Some("http://proxy.example.com:8080".to_string())
    );
    assert_eq!(settings.network.https_proxy, None); // Default value
}

#[test]
fn test_network_all_proxy_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
all_proxy = "socks5://proxy.example.com:1080"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.network.all_proxy,
        Some("socks5://proxy.example.com:1080".to_string())
    );
    assert_eq!(settings.network.https_proxy, None); // Default value
}

#[test]
fn test_network_connect_timeout_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
connect_timeout = 60
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.network.connect_timeout, 60);
    assert_eq!(settings.network.request_timeout, 60); // Default value
}

#[test]
fn test_network_request_timeout_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
request_timeout = 120
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.network.request_timeout, 120);
    assert_eq!(settings.network.connect_timeout, 30); // Default value
}

#[test]
fn test_network_max_retries_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
max_retries = 5
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.network.max_retries, 5);
    assert_eq!(settings.network.retry_interval, 5000); // Default value
}

#[test]
fn test_network_retry_interval_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
retry_interval = 10000
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.network.retry_interval, 10000);
    assert_eq!(settings.network.max_retries, 3); // Default value
}

#[test]
fn test_network_user_agent_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
user_agent = "Custom User Agent"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.network.user_agent, "Custom User Agent");
    assert_eq!(settings.network.connect_timeout, 30); // Default value
}

#[test]
fn test_network_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[network]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.network.https_proxy, None);
    assert_eq!(settings.network.http_proxy, None);
    assert_eq!(settings.network.all_proxy, None);
    assert_eq!(settings.network.connect_timeout, 30);
    assert_eq!(settings.network.request_timeout, 60);
    assert_eq!(settings.network.max_retries, 3);
    assert_eq!(settings.network.retry_interval, 5000);
    assert_eq!(
        settings.network.user_agent,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    );
}

#[test]
fn test_botguard_request_key_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
request_key = "CustomRequestKey"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.botguard.request_key, "CustomRequestKey");
    assert!(settings.botguard.enable_vm); // Default value
    assert_eq!(settings.botguard.vm_timeout, 30); // Default value
}

#[test]
fn test_botguard_enable_vm_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
enable_vm = false
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(!settings.botguard.enable_vm);
    assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo"); // Default value
}

#[test]
fn test_botguard_vm_timeout_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
vm_timeout = 60
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.botguard.vm_timeout, 60);
    assert!(settings.botguard.enable_vm); // Default value
}

#[test]
fn test_botguard_disable_innertube_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
disable_innertube = true
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(settings.botguard.disable_innertube);
    assert!(settings.botguard.enable_vm); // Default value
}

#[test]
fn test_botguard_challenge_endpoint_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
challenge_endpoint = "https://custom.endpoint.com/challenge"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.botguard.challenge_endpoint,
        Some("https://custom.endpoint.com/challenge".to_string())
    );
    assert!(settings.botguard.enable_vm); // Default value
}

#[test]
fn test_botguard_user_agent_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
user_agent = "Custom BotGuard User Agent"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.botguard.user_agent,
        Some("Custom BotGuard User Agent".to_string())
    );
    assert!(settings.botguard.enable_vm); // Default value
}

#[test]
fn test_botguard_disable_snapshot_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
disable_snapshot = true
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(settings.botguard.disable_snapshot);
    assert!(settings.botguard.enable_vm); // Default value
}

#[test]
fn test_botguard_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[botguard]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo");
    assert!(settings.botguard.enable_vm);
    assert_eq!(settings.botguard.vm_timeout, 30);
    assert!(!settings.botguard.disable_innertube);
    assert_eq!(settings.botguard.challenge_endpoint, None);
    assert_eq!(settings.botguard.user_agent, None);
    assert!(!settings.botguard.disable_snapshot);
}

#[test]
fn test_cache_cache_dir_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[cache]
cache_dir = "/tmp/custom_cache"
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(
        settings.cache.cache_dir,
        Some("/tmp/custom_cache".to_string())
    );
    assert!(settings.cache.enable_file_cache); // Default value
    assert_eq!(settings.cache.memory_cache_size, 100); // Default value
}

#[test]
fn test_cache_enable_file_cache_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[cache]
enable_file_cache = false
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(!settings.cache.enable_file_cache);
    assert_eq!(settings.cache.cache_dir, None); // Default value
}

#[test]
fn test_cache_memory_cache_size_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[cache]
memory_cache_size = 200
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert_eq!(settings.cache.memory_cache_size, 200);
    assert!(settings.cache.enable_file_cache); // Default value
}

#[test]
fn test_cache_enable_compression_only() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[cache]
enable_compression = true
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    assert!(settings.cache.enable_compression);
    assert!(settings.cache.enable_file_cache); // Default value
}

#[test]
fn test_cache_empty_section() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[cache]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.cache.cache_dir, None);
    assert!(settings.cache.enable_file_cache);
    assert_eq!(settings.cache.memory_cache_size, 100);
    assert!(!settings.cache.enable_compression);
}

#[test]
fn test_empty_config_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "").unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All sections and fields should use defaults
    assert_eq!(settings.server.host, "::");
    assert_eq!(settings.server.port, 4416);
    assert_eq!(settings.token.ttl_hours, 6);
    assert_eq!(settings.logging.level, "info");
    assert_eq!(settings.network.connect_timeout, 30);
    assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo");
    assert!(settings.cache.enable_file_cache);
}

#[test]
fn test_all_sections_empty() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]

[token]

[logging]

[network]

[botguard]

[cache]
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // All fields should use defaults
    assert_eq!(settings.server.host, "::");
    assert_eq!(settings.server.port, 4416);
    assert_eq!(settings.token.ttl_hours, 6);
    assert_eq!(settings.logging.level, "info");
    assert_eq!(settings.network.connect_timeout, 30);
    assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo");
    assert!(settings.cache.enable_file_cache);
}

#[test]
fn test_mixed_partial_fields() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "0.0.0.0"

[token]
ttl_hours = 12

[logging]
level = "debug"

[network]
https_proxy = "https://proxy.example.com:8080"

[botguard]
enable_vm = false

[cache]
memory_cache_size = 500
        "#
    )
    .unwrap();

    let settings = Settings::from_file(temp_file.path()).unwrap();
    // Specified fields should use provided values
    assert_eq!(settings.server.host, "0.0.0.0");
    assert_eq!(settings.token.ttl_hours, 12);
    assert_eq!(settings.logging.level, "debug");
    assert_eq!(
        settings.network.https_proxy,
        Some("https://proxy.example.com:8080".to_string())
    );
    assert!(!settings.botguard.enable_vm);
    assert_eq!(settings.cache.memory_cache_size, 500);

    // Other fields should use defaults
    assert_eq!(settings.server.port, 4416);
    assert!(settings.token.enable_cache);
    assert!(!settings.logging.verbose);
    assert_eq!(settings.network.connect_timeout, 30);
    assert_eq!(settings.botguard.request_key, "O43z0dpjhgX20SCx4KAo");
    assert!(settings.cache.enable_file_cache);
}

#[test]
fn test_integration_with_env_override() {
    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
        "#
    )
    .unwrap();

    // Save original environment state
    let original_port = std::env::var("POT_SERVER_PORT").ok();

    // Set environment variable
    unsafe {
        std::env::set_var("POT_SERVER_PORT", "9000");
    }

    let settings = Settings::from_file(temp_file.path())
        .unwrap()
        .merge_with_env()
        .unwrap();

    // Config file host should be used
    assert_eq!(settings.server.host, "127.0.0.1");
    // Environment variable port should override default
    assert_eq!(settings.server.port, 9000);
    // Other fields should use defaults
    assert_eq!(settings.server.timeout.as_secs(), 30);
    assert!(settings.server.enable_cors);

    // Restore original environment state
    unsafe {
        std::env::remove_var("POT_SERVER_PORT");
        if let Some(port) = original_port {
            std::env::set_var("POT_SERVER_PORT", port);
        }
    }
}
