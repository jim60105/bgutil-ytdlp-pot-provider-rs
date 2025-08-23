# Rustdoc Guidelines

This document establishes comprehensive standards for documenting Rust code in the project, ensuring consistency, clarity, and maintainability.

## General Principles

- **Language**: All documentation must be written in English
- **Clarity**: Use clear, concise language that avoids unnecessary jargon
- **Practicality**: Provide practical examples that demonstrate real-world usage
- **Completeness**: Document error conditions, panics, and edge cases
- **Connectivity**: Include links to related functions, types, and modules using `[`backticks`]`

## Module Documentation

Module-level documentation should use `//!` syntax and include:

```rust
//! Brief one-line summary of the module's purpose.
//!
//! Detailed explanation of the module's functionality, architecture,
//! and key concepts. Include examples of typical usage patterns.
//!
//! # Key Components
//!
//! - [`MainStruct`] - Primary structure for handling operations
//! - [`helper_function`] - Utility function for common tasks
//!
//! # Examples
//!
//! ```rust
//! use subx_cli::module_name::MainStruct;
//!
//! let instance = MainStruct::new();
//! instance.do_something()?;
//! ```
//!
//! # Architecture
//!
//! Describe the module's design patterns, relationships with other modules,
//! and any important architectural decisions.
```

## Struct and Enum Documentation

### Structs

```rust
/// Brief description of the struct's purpose.
///
/// Detailed explanation of the struct's role, typical usage,
/// and relationship to other types in the system.
///
/// # Fields
///
/// - `field_name`: Description of the field's purpose and constraints
/// - `another_field`: Description with validation rules or format
///
/// # Examples
///
/// ```rust
/// use subx_cli::SomeStruct;
///
/// let config = SomeStruct::new();
/// config.validate()?;
/// ```
///
/// # Thread Safety
///
/// Document thread safety characteristics if relevant.
pub struct MyStruct {
    /// Brief description of the field.
    /// 
    /// More detailed explanation if the field has complex behavior,
    /// validation rules, or specific format requirements.
    pub field_name: String,
}
```

### Enums

```rust
/// Brief description of the enum's purpose.
///
/// Detailed explanation of what the enum represents and when
/// each variant should be used.
///
/// # Variants
///
/// Each variant should be documented with its specific use case
/// and any associated data meanings.
///
/// # Examples
///
/// ```rust
/// use subx_cli::MyEnum;
///
/// let variant = MyEnum::VariantA("value".to_string());
/// match variant {
///     MyEnum::VariantA(data) => println!("Found: {}", data),
///     MyEnum::VariantB => println!("No data"),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum MyEnum {
    /// Description of when this variant is used.
    ///
    /// Additional details about the associated data if present.
    VariantA(String),
    
    /// Description of when this variant is used.
    VariantB,
}
```

## Function Documentation

### Public Functions

```rust
/// Brief description of what the function does.
///
/// Detailed explanation of the function's behavior, including
/// any important algorithm details or implementation notes.
///
/// # Arguments
///
/// - `param1`: Description of the parameter and its constraints
/// - `param2`: Description with expected range or format
///
/// # Returns
///
/// Description of the return value and its meaning.
/// For `Result` types, describe both success and error cases.
///
/// # Errors
///
/// This function returns an error if:
/// - Specific condition 1 occurs
/// - Specific condition 2 occurs
/// - Input validation fails
///
/// # Panics
///
/// This function panics if:
/// - Specific panic condition (avoid panics in library code)
///
/// # Examples
///
/// ```rust
/// use subx_cli::my_function;
///
/// let result = my_function("input", 42)?;
/// assert_eq!(result, "expected_output");
/// ```
///
/// # Performance
///
/// Document performance characteristics for computationally expensive functions.
///
/// # Safety
///
/// Document safety requirements for unsafe functions.
pub fn my_function(param1: &str, param2: i32) -> Result<String, Error> {
    // Implementation
}
```

### Associated Functions and Methods

```rust
impl MyStruct {
    /// Creates a new instance with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let instance = MyStruct::new();
    /// ```
    pub fn new() -> Self {
        // Implementation
    }

    /// Performs an operation on the instance.
    ///
    /// # Arguments
    ///
    /// - `input`: The data to process
    ///
    /// # Returns
    ///
    /// The processed result or an error if processing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut instance = MyStruct::new();
    /// let result = instance.process("data")?;
    /// ```
    pub fn process(&mut self, input: &str) -> Result<String, Error> {
        // Implementation
    }
}
```

## Error Documentation

Error types require special attention:

```rust
/// Comprehensive error handling for SubX operations.
///
/// This enum covers all possible error conditions with specific
/// context to facilitate debugging and user-friendly reporting.
///
/// # Error Categories
///
/// - I/O errors: File system operations
/// - Configuration errors: Invalid settings or missing values
/// - Processing errors: Format or content-related failures
///
/// # Examples
///
/// ```rust
/// use subx_cli::error::{SubXError, SubXResult};
///
/// fn example() -> SubXResult<()> {
///     Err(SubXError::Config {
///         message: "Missing required field".to_string(),
///     })
/// }
/// ```
#[derive(Error, Debug)]
pub enum SubXError {
    /// I/O operation failed during file system access.
    ///
    /// # Common Causes
    /// - Permission issues
    /// - Disk space shortage
    /// - Network file system problems
    ///
    /// # Resolution
    /// Check file permissions and available disk space.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Example Code Standards

### Quality Requirements

- **Compilation**: All examples must compile successfully
- **Relevance**: Examples should demonstrate practical usage
- **Completeness**: Include necessary imports and setup
- **Testing**: Use `cargo test --doc` to validate examples

### Example Patterns

```rust
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use subx_cli::SomeType;
///
/// let instance = SomeType::new();
/// let result = instance.process()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Advanced usage with configuration:
///
/// ```rust
/// # use subx_cli::{SomeType, Config};
/// let config = Config::builder()
///     .option("value")
///     .build();
/// let instance = SomeType::with_config(config);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

## Documentation Validation and CI Integration

### Local Development

```bash
# Generate documentation and check for warnings
cargo doc --all-features --no-deps --document-private-items

# Test documentation examples
cargo test --doc --verbose

# Check for missing documentation
cargo clippy -- -W missing_docs -D warnings
```

### CI/CD Integration

Documentation quality is enforced through our comprehensive CI/CD pipeline. The project includes an automated documentation quality check script that is integrated into the GitHub Actions workflow.

#### Automated Quality Check

The `scripts/quality_check.sh` script performs comprehensive quality checks and is automatically executed in our CI pipeline:

```yaml
# Comprehensive Code Quality Check
- name: Comprehensive Code Quality Check
  run: |
    # Make the code check script executable (if not already)
    chmod +x scripts/quality_check.sh
    
    # Run the comprehensive quality check script
    ./scripts/quality_check.sh
```

#### What the Script Checks

The `scripts/quality_check.sh` script performs the following checks:

1. **Code Compilation**: Ensures all code compiles successfully with `cargo check --all-features`
2. **Code Formatting**: Validates code formatting with `cargo fmt -- --check`
3. **Code Quality**: Runs Clippy linting with `cargo clippy --all-features -- -D warnings`
4. **Documentation Generation**: Builds documentation and checks for errors/warnings with `cargo doc`
5. **Documentation Coverage**: Checks for missing documentation on public APIs
6. **Documentation Examples**: Tests all code examples in documentation with `cargo test --doc`
7. **Unit Tests**: Runs all unit tests with `cargo test`
8. **Integration Tests**: Runs all integration tests

#### Running Documentation Checks Locally

Developers can run the same checks locally before committing:

```bash
# Run the comprehensive documentation quality check
./scripts/quality_check.sh

# Or run individual checks
cargo clippy --all-features -- -W missing_docs -D warnings
cargo doc --all-features --no-deps --document-private-items
cargo test --doc --verbose --all-features
```

## Special Considerations

### Async Functions

```rust
/// Asynchronously processes the input data.
///
/// # Arguments
///
/// - `data`: Input data to process
///
/// # Returns
///
/// A future that resolves to the processed result.
///
/// # Examples
///
/// ```rust
/// # use tokio_test;
/// # use subx_cli::async_function;
/// #[tokio::test]
/// async fn test_async() {
///     let result = async_function("input").await?;
///     assert_eq!(result, "expected");
/// #   Ok::<(), Box<dyn std::error::Error>>(())
/// }
/// ```
pub async fn async_function(data: &str) -> Result<String, Error> {
    // Implementation
}
```

### Generic Functions

```rust
/// Processes items of any type that implements `Display`.
///
/// # Type Parameters
///
/// - `T`: Must implement `Display` for string conversion
///
/// # Examples
///
/// ```rust
/// use subx_cli::process_displayable;
///
/// let result = process_displayable(&42);
/// let result2 = process_displayable(&"hello");
/// ```
pub fn process_displayable<T: std::fmt::Display>(item: &T) -> String {
    // Implementation
}
```

## Maintenance and Quality Assurance

### Code Review Checklist

- [ ] All public APIs have documentation
- [ ] Examples compile and are tested
- [ ] Error conditions are documented
- [ ] Links to related types are included
- [ ] Documentation follows the established style

### Documentation Debt Management

- Track undocumented APIs in issues
- Prioritize documentation for frequently used functions
- Update documentation when APIs change
- Regular documentation audits

This guide ensures that SubX maintains high-quality, consistent documentation that serves both contributors and users effectively.

## Documentation Quality Assurance

### Automated Quality Checks

SubX provides automated documentation quality checking through various tools and scripts:

#### Daily Development Checks

```bash
# Quick documentation quality check
./scripts/quality_check.sh

# Generate documentation with quality verification
cargo doc --all-features --no-deps --open
```

#### Continuous Integration Checks

The project includes comprehensive documentation checks in CI/CD:

- **Missing Documentation Detection**: Identifies public APIs without documentation
- **Example Code Validation**: Tests all documentation examples for compilation
- **Link Verification**: Checks for broken intra-documentation links
- **Format Consistency**: Validates documentation format standards

### Quality Metrics and Standards

#### Coverage Requirements
- **Public API Coverage**: ≥95% of public APIs must have complete documentation
- **Example Coverage**: ≥90% of complex functions should include usage examples
- **Error Documentation**: All error types must document their conditions and resolution

#### Quality Standards
- **Clarity**: Documentation must be understandable by developers unfamiliar with the codebase
- **Accuracy**: Documentation must accurately reflect current implementation
- **Completeness**: Include all necessary information for effective API usage
- **Consistency**: Follow established patterns and formatting standards

### Maintenance Workflow

#### Development Phase
1. **Documentation-First Development**: Write or update documentation before implementation
2. **Incremental Updates**: Update documentation with each code change
3. **Local Validation**: Run documentation checks before committing

#### Review Phase  
1. **Peer Review**: Include documentation review in code review process
2. **Quality Verification**: Ensure examples compile and are relevant
3. **Style Consistency**: Verify adherence to documentation standards

#### Release Phase
1. **Comprehensive Review**: Full documentation audit before releases
2. **User Testing**: Validate documentation clarity with fresh perspectives
3. **Maintenance Planning**: Schedule regular documentation maintenance

### Common Issues and Solutions

#### Missing Documentation Warnings
```rust
// ❌ Incorrect: Missing documentation
pub struct Example {
    pub field: String,
}

// ✅ Correct: Complete documentation
/// Configuration structure for example operations.
///
/// This structure holds all necessary configuration
/// parameters for performing example operations.
pub struct Example {
    /// Primary identifier for the operation
    pub field: String,
}
```

#### Broken Intra-doc Links
```rust
// ❌ Incorrect: Unresolved link
/// See [`NonexistentType`] for details

// ✅ Correct: Proper reference
/// See [`crate::error::SubXError`] for error handling details
```

#### Compilation Errors in Examples
```rust
/// # Examples
///
/// ```rust
/// // ❌ Incorrect: Missing imports and error handling
/// let result = some_function("input");
/// ```
///
/// ```rust
/// // ✅ Correct: Complete example
/// use subx_cli::some_function;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = some_function("input")?;
/// assert!(!result.is_empty());
/// # Ok(())
/// # }
/// ```
```

## Documentation Maintenance and Workflow

### Maintenance Principles

#### Synchronization Principle
- **Code and Documentation Sync**: Every API change, new feature, or behavior modification must synchronously update related documentation
- **Documentation-First Approach**: For major features, write or update design documentation before implementation
- **Version Control Integration**: Documentation changes must be committed together with code changes

#### Quality Assurance Principle
- **Completeness**: All public APIs must have complete documentation
- **Accuracy**: Documentation content must remain consistent with actual implementation
- **Practicality**: Provide concrete, executable example code
- **Readability**: Use clear, concise language avoiding excessive technical jargon

#### Consistency Principle
- **Unified Format**: Follow documentation format standards established in this guide
- **Style Consistency**: Use unified writing style and terminology
- **Structured Organization**: Adopt standardized documentation structure and organization

### Development Workflow

#### Daily Maintenance Process

**1. Pre-development Check**
```bash
# Execute code quality check
./scripts/quality_check.sh

# Check existing documentation completeness
cargo doc --all-features --no-deps
```

**2. During Development**
- **When adding APIs**: Immediately write complete documentation
- **When modifying APIs**: Synchronously update related documentation and examples
- **During refactoring**: Check and update affected documentation

**3. Pre-commit Check**
```bash
# Format code
cargo fmt

# Execute comprehensive check
cargo clippy --all-features -- -D warnings

# Test documentation examples
cargo test --doc --verbose

# Generate documentation verification
cargo doc --all-features --no-deps
```

#### Periodic Maintenance Tasks

**Weekly Maintenance (Recommended Fridays)**
```bash
# Execute comprehensive code quality check
./scripts/quality_check.sh

# Check documentation coverage
cargo clippy --all-features -- -W missing_docs | grep "missing documentation"

# Update CHANGELOG.md
# Check if new features need documentation
```

**Monthly Maintenance**
1. **Documentation Completeness Review**
   - Check if all new modules have complete documentation
   - Verify example code is still valid
   - Update outdated design documents

2. **Documentation Quality Improvement**
   - Check user feedback to improve documentation readability
   - Add examples for common usage scenarios
   - Optimize documentation structure and organization

3. **Tool and Process Improvement**
   - Update documentation check scripts
   - Improve CI/CD documentation check processes
   - Evaluate new documentation tools and best practices

### Tools and Automation

#### Primary Maintenance Tools

**1. Documentation Quality Check Script**
```bash
# Execute comprehensive code quality check
./scripts/quality_check.sh

# Check specific module documentation
cargo doc --package subx-cli --no-deps
```

**2. Documentation Generation Tools**
```bash
# Generate complete project documentation
cargo doc --all-features --no-deps --open

# Generate private item documentation (for development)
cargo doc --all-features --document-private-items --open
```

**3. Documentation Testing Tools**
```bash
# Test all documentation examples
cargo test --doc --verbose

# Test specific module documentation examples
cargo test --doc --package subx-cli --verbose
```

#### Automation Tool Configuration

**VS Code Settings**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-W", "missing_docs"],
    "files.associations": {
        "*.md": "markdown"
    }
}
```

**Git Hooks Setup**
```bash
# Setup pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "Executing code quality check..."
./scripts/quality_check.sh
EOF

chmod +x .git/hooks/pre-commit
```

### Troubleshooting Common Issues

#### Documentation Generation Errors

**Issue: Intra-doc link errors**
```
warning: unresolved link to `SomeType`
```

**Solution:**
```rust
// Use full path
/// See [`crate::module::SomeType`] for detailed explanation

// Or use relative path
/// See [`super::SomeType`] for detailed explanation
```

**Issue: Documentation example compilation failure**
```
error[E0433]: failed to resolve: use of undeclared crate or module
```

**Solution:**
```rust
/// # Examples
///
/// ```rust
/// use subx_cli::module::Type;  // Add necessary use statements
/// 
/// let instance = Type::new();
/// # Ok::<(), Box<dyn std::error::Error>>(())  // Handle errors
/// ```
```

**Issue: Missing documentation warnings**
```
warning: missing documentation for a struct field
```

**Solution:**
```rust
pub struct Example {
    /// Detailed field description
    pub field: String,
}
```

#### Documentation Example Best Practices

**1. Error Handling**
```rust
/// # Examples
/// 
/// ```rust
/// # use subx_cli::error::SubXError;
/// # fn main() -> Result<(), SubXError> {
/// let result = some_function()?;
/// # Ok(())
/// # }
/// ```
```

**2. Async Functions**
```rust
/// # Examples
///
/// ```rust
/// # tokio_test::block_on(async {
/// let result = async_function().await?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
```

**3. External Dependencies**
```rust
/// # Examples
///
/// ```rust,ignore  // Mark as ignore if external setup needed
/// // This example requires AI API key configuration
/// let client = create_ai_client("your-api-key");
/// ```
```

### CI/CD Integration

#### GitHub Actions Workflow

The project uses a comprehensive documentation quality check script integrated into CI/CD pipeline:

```yaml
# .github/workflows/build-test-audit-coverage.yml
- name: Comprehensive Code Quality Check
  run: |
    # Make the code check script executable (if not already)
    chmod +x scripts/quality_check.sh
    
    # Run the comprehensive quality check script
    ./scripts/quality_check.sh
```

This replaces individual code checks with a unified script (`scripts/quality_check.sh`) that performs:
- Code compilation and formatting validation
- Clippy linting with documentation checks
- Documentation generation and warning detection
- Documentation coverage analysis
- Documentation example testing
- Unit and integration test execution

#### Documentation Quality Metrics

**Automated Tracking via CI**
- **Build Status**: All documentation checks must pass for CI success
- **Example Test Pass Rate**: 100% of documentation examples must compile and pass tests
- **Documentation Error Count**: Zero errors allowed, warnings reported but don't fail build
- **Coverage Reporting**: Missing documentation items are identified and reported

**Target Standards Enforced**
- 📊 **Documentation Coverage**: All public APIs must have complete documentation
- ✅ **Example Testing**: All documentation examples must pass `cargo test --doc`
- ⚠️ **Documentation Warnings**: Minimized through automated checking
- 🎯 **Quality Score**: Enforced through comprehensive CI script validation

### Commit Standards

#### Documentation-related Commit Message Format

```bash
# Adding documentation
git commit -m "docs: add comprehensive documentation for core::matcher module

- Add module-level documentation with architecture overview
- Document all public APIs with examples
- Include error handling documentation
- Add usage examples for common scenarios"

# Updating documentation  
git commit -m "docs: update config module documentation

- Fix broken intra-doc links
- Update examples to match current API
- Add missing field documentation"

# Fixing documentation issues
git commit -m "docs: fix documentation warnings and broken links

- Resolve 3 broken intra-doc links in services module
- Fix documentation example compilation errors
- Update deprecated API references"
```

#### Documentation Review Checklist

**Pre-PR Submission Check**
- [ ] `./scripts/quality_check.sh` passes
- [ ] All new public APIs have complete documentation
- [ ] Documentation example code compiles and runs
- [ ] No broken intra-doc links
- [ ] Follows project documentation format standards

**Code Review Check**
- [ ] Documentation descriptions are clear and understandable
- [ ] Example code is representative
- [ ] Error conditions are appropriately documented
- [ ] Consistent with other module documentation style
- [ ] Includes appropriate cross-reference links

### Continuous Improvement

#### Documentation Quality Monitoring

**Automated Monitoring**
- Integrate documentation quality checks into CI/CD pipeline
- Set up documentation coverage change alerts
- Monitor documentation example test failure situations

**Manual Inspection**
- Regularly review documentation completeness and accuracy
- Collect user feedback on documentation quality
- Identify documentation areas needing improvement

#### Future Improvement Directions

**Short-term Goals (1-3 months)**
- [ ] Complete documentation work for all modules
- [ ] Establish automated documentation test suite
- [ ] Integrate documentation quality metrics monitoring

**Medium-term Goals (3-6 months)**  
- [ ] Build interactive documentation examples
- [ ] Integrate documentation search and navigation features
- [ ] Build automated documentation style checking tools

**Long-term Goals (6+ months)**
- [ ] Build documentation internationalization support
- [ ] Integrate AI-assisted documentation generation
- [ ] Build documentation quality scoring system

## Documentation Maintenance Guide

For comprehensive maintenance procedures, quality standards, and workflow guidelines, see the sections above.
