//! Integration tests for command-line interface
//!
//! Tests the CLI behavior and ensures compatibility with TypeScript version.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("content-binding"))
        .stdout(predicate::str::contains("proxy"))
        .stdout(predicate::str::contains("bypass-cache"));
}

#[test]
fn test_deprecated_visitor_data_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&["--visitor-data", "deprecated_value"]);
    
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn test_deprecated_data_sync_id_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&["--data-sync-id", "deprecated_value"]);
    
    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn test_basic_token_generation() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&["--content-binding", "test_video_id"]);
    
    // Should succeed and output JSON
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}

#[test]
fn test_bypass_cache_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&[
        "--content-binding", "test_video_id",
        "--bypass-cache"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}

#[test]
fn test_proxy_configuration() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&[
        "--content-binding", "test_video_id",
        "--proxy", "http://localhost:8080"
    ]);
    
    // May fail due to proxy not being available, but should handle gracefully
    // At minimum, should not panic
    let output = cmd.output().unwrap();
    
    // Either succeeds with valid JSON or fails with error message
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("{"));
    } else {
        // Should output empty JSON on error
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "{}");
    }
}

#[test]
fn test_cache_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("test_cache");
    
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.env("XDG_CACHE_HOME", cache_dir.to_str().unwrap());
    cmd.args(&["--content-binding", "test_video_id"]);
    
    cmd.assert()
        .success();
    
    // Cache directory should be created
    assert!(cache_dir.join("bgutil-ytdlp-pot-provider").exists());
}

#[test]
fn test_verbose_logging() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&[
        "--content-binding", "test_video_id",
        "--verbose"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}

#[test]
fn test_json_output_format() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&["--content-binding", "test_video_id"]);
    
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    // Check required fields
    assert!(json.get("poToken").is_some());
    assert!(json.get("contentBinding").is_some());
    assert!(json.get("expiresAt").is_some());
    
    // Check content binding value
    assert_eq!(json["contentBinding"], "test_video_id");
}

#[test]
fn test_source_address_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&[
        "--content-binding", "test_video_id",
        "--source-address", "192.168.1.100"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}

#[test]
fn test_disable_tls_verification_flag() {
    let mut cmd = Command::cargo_bin("bgutil-pot-generate").unwrap();
    cmd.args(&[
        "--content-binding", "test_video_id",
        "--disable-tls-verification"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*\}"#).unwrap());
}