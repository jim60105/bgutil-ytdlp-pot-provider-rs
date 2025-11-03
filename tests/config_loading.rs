//! Configuration loading integration tests
//!
//! Tests the BGUTIL_CONFIG environment variable support and proper configuration precedence

use std::io::Write;
use std::sync::Mutex;
use tempfile::NamedTempFile;

// Static mutex to ensure environment variable tests don't interfere with each other
static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_bgutil_config_env_var_loading() {
    use bgutil_ytdlp_pot_provider::config::ConfigLoader;

    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    // Create a temporary config file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
port = 9999

[token]
ttl_hours = 24
        "#
    )
    .unwrap();
    temp_file.flush().unwrap();

    // Save original environment state
    let original_config = std::env::var("BGUTIL_CONFIG").ok();

    // Set BGUTIL_CONFIG environment variable
    unsafe {
        std::env::set_var("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
    }

    // Load configuration - should read from BGUTIL_CONFIG
    let loader = ConfigLoader::new();
    let config_path = ConfigLoader::get_config_path();

    // Config path should come from BGUTIL_CONFIG
    assert!(config_path.is_some());
    assert_eq!(
        config_path.as_ref().unwrap().to_str().unwrap(),
        temp_file.path().to_str().unwrap()
    );

    // Load the settings
    let settings = loader.load(config_path.as_deref()).unwrap();

    // Verify settings were loaded from the config file
    assert_eq!(settings.server.host, "127.0.0.1");
    assert_eq!(settings.server.port, 9999);
    assert_eq!(settings.token.ttl_hours, 24);

    // Restore original environment state
    unsafe {
        std::env::remove_var("BGUTIL_CONFIG");
        if let Some(config) = original_config {
            std::env::set_var("BGUTIL_CONFIG", config);
        }
    }
}

#[test]
fn test_env_var_overrides_config_file() {
    use bgutil_ytdlp_pot_provider::config::ConfigLoader;

    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    // Create a config file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
port = 9999

[token]
ttl_hours = 24
        "#
    )
    .unwrap();
    temp_file.flush().unwrap();

    // Save original environment state
    let original_config = std::env::var("BGUTIL_CONFIG").ok();
    let original_host = std::env::var("POT_SERVER_HOST").ok();
    let original_port = std::env::var("POT_SERVER_PORT").ok();

    // Set environment variables - these should override config file
    unsafe {
        std::env::set_var("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
        std::env::set_var("POT_SERVER_HOST", "0.0.0.0");
        std::env::set_var("POT_SERVER_PORT", "8888");
    }

    // Load configuration
    let loader = ConfigLoader::new();
    let config_path = ConfigLoader::get_config_path();
    let settings = loader.load(config_path.as_deref()).unwrap();

    // Environment variables should override config file values
    assert_eq!(settings.server.host, "0.0.0.0");
    assert_eq!(settings.server.port, 8888);
    // Token TTL should still come from config file
    assert_eq!(settings.token.ttl_hours, 24);

    // Restore original environment state
    unsafe {
        std::env::remove_var("BGUTIL_CONFIG");
        std::env::remove_var("POT_SERVER_HOST");
        std::env::remove_var("POT_SERVER_PORT");

        if let Some(config) = original_config {
            std::env::set_var("BGUTIL_CONFIG", config);
        }
        if let Some(host) = original_host {
            std::env::set_var("POT_SERVER_HOST", host);
        }
        if let Some(port) = original_port {
            std::env::set_var("POT_SERVER_PORT", port);
        }
    }
}

#[test]
fn test_default_config_path() {
    use bgutil_ytdlp_pot_provider::config::ConfigLoader;

    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    // Save and clear BGUTIL_CONFIG
    let original_config = std::env::var("BGUTIL_CONFIG").ok();
    unsafe {
        std::env::remove_var("BGUTIL_CONFIG");
    }

    // Without BGUTIL_CONFIG, should return default path or None
    let config_path = ConfigLoader::get_config_path();

    // Should be either None or default path
    if let Some(path) = config_path {
        // Default path should be in user's config directory
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("bgutil-pot-provider") || path_str.contains(".config"));
    }

    // Restore original environment state
    if let Some(config) = original_config {
        unsafe {
            std::env::set_var("BGUTIL_CONFIG", config);
        }
    }
}

#[cfg(unix)]
#[test]
fn test_bgutil_config_with_server_cli() {
    use assert_cmd::Command;

    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    // Create a config file with specific host
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
port = 4416
        "#
    )
    .unwrap();
    temp_file.flush().unwrap();

    // Test server command with BGUTIL_CONFIG
    let mut cmd = Command::cargo_bin("bgutil-pot").unwrap();
    cmd.env("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
    cmd.args(&["server", "-v"]); // verbose mode to see the address in logs
    cmd.timeout(std::time::Duration::from_secs(2));

    // The server should try to bind to 127.0.0.1 from config file
    // We can't test actual binding without a running server, but we can verify
    // it's using the config by checking the output
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should see 127.0.0.1 in the debug output
    assert!(
        stderr.contains("127.0.0.1")
            || stderr.contains("Parsed address: 127.0.0.1")
            || stdout.contains("127.0.0.1"),
        "Expected to see 127.0.0.1 in server output, but got:\nSTDOUT: {}\nSTDERR: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_cli_args_override_everything() {
    use assert_cmd::Command;

    let _lock = ENV_TEST_MUTEX.lock().unwrap();

    // Create a config file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
[server]
host = "127.0.0.1"
port = 9999
        "#
    )
    .unwrap();
    temp_file.flush().unwrap();

    // Save original environment state
    let original_config = std::env::var("BGUTIL_CONFIG").ok();
    let original_host = std::env::var("POT_SERVER_HOST").ok();

    // Test with config file and env var, but CLI should win
    let mut cmd = Command::cargo_bin("bgutil-pot").unwrap();
    cmd.env("BGUTIL_CONFIG", temp_file.path().to_str().unwrap());
    unsafe {
        std::env::set_var("POT_SERVER_HOST", "0.0.0.0");
    }
    cmd.args(&["server", "--host", "::1", "--port", "7777", "-v"]);
    cmd.timeout(std::time::Duration::from_secs(2));

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // CLI args should override both config file and env vars
    // Should see ::1 and 7777 in the output
    assert!(
        (stderr.contains("::1") || stdout.contains("::1"))
            && (stderr.contains("7777") || stdout.contains("7777")),
        "Expected to see CLI args ::1 and 7777 in output, but got:\nSTDOUT: {}\nSTDERR: {}",
        stdout,
        stderr
    );

    // Restore original environment state
    unsafe {
        std::env::remove_var("POT_SERVER_HOST");
        if let Some(config) = original_config {
            std::env::set_var("BGUTIL_CONFIG", config);
        }
        if let Some(host) = original_host {
            std::env::set_var("POT_SERVER_HOST", host);
        }
    }
}
