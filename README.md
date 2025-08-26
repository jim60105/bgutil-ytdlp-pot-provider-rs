# BgUtils POT Provider (Rust Implementation)

A high-performance YouTube POT (Proof-of-Origin Token) provider implemented in Rust, designed to help yt-dlp bypass the "Sign in to confirm you're not a bot" restrictions with improved performance and reliability.

> [!CAUTION]
> Providing a POT token does not guarantee bypassing 403 errors or bot checks, but it _may_ help your traffic seem more legitimate.

[![GitHub Release](https://img.shields.io/github/v/release/jim60105/bgutil-ytdlp-pot-provider-rs?style=for-the-badge)](https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases)
[![CI Status](https://img.shields.io/github/actions/workflow/status/jim60105/bgutil-ytdlp-pot-provider-rs/build-test-audit-coverage.yml?branch=master&label=Tests&style=for-the-badge)](https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/actions/workflows/build-test-audit-coverage.yml)
[![Code Coverage](https://img.shields.io/codecov/c/github/jim60105/bgutil-ytdlp-pot-provider-rs?style=for-the-badge)](https://codecov.io/gh/jim60105/bgutil-ytdlp-pot-provider-rs)
[![Crates.io](https://img.shields.io/crates/v/bgutil-ytdlp-pot-provider?style=for-the-badge)](https://crates.io/crates/bgutil-ytdlp-pot-provider)

This Rust implementation uses [LuanRT's BotGuard interfacing library](https://github.com/LuanRT/BgUtils) to generate POT tokens, helping bypass YouTube's bot detection when using yt-dlp from flagged IP addresses. See the _[PO Token Guide](https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide)_ for technical details.

## Why Rust?

This Rust rewrite offers significant improvements over the original TypeScript version:

- **🚀 Performance**: Sub-second token generation with optimized caching
- **💾 Memory Efficiency**: Lower memory footprint and better resource management  
- **🔒 Reliability**: Memory safety and robust error handling
- **📦 Easy Deployment**: Single binary with no runtime dependencies
- **🌐 Cross-Platform**: Native support for Linux, Windows, and macOS

## Architecture Overview

The system consists of two main components working together:

```
yt-dlp
  ↓ (via POT plugin)
Python Plugin (read-only)
  ↓ HTTP API calls
Rust POT Provider
  ↓ BotGuard integration
YouTube BotGuard API
  ↓ returns POT Token
yt-dlp (bypasses bot check)
```

### Core Components

1. **Rust POT Provider** (this project): Two operation modes:
   - **HTTP Server Mode** (`bgutil-pot-server`): Always-running REST API service (recommended)
   - **Script Mode** (`bgutil-pot-generate`): Per-request command-line execution
   
2. **Python Plugin** (inherited from TypeScript version): Integrates with yt-dlp's POT framework to automatically fetch tokens from the provider.

## Installation

### Prerequisites

1. **yt-dlp**: Version `2025.05.22` or above
2. **System Requirements**: 
   - Linux (x86_64), Windows (x86_64), or macOS (Intel/Apple Silicon)
   - 512MB available memory
   - Stable internet connection

### Step 1: Install the Rust POT Provider

Choose one of the following installation methods:

#### Option A: Pre-compiled Binaries (Recommended)

Download the appropriate binary from [Releases](https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest):

```bash
# Linux x86_64
wget https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest/download/bgutil-ytdlp-pot-provider-rs-linux-x86_64
chmod +x bgutil-ytdlp-pot-provider-rs-linux-x86_64

# macOS Intel
wget https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest/download/bgutil-ytdlp-pot-provider-rs-macos-x86_64
chmod +x bgutil-ytdlp-pot-provider-rs-macos-x86_64

# macOS Apple Silicon  
wget https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest/download/bgutil-ytdlp-pot-provider-rs-macos-aarch64
chmod +x bgutil-ytdlp-pot-provider-rs-macos-aarch64

# Windows (download .exe file)
# https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest/download/bgutil-ytdlp-pot-provider-rs-windows-x86_64.exe
```

#### Option B: Install from crates.io

```bash
cargo install bgutil-ytdlp-pot-provider
```

#### Option C: Build from Source

Requirements: Rust 1.85+ (edition 2024) and Cargo

```bash
git clone https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs.git
cd bgutil-ytdlp-pot-provider-rs
cargo build --release

# Binaries will be available at:
# target/release/bgutil-pot-server    (HTTP server mode)
# target/release/bgutil-pot-generate  (script mode)
```

### Step 2: Install the yt-dlp Plugin

#### Option A: PyPI Installation

If yt-dlp is installed via `pip` or `pipx`:

```bash
python3 -m pip install -U bgutil-ytdlp-pot-provider
```

#### Option B: Manual Installation

1. Download the latest plugin zip from [original project releases](https://github.com/Brainicism/bgutil-ytdlp-pot-provider/releases)
2. Extract to one of the [yt-dlp plugin directories](https://github.com/yt-dlp/yt-dlp#installing-plugins)
## Usage

### HTTP Server Mode (Recommended)

The HTTP server mode provides the best performance and user experience.

#### 1. Start the POT Provider Server

```bash
# Using default settings (binds to 127.0.0.1:4416)
./bgutil-pot-server

# Custom port
./bgutil-pot-server --port 8080

# Custom bind address (e.g., to accept connections from other machines)
./bgutil-pot-server --bind 0.0.0.0 --port 4416

# With verbose logging
./bgutil-pot-server --verbose
```

**Server Command Line Options:**
- `--bind <ADDRESS>`: Bind address (default: 127.0.0.1)
- `--port <PORT>`: Listen port (default: 4416)
- `--config <FILE>`: Configuration file path
- `--log-level <LEVEL>`: Log level (error, warn, info, debug, trace)
- `--verbose`: Enable verbose logging

#### 2. Use with yt-dlp

Once the server is running, yt-dlp will automatically detect and use it:

```bash
# Standard usage - works automatically with default settings
yt-dlp "https://www.youtube.com/watch?v=VIDEO_ID"

# If using a custom port, specify the base URL
yt-dlp --extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:8080" "VIDEO_URL"

# If tokens stop working, try legacy mode
yt-dlp --extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:4416;disable_innertube=1" "VIDEO_URL"
```

### Script Mode

For occasional use or environments where running a persistent service is not desired:

#### 1. Generate POT Token Manually

```bash
# Generate token for a specific video
./bgutil-pot-generate --content-binding "VIDEO_ID"

# With proxy support
./bgutil-pot-generate --content-binding "VIDEO_ID" --proxy "http://proxy.example.com:8080"

# Bypass cache to force new token generation
./bgutil-pot-generate --content-binding "VIDEO_ID" --bypass-cache

# JSON output format
./bgutil-pot-generate --content-binding "VIDEO_ID" --output json
```

#### 2. Use with yt-dlp

```bash
# Specify the script path for yt-dlp integration
yt-dlp --extractor-args "youtubepot-bgutilscript:script_path=/path/to/bgutil-pot-generate" "VIDEO_URL"
```

### Configuration

Both modes support configuration via:
1. Command line arguments (highest priority)
2. Environment variables
3. Configuration file (`~/.config/bgutil-pot-provider/config.toml`)
4. Default values (lowest priority)

**Example configuration file:**
```toml
[server]
bind = "127.0.0.1"
port = 4416

[logging]
level = "info"

[cache]
ttl_hours = 6
max_entries = 1000

[network]
connect_timeout = 30
request_timeout = 60
```

### Proxy Support

Both modes support proxy configuration:

```bash
# HTTP/HTTPS proxy
--proxy "http://proxy.example.com:8080"

# SOCKS5 proxy
--proxy "socks5://proxy.example.com:1080"

# Proxy with authentication
--proxy "http://user:pass@proxy.example.com:8080"
```

### Verification

To verify the plugin installation, check yt-dlp's verbose output:

```bash
yt-dlp -v "https://www.youtube.com/watch?v=VIDEO_ID"
```

You should see output similar to:
```
[debug] [youtube] [pot] PO Token Providers: bgutil:http-1.2.2 (external), bgutil:script-1.2.2 (external)
```

## Troubleshooting

### Common Issues

#### POT tokens not working
If tokens stop working, try the following in order:

1. **Restart the provider**: Stop and restart the HTTP server or regenerate tokens with `--bypass-cache`
2. **Check your IP**: Your IP might be flagged. Try using a different network or proxy
3. **Legacy mode**: Add `disable_innertube=1` to extractor arguments
4. **Update software**: Ensure you're using the latest versions of both this provider and yt-dlp

#### Connection issues
```bash
# Check if the server is running (HTTP mode)
curl http://127.0.0.1:4416/ping

# Test with verbose logging
./bgutil-pot-server --verbose

# Test script mode
./bgutil-pot-generate --content-binding "test" --verbose
```

#### Plugin not detected
Verify the plugin installation:
```bash
yt-dlp -v "https://www.youtube.com/watch?v=dQw4w9WgXcQ" 2>&1 | grep -i "pot"
```

Should show: `[debug] [youtube] [pot] PO Token Providers: bgutil:http-...`

### Performance Tips

- **Use HTTP server mode** for better performance and resource usage
- **Configure appropriate cache TTL** (default 6 hours) based on your usage patterns  
- **Use proxy rotation** if making many requests from the same IP
- **Monitor memory usage** - the server typically uses <50MB RAM

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `BGUTIL_CONFIG` | Config file path | `~/.config/bgutil-pot-provider/config.toml` |
| `TOKEN_TTL` | Token cache TTL (hours) | `6` |

## Contributing

This project welcomes contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs.git
cd bgutil-ytdlp-pot-provider-rs

# Install development dependencies
cargo build

# Run tests
cargo nextest run

# Run quality checks
./scripts/quality_check.sh
```

## License

This project is licensed under the GPL-3.0-or-later License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [LuanRT](https://github.com/LuanRT) for the [BgUtils library](https://github.com/LuanRT/BgUtils)
- [Brainicism](https://github.com/Brainicism) for the [original TypeScript implementation](https://github.com/Brainicism/bgutil-ytdlp-pot-provider)
- The [yt-dlp team](https://github.com/yt-dlp/yt-dlp) for the excellent POT provider framework
