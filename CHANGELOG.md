# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/tag/v0.1.0
