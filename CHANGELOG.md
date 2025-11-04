# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/tag/v0.1.0
