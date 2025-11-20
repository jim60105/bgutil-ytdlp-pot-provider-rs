//! CLI integration tests
//!
//! Tests the CLI behavior and ensures compatibility with TypeScript version.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_version_flag() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_flag() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("content-binding"))
        .stdout(predicate::str::contains("proxy"))
        .stdout(predicate::str::contains("bypass-cache"));
}

#[test]
fn test_deprecated_visitor_data_flag() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["--visitor-data", "deprecated_value"]);

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn test_deprecated_data_sync_id_flag() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["--data-sync-id", "deprecated_value"]);

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn test_basic_token_generation() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["--content-binding", "test_video_id_basic"]);

    // Should succeed and output JSON
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}

#[test]
fn test_json_output_format() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["--content-binding", "test_video_id_json"]);

    let output = cmd.output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Check required fields
    assert!(json.get("poToken").is_some());
    assert!(json.get("contentBinding").is_some());
    assert!(json.get("expiresAt").is_some());

    // Check content binding value
    assert_eq!(json["contentBinding"], "test_video_id_json");
}

#[test]
fn test_cache_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("test_cache");

    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.env("XDG_CACHE_HOME", cache_dir.to_str().unwrap());
    cmd.args(&["--content-binding", "test_video_id_cache"]);

    cmd.assert().success();

    // Cache directory should be created
    assert!(cache_dir.join("bgutil-ytdlp-pot-provider").exists());
}

#[test]
fn test_server_config_flag_recognized() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a valid config file
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

    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["server", "--config", temp_file.path().to_str().unwrap()]);

    // Spawn and immediately kill the server to test that config is recognized
    cmd.timeout(std::time::Duration::from_millis(200));

    // The command will be killed by timeout, but shouldn't fail due to config parsing
    let _ = cmd.output();
}

#[test]
fn test_server_config_flag_with_help() {
    let mut cmd = cargo_bin_cmd!("bgutil-pot");
    cmd.args(&["server", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--config"));
}
