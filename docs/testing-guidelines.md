# Testing Guidelines

## Overview

This document outlines the testing principles, practices, and guidelines for the project. Following the architectural principles, our testing approach emphasizes safety, isolation, and parallel execution while maintaining comprehensive coverage.

## Table of Contents

- [Core Testing Principles](#core-testing-principles)
- [Architecture and Design](#architecture-and-design)
- [Configuration Testing](#configuration-testing)
- [Dependency Injection in Tests](#dependency-injection-in-tests)
- [Test Organization](#test-organization)
- [Best Practices](#best-practices)
- [Common Patterns](#common-patterns)
- [Performance Guidelines](#performance-guidelines)
- [Troubleshooting](#troubleshooting)

## Core Testing Principles

### 1. Safety First
- **No `unsafe` code** in tests or production code
- **No global state mutation** - all tests must be completely isolated
- **Memory safety** - leverage Rust's ownership system fully

### 2. Complete Isolation
- **No shared state** between tests
- **No side effects** that affect other tests
- **Parallel execution** - all tests must run safely in parallel
- **Deterministic behavior** - tests must produce consistent results

### 3. Dependency Injection
- **Service injection** - inject dependencies rather than creating global instances
- **Mock-friendly design** - enable easy mocking and testing

### 4. Comprehensive Coverage
- **Unit tests** for individual components
- **Integration tests** for component interactions
- **End-to-end tests** for complete workflows
- **Error path testing** for robustness

### ⚠️ Critical Anti-Patterns to Avoid

The following practices **must never** be used in tests as they violate safety and isolation principles:

```rust
// ❌ NEVER DO THESE IN TESTS:

// Global environment variable mutation
std::env::set_var("ANY_VAR", "value");
std::env::remove_var("ANY_VAR");

// Global state modification
static mut GLOBAL_STATE: Option<Config> = None;

// Shared mutable state
static SHARED_CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::default()));

// File system pollution in global locations
std::fs::write("/tmp/global_config.toml", content);
```

**Why these are prohibited:**
- Create race conditions in parallel test execution
- Cause non-deterministic test results
- Violate test isolation principles
- Make tests depend on execution order
- Break the dependency injection architecture

## Architecture and Design

TODO

## Dependency Injection in Tests

TODO

## Test Organization

### Directory Structure

TODO

### Unit Test Location

Unit tests should be located in the same file as the code they test:

```rust
// src/config/service.rs

impl ProductionConfigService {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config_service_creation() {
        let service = ProductionConfigService::new();
        assert!(service.is_ok());
    }
}
```

### Integration Test Organization

```rust
// tests/config_integration_tests.rs

use std::sync::Arc;
use subx_cli::config::{ConfigService, TestConfigService};

mod config_loading {
    use super::*;

    #[test]
    fn test_default_configuration_loading() {
        // Test implementation...
    }
    
    #[test]
    fn test_custom_configuration_loading() {
        // Test implementation...
    }
}

mod config_validation {
    use super::*;
    
    #[test]
    fn test_configuration_validation() {
        // Test implementation...
    }
}
```

## Best Practices

### 1. Test Naming

Use descriptive test names that clearly indicate what is being tested:

```rust
#[test]
fn test_config_service_with_openai_api_key() { /* ... */ }

#[test]
fn test_match_command_with_invalid_file_paths() { /* ... */ }

#[test]
fn test_subtitle_format_conversion_srt_to_vtt() { /* ... */ }
```

### 2. Test Data Management

Use the test helper utilities for consistent test data:

```rust
#[test]
async fn test_subtitle_parsing() {
    let helper = CLITestHelper::new();
    
    // Use helper to create predictable test files
    let srt_content = helper.generate_srt_content(vec![
        ("00:00:01,000", "00:00:03,000", "First subtitle"),
        ("00:00:04,000", "00:00:06,000", "Second subtitle"),
    ]);
    
    helper.create_subtitle_file("test.srt", &srt_content).await.unwrap();
    
    // Test parsing logic...
}
```

### 3. Error Testing

Always test error conditions:

```rust
#[test]
fn test_invalid_configuration_returns_error() {
    let mut config = Config::default();
    config.ai.temperature = -1.0; // Invalid value
    
    let service = TestConfigService::new(config);
    let result = service.get_config();
    
    assert!(result.is_err());
    
    match result.unwrap_err() {
        SubXError::Config(msg) => {
            assert!(msg.contains("temperature"));
        }
        _ => panic!("Expected ConfigError"),
    }
}
```

### 4. Async Testing

Use appropriate async testing patterns:

```rust
#[tokio::test]
async fn test_async_operation() {
    let config_service = Arc::new(TestConfigService::with_defaults());
    
    let result = async_function_under_test(&*config_service).await;
    
    assert!(result.is_ok());
}
```

### 5. Resource Cleanup

Use RAII and Drop traits for automatic cleanup:

```rust
#[test]
fn test_with_temporary_resources() {
    let helper = CLITestHelper::new(); // Automatically cleaned up on drop
    
    // Test logic that uses temporary files...
    
    // No manual cleanup needed - Drop trait handles it
}
```

## Common Patterns

### Pattern 1: Configuration Service Injection

```rust
async fn test_function_with_config_injection<T: ConfigService>(
    config_service: &T
) -> Result<()> {
    let config = config_service.get_config()?;
    // Use config for test logic...
    Ok(())
}

#[tokio::test]
async fn test_with_custom_config() {
    let config_service = TestConfigService::with_ai_settings("openai", "gpt-4.1");
    let result = test_function_with_config_injection(&config_service).await;
    assert!(result.is_ok());
}
```

### Pattern 2: Test Data Builders

```rust
struct TestConfigBuilder {
    config: Config,
}

impl TestConfigBuilder {
    fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }
    
    fn with_ai_provider(mut self, provider: &str) -> Self {
        self.config.ai.provider = provider.to_string();
        self
    }
    
    fn with_api_key(mut self, key: &str) -> Self {
        self.config.ai.api_key = Some(key.to_string());
        self
    }
    
    fn build(self) -> TestConfigService {
        TestConfigService::new(self.config)
    }
}

#[test]
fn test_with_builder_pattern() {
    let config_service = TestConfigBuilder::new()
        .with_ai_provider("openai")
        .with_api_key("sk-test-key")
        .build();
    
    let config = config_service.get_config().unwrap();
    assert_eq!(config.ai.provider, "openai");
}
```

### Pattern 3: Parameterized Testing

```rust
#[test]
fn test_multiple_ai_providers() {
    let test_cases = vec![
        ("openai", "gpt-4.1"),
        ("anthropic", "claude-3"),
    ];
    
    for (provider, model) in test_cases {
        let config_service = TestConfigService::with_ai_settings(provider, model);
        let config = config_service.get_config().unwrap();
        
        assert_eq!(config.ai.provider, provider);
        assert_eq!(config.ai.model, model);
    }
}
```

## Performance Guidelines

### 1. Parallel Test Execution

All tests must be designed for parallel execution following the principles in [Critical Anti-Patterns](#️-critical-anti-patterns-to-avoid):

```rust
// ✅ Safe for parallel execution - uses isolated configuration
#[test]
fn test_isolated_configuration() {
    let config_service = TestConfigService::with_defaults();
    // Test logic using only injected dependencies
}
```

### 2. Resource Efficiency

Minimize resource usage in tests:

```rust
#[test]
fn test_efficient_resource_usage() {
    // Use minimal configuration
    let config_service = TestConfigService::with_defaults();
    
    // Avoid creating unnecessary large test data
    let small_test_data = "minimal test content";
    
    // Use appropriate data structures
    let result = process_small_data(small_test_data, &config_service);
    assert!(result.is_ok());
}
```

### 3. Test Execution Time

Keep individual tests fast:

```rust
#[test]
fn test_fast_operation() {
    let start = std::time::Instant::now();
    
    // Test logic should complete quickly
    let config_service = TestConfigService::with_defaults();
    let result = fast_operation(&config_service);
    
    assert!(result.is_ok());
    assert!(start.elapsed() < std::time::Duration::from_millis(100));
}
```

## Troubleshooting

### Common Issues and Solutions

#### Issue: Test Failures in Parallel Execution

**Symptom**: Tests pass individually but fail when run in parallel.

**Root Cause**: Violation of [Critical Anti-Patterns](#️-critical-anti-patterns-to-avoid) - likely global state mutation.

**Solution**: Use dependency injection with isolated configuration:

```rust
// ✅ Correct approach - isolated state
#[test]
fn test_with_isolated_config() {
    let mut config = Config::default();
    config.some_value = "test".to_string();
    let config_service = TestConfigService::new(config);
    // Test logic...
}
```

#### Issue: Inconsistent Test Results

**Symptom**: Tests produce different results on different runs.

**Solution**: Eliminate non-deterministic behavior:

```rust
// Problem: Non-deterministic behavior
#[test]
fn problematic_test() {
    let random_value = rand::random::<u32>(); // ❌ Non-deterministic
    // Test logic using random_value...
}

// Solution: Use fixed test data
#[test]
fn fixed_test() {
    let fixed_value = 12345u32; // ✅ Deterministic
    // Test logic using fixed_value...
}
```

#### Issue: Configuration Loading Errors

**Symptom**: Configuration-related tests fail with loading errors.

**Root Cause**: Attempting to use production configuration services in tests.

**Solution**: Use `TestConfigService` for all test scenarios:

```rust
// ✅ Correct approach - test configuration service
#[test]
fn test_with_proper_config() {
    let config_service = TestConfigService::with_defaults();
    let config = config_service.get_config().unwrap();
    // Test logic...
}
```

### Debugging Test Issues

1. **Run tests individually**: `cargo nextest run test_name`
2. **Enable debug logging**: `RUST_LOG=debug cargo nextest run`
3. **Check for anti-patterns**: Review [Critical Anti-Patterns](#️-critical-anti-patterns-to-avoid) section
4. **Verify isolation**: Ensure tests don't create shared files or state
5. **Test in parallel**: `cargo nextest run --test-threads=8` to catch race conditions

### Performance Debugging

```rust
#[test]
fn test_with_performance_monitoring() {
    let start = std::time::Instant::now();
    
    let config_service = TestConfigService::with_defaults();
    
    let config_load_time = start.elapsed();
    println!("Config loading took: {:?}", config_load_time);
    
    let operation_start = std::time::Instant::now();
    let result = test_operation(&config_service);
    let operation_time = operation_start.elapsed();
    
    println!("Operation took: {:?}", operation_time);
    
    assert!(result.is_ok());
    assert!(operation_time < std::time::Duration::from_millis(500));
}
```

## JUnit XML Test Results

### Configuration
The project uses nextest to generate JUnit XML format test results, primarily for CI environments:

```bash
# CI profile automatically generates JUnit XML
cargo nextest run --profile ci
```

### Output Location
- JUnit XML file: `target/nextest/ci/junit.xml`
- Contains detailed output and stack traces for all tests (successful and failed)

### Codecov Test Analysis
Test results are automatically uploaded to Codecov, providing:
- Test execution time analysis
- Failure rate statistics
- Flaky test identification

## Conclusion

Following these guidelines ensures that maintains high code quality, safety, and testability. The dependency injection architecture enables comprehensive testing without sacrificing safety or performance.

### Key Takeaways

1. **Follow the [Critical Anti-Patterns](#️-critical-anti-patterns-to-avoid)** guidelines religiously
2. **Design for parallel execution** from the start
3. **Use dependency injection** throughout the testing architecture
4. **Maintain comprehensive coverage** with isolated, fast tests

### Resources

- [Technical Architecture Documentation](./tech-architecture.md)
- [Rustdoc Guidelines](./rustdoc-guidelines.md)

For questions or improvements to these guidelines, please refer to the project's issue tracker or discuss with the development team.
