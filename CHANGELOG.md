# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.6.4] - 2026-02-03

### Changed

- Changed: Removed POT token validation to match TypeScript implementation, eliminating false positives for complex content bindings with protobuf data

### Fixed

- Fixed: Allow hyphen-prefixed values in content-binding argument to support YouTube video IDs starting with dash (e.g., -6OjhRWNLfk)

## [0.6.3] - 2026-01-23

### Fixed

- Fixed: Snapshot directory creation before BotGuard initialization to prevent "file I/O: No such file or directory" errors
- Fixed: V8 isolate leak warning in CLI mode by implementing proper cleanup with shutdown methods and Drop trait
- Fixed: Permission denied errors during container cache creation by restructuring directory setup in Containerfile

### Changed

- Changed: Default BotGuard snapshot path moved from user data directory to temp directory for better portability and to avoid permission issues
- Changed: Documentation updated to reflect unified CLI commands (bgutil-pot binary with server subcommand)
- Changed: Agent guidance documentation surfaced to top-level AGENTS.md for better visibility

## [0.6.2] - 2026-01-22

### Fixed

- Fixed: CLI exit code 1 failures when generating POT tokens by removing forced Innertube API calls that diverged from TypeScript reference implementation (#92, #91)
- Fixed: Token generation now directly passes content_binding to BotGuard client regardless of format, matching TypeScript behavior
- Fixed: CLI robustness improved when Innertube API is unavailable or rate-limited

### Changed

- Changed: Simplified `mint_pot_token` method to eliminate complex token type determination logic during generation
- Changed: Token type helper methods (`determine_token_type`, `is_video_id_format`, `is_visitor_data_format`) marked as unused for potential future use

### Added

- Added: 8 comprehensive CLI robustness tests covering video ID formats, visitor data formats, custom formats, and bypass_cache functionality

## [0.6.1] - 2025-12-28

### Fixed

- Fixed: POT token validation maximum length increased from 200 to 250 characters to accommodate longer BotGuard tokens (#90, #89)
- Fixed: Valid BotGuard tokens of 212 characters no longer rejected due to overly restrictive length validation

## [0.6.0] - 2025-12-01

### Added

- Added: `--config` option to server subcommand allowing users to specify configuration file path via CLI argument (#84, #83)
- Added: `BotGuardClient::reinitialize()` method for gracefully restarting BotGuard worker with fresh snapshot data (#88, #87)
- Added: Helper methods `get_botguard_expiry_as_chrono()` and `create_token_minter_entry()` for better code organization

### Fixed

- Fixed: Long-running processes now automatically reinitialize expired BotGuard snapshots instead of generating invalid POT tokens (#88, #87)
- Fixed: Logging level from configuration file is now properly respected in server mode (#86, #85)
- Fixed: Log level precedence correctly implemented: CLI `--verbose` > `RUST_LOG` env var > config file > default

### Changed

- Changed: API documentation updated to reflect actual binary structure (`bgutil-pot server` instead of `bgutil-pot-server`)
- Changed: Server mode logging initialization reordered to load configuration before setting up tracing
- Changed: Logging system migrated from `tracing_subscriber::fmt()` to registry-based approach with `EnvFilter` for flexible configuration

## [0.5.4] - 2025-11-20

### Fixed

- Fixed: Critical memory leak in BotGuard causing ~25MB growth per request by implementing persistent worker thread pattern (#82, #81)
- Fixed: V8 isolate leaks reduced from 13 per run to 1 at shutdown
- Fixed: Memory growth eliminated - reduced from 249MB for 10 requests to 0MB with stable 80MB baseline usage

### Added

- Added: Memory usage regression tests in `tests/memory_usage_test.rs` to prevent future leaks
- Added: Worker thread pattern using mpsc channel for sequential BotGuard token generation

### Changed

- Changed: BotGuard instance creation from per-request to single persistent instance maintained by dedicated worker thread
- Changed: Token generation now uses message passing instead of creating new tokio runtime per request

## [0.5.3] - 2025-11-05

### Fixed

- Fixed: Configuration parser now properly handles partial TOML configurations by making all config fields optional with `#[serde(default)]` attributes (#80, #79)
- Fixed: "missing field" errors when specifying only some fields in config file (e.g., setting `host` without `port`)

### Added

- Added: 44 comprehensive tests in `tests/config_optional_fields.rs` covering individual field omission scenarios, empty sections, and partial configurations
- Added: Documentation file `docs/config-optional-fields.md` with default values reference table and minimal configuration examples

## [0.5.2] - 2025-11-04

### Fixed

- Fixed: GitHub Actions release workflow permissions to enable write access for contents

## [0.5.1] - 2025-11-04

### Changed

- Changed: Updated CI status badge in README to reference build-test-coverage workflow
- Changed: Updated bug report URL in Python plugin to point to jim60105 GitHub repository

## [0.5.0] - 2025-11-04

### Added

- Added: BGUTIL_CONFIG environment variable support for specifying custom configuration file paths (#66)
- Added: Proper configuration precedence order: CLI arguments > environment variables > configuration file > default values
- Added: Build-provenance attestations for binary assets to enhance supply chain security verification
- Added: Support for structured Challenge data format from yt-dlp with dedicated ChallengeData type (#65)
- Added: InterpreterUrl wrapper type for Google's trusted resource URL format
- Added: Enhanced JSON error logging with detailed serde diagnostics and request body preview for debugging

### Changed

- Changed: Upgraded all dependencies to latest versions (tokio 1.43.0, serde 1.0.216, reqwest 0.12.12, and more)
- Changed: Challenge field in PotRequest now accepts both String (legacy) and structured ChallengeData formats using untagged enum
- Changed: Server CLI arguments (host/port) changed to Option type to properly detect explicit user values
- Changed: ConfigLoader now reads BGUTIL_CONFIG environment variable with fallback to default config path

### Fixed

- Fixed: HTTP 422 errors when yt-dlp sends structured Challenge data as JSON objects instead of strings (#65, #63)
- Fixed: Configuration file server.host setting being ignored when using BGUTIL_CONFIG environment variable (#64)
- Fixed: JSON deserialization errors now provide detailed error messages with request body context
- Fixed: Deprecated assert_cmd::Command::cargo_bin usage in tests

### Security

- Security: Implemented cryptographic build-provenance attestations for released binaries using GitHub Actions
- Security: Enhanced supply chain security allowing consumers to verify integrity and origin of binaries

## [0.4.0] - 2025-09-02

### Added

- Added: Container image now includes yt-dlp plugin distribution for unified deployment
- Added: Plugin files are now available at `/client/yt_dlp_plugins` path in container images

### Fixed

- Fixed: `/minter_cache` endpoint returning JSON-serialized strings instead of meaningful cache keys (#62)
- Fixed: Cache key generation now returns human-readable patterns like "default", "proxy:<http://proxy:8080>" instead of problematic format

### Changed

- Changed: Container binary path updated to `/bgutil-pot` for consistency
- Changed: Improved cache key format for better debugging experience

## [0.3.0] - 2025-09-01

### Added

- Added: Enhanced test execution with cargo nextest integration for improved performance and reporting capabilities

### Fixed

- Fixed: Test isolation issue causing CI failures in container builds by implementing static mutex synchronization for environment variable tests
- Fixed: Race conditions between parallel test execution affecting environment variables

### Changed

- Changed: Container test framework migrated from cargo test to cargo nextest for better test performance and parallel execution
- Changed: Release workflow timing improved with proper wait steps for asset upload reliability

## [0.2.0] - 2025-08-31

### Added

- Added: Unified CLI architecture with `bgutil-pot` binary supporting both server and generate modes via subcommands
- Added: Container deployment support with multi-platform builds (Linux amd64/arm64)
- Added: GitHub Actions workflow for automated container building with SLSA attestation support
- Added: Plugin packaging in GitHub Releases for unified distribution (yt-dlp plugin + Rust binaries)
- Added: Comprehensive container deployment with Docker/Podman support and SELinux compatibility
- Added: Multi-registry container publishing (Docker Hub, GitHub Container Registry, Quay.io)
- Added: Static binary builds with UPX compression for minimal container images
- Added: CLI migration guide (`docs/CLI_MIGRATION.md`) for transitioning from dual-binary system

### Changed

- Changed: Merged dual binary system (`bgutil-pot-server` + `bgutil-pot-generate`) into single `bgutil-pot` CLI tool
- Changed: CLI interface now uses subcommands: `bgutil-pot server` for server mode, `bgutil-pot` for generate mode
- Changed: Container base image migrated from Alpine to Debian bookworm-slim for better V8 compatibility
- Changed: Python plugin backend migrated from TypeScript to Rust implementation
- Changed: Plugin provider names updated from 'bgutil:script' to 'bgutil:cli' for better terminology
- Changed: Installation documentation updated to reference this project's GitHub Releases

### Fixed

- Fixed: CLI integration tests updated to use correct binary name after unification
- Fixed: Container exit code 127 resolved by using static dumb-init binary for scratch compatibility
- Fixed: Version checking tests made dynamic to prevent failures during version bumps
- Fixed: Visitor data validation to accept underscore and hyphen characters from YouTube API
- Fixed: Python plugin executable path detection and validation logic
- Fixed: Container SELinux flag compatibility for GitHub Actions environment

### Security

- Security: Implemented SLSA Level 3 build-provenance attestations for container images
- Security: Added SBOM (Software Bill of Materials) generation for supply chain transparency
- Security: Container images run as non-root user (UID 1001) with minimal scratch base

## [0.1.1] - 2025-08-31

### Fixed

- Fixed: Cargo publish compatibility by correcting exclude path pattern from 'server/' to '/server/' to specifically exclude the root-level TypeScript server directory while preserving the src/server/ Rust module
- Fixed: CI dependency audit workflow by ignoring RUSTSEC-2024-0436 vulnerability warning to prevent false positive build failures

## [0.1.0] - 2025-08-31

### Added

- Complete Rust implementation of BgUtils POT Provider for YouTube POT token generation
- HTTP server mode with comprehensive REST API endpoints:
  - `POST /get_pot` - Generate POT tokens with content binding support
  - `GET /ping` - Health check endpoint returning server uptime and version
  - `POST /invalidate_caches` - Cache invalidation endpoint
  - `POST /invalidate_it` - Integrity token invalidation endpoint
  - `GET /minter_cache` - Debug endpoint for cache inspection
- Command-line tool (`bgutil-pot-generate`) for one-time POT token generation
- HTTP server binary (`bgutil-pot-server`) for persistent service mode
- Real BotGuard integration using `rustypipe-botguard` crate for authentic token generation
- WebPoMinter functionality for complete POT token minting workflow
- Enhanced SessionManager with comprehensive token generation and caching
- Configuration management system with environment variable support:
  - TOML configuration file loading
  - Proxy configuration (HTTP_PROXY, HTTPS_PROXY, ALL_PROXY)
  - Configurable token TTL, caching, and BotGuard settings
- Comprehensive error handling framework with structured error types
- File-based caching system following XDG Base Directory Specification
- IPv6/IPv4 dual-stack server support with automatic fallback
- Complete proxy support including SOCKS4/5 and HTTP/HTTPS proxies
- Professional testing framework with 200+ tests and 87%+ code coverage
- Quality assurance tools and scripts (`scripts/quality_check.sh`, `scripts/check_coverage.sh`)
- Comprehensive documentation and API reference
- Docker container support with multi-platform builds
- Three practical usage examples (basic usage, server setup, configuration)
- TypeScript API compatibility for seamless migration

### Changed

- Migrated core implementation from TypeScript to Rust for improved performance and memory safety
- Replaced manual JavaScript VM integration with `rustypipe-botguard` crate
- Enhanced POT token generation with real BotGuard attestation instead of placeholder tokens
- Improved error handling with structured error types and better diagnostics
- Streamlined codebase removing 1500+ lines of complex manual implementations

### Fixed

- Resolved thread safety issues in BotGuard operations
- Fixed Handler trait compatibility issues in HTTP server
- Corrected token validation to support real BotGuard token formats (80-200 characters)
- Improved concurrent request handling and session management
- Enhanced JavaScript execution integration for WebPoMinter functionality

### Security

- Implemented secure proxy credential handling with password masking in logs
- Added comprehensive input validation and sanitization
- Enhanced token generation security using authentic BotGuard integration

[Unreleased]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.6.4...HEAD
[0.6.4]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.4...v0.6.0
[0.5.4]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/tag/v0.1.0
