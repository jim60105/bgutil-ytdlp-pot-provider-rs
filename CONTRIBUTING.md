# Contributing to BgUtils POT Provider (Rust)

Welcome to the BgUtils POT Provider Rust implementation! This document provides guidelines for contributing to the project.

## Prerequisites

Before contributing, ensure you have the following installed:

- **Rust**: Version 1.85+ with edition 2024 support
- **Git**: For version control
- **System dependencies**:
  - `jq` and `bc` (for coverage checking)
  - Standard development tools for your platform

### Installing Rust

If you don't have Rust installed, visit [rustup.rs](https://rustup.rs/) for installation instructions.

```bash
# Verify Rust installation
rustc --version
cargo --version
```

### Installing Additional Tools

```bash
# Install development tools
cargo install cargo-nextest --locked  # Fast test runner
cargo install cargo-llvm-cov          # Code coverage

# Install system dependencies (Ubuntu/Debian)
sudo apt install jq bc

# Install system dependencies (macOS)
brew install jq bc
```

## Development Setup

### 1. Plugin Installation (Development)

This script will automatically copy the current working directory's plugin to one of the default yt-dlp plugin directory at `~/yt-dlp-plugins`. Use it for quick development setup:

```bash
bash scripts/install_plugin_dev.sh
```

### 2. Building the Project

```bash
# Development build
cargo build

# Release build
cargo build --release

# Build binary
cargo build --bin bgutil-pot
```

### 3. Running the Application

```bash
# Run HTTP server
cargo run --bin bgutil-pot -- server

# Generate POT token (script mode)
cargo run --bin bgutil-pot -- --content-binding "VIDEO_ID"
```

## Code Quality Standards

This project maintains high code quality standards through automated tooling and conventions.

### Formatting

Code formatting is enforced using `rustfmt` with project-specific configuration in `rustfmt.toml`:

```bash
# Format all code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Linting

We use `clippy` for code linting with strict warning levels:

```bash
# Run clippy checks
cargo clippy --all-features -- -D warnings

# Run clippy for all targets
cargo clippy --all-targets --all-features -- -D warnings
```

### Quality Assurance Script

Use our comprehensive quality check script before submitting code:

```bash
# Run all quality checks
./scripts/quality_check.sh

# Run with verbose output
./scripts/quality_check.sh --verbose

# Run with full test profile (longer timeouts)
./scripts/quality_check.sh --full
```

This script performs:

- Compilation checks
- Code formatting validation
- Clippy linting
- Documentation generation
- Documentation example testing
- Unit and integration tests

## Testing

### Running Tests

```bash
# Run all tests (fast)
cargo nextest run

# Run all tests (with full profile for longer timeouts)
cargo nextest run --profile full

# Run specific test
cargo nextest run -E 'test(test_name)'

# Run tests for specific package
cargo nextest run -p bgutil-ytdlp-pot-provider
```

### Test Categories

- **Unit tests**: Located alongside source code
- **Integration tests**: Located in `tests/` directory
- **Documentation tests**: Embedded in documentation comments

### Coverage Checking

Monitor test coverage using our coverage script:

```bash
# Check coverage with default threshold (75%)
./scripts/check_coverage.sh

# Check with custom threshold
./scripts/check_coverage.sh --threshold 80

# Show coverage table for all files
./scripts/check_coverage.sh --table

# Generate LCOV output
./scripts/check_coverage.sh --lcov coverage.info
```

## Documentation

### Code Documentation

All public APIs must be documented using rustdoc comments:

```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `param` - Description of the parameter
///
/// # Returns
///
/// Description of the return value
///
/// # Examples
///
/// ```
/// let result = function_name(param);
/// assert_eq!(result, expected_value);
/// ```
pub fn function_name(param: Type) -> ReturnType {
    // Implementation
}
```

### Documentation Generation

```bash
# Generate documentation
cargo doc --all-features --no-deps --document-private-items

# Generate and open documentation
cargo doc --all-features --no-deps --document-private-items --open

# Test documentation examples
cargo test --doc --all-features
```

## Project Structure

### Key Directories

- `src/`: Main Rust source code
  - `main.rs`: Unified CLI entry point with server and generate subcommands
  - `config/`: Configuration management
  - `server/`: HTTP server implementation
  - `session/`: POT token generation logic
  - `types/`: Type definitions
  - `utils/`: Utility functions
- `tests/`: Integration tests
- `scripts/`: Development and automation scripts
- `plugin/`: Python yt-dlp plugin (read-only)
- `docs/`: Project documentation

### Code Organization Principles

- **Modularity**: Each module has a single, well-defined responsibility
- **Error Handling**: Use `Result<T, E>` and proper error types
- **Async/Await**: Use `tokio` for async operations
- **Configuration**: Support multiple configuration sources (CLI, env, file)
- **Testing**: Write comprehensive tests for all functionality

## Commit Guidelines

### Commit Message Format

Follow conventional commit format:

```text
type(scope): brief description

Detailed description if needed

Closes #issue_number
```

**Types:**

- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test additions or modifications
- `chore`: Maintenance tasks

**Examples:**

```text
feat(server): add health check endpoint

Add /health endpoint for monitoring server status

Closes #123
```

```text
fix(session): resolve token expiration handling

Improve token refresh logic to handle edge cases
where tokens expire during generation

Closes #456
```

### Committing Code

Always ensure your code passes all quality checks before committing:

```bash
# Run quality checks
./scripts/quality_check.sh

# Stage changes
git add .

# Commit with proper message
git commit -m "feat(component): description"
```

## Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch from `master`
3. **Implement** your changes following the guidelines above
4. **Run** all quality checks: `./scripts/quality_check.sh`
5. **Write** tests for new functionality
6. **Update** documentation if needed
7. **Submit** a pull request with a clear description

### Pull Request Requirements

- All quality checks must pass
- Test coverage should not decrease
- Documentation must be updated for new features
- Commit messages follow conventional format
- No merge conflicts with master branch

## Development Tips

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Set custom server configuration
export POT_SERVER_HOST="127.0.0.1"
export POT_SERVER_PORT="8080"
```

### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run --bin bgutil-pot -- server

# Build with debug info
cargo build --bin bgutil-pot
```

### Performance Profiling

```bash
# Build with release optimizations
cargo build --release

# Profile with criterion (if benchmarks exist)
cargo bench
```

## Getting Help

- **Issues**: Check existing [GitHub issues](https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/issues)
- **Documentation**: See the `docs/` directory
- **Code Examples**: Check the `examples/` directory

## License

By contributing to this project, you agree that your contributions will be licensed under the GPL-3.0-or-later License.
