# Configuration Optional Fields

This document describes which configuration fields are optional and what their default values are when not specified in the TOML configuration file.

## Overview

All configuration fields in the TOML configuration file are **optional**. When a field is not specified, the application will use a sensible default value. This means you can create minimal configuration files that only specify the values you want to override.

## Configuration Sections

### `[server]` - HTTP Server Configuration

All fields in the `[server]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `host` | string | `"::"` | Server host address to bind to |
| `port` | u16 | `4416` | Server port to listen on |
| `timeout` | u64 | `30` | Request timeout in seconds |
| `enable_cors` | bool | `true` | Enable CORS support |
| `max_body_size` | usize | `1048576` (1 MB) | Maximum request body size in bytes |

**Example:**
```toml
[server]
host = "127.0.0.1"
# All other fields will use default values
```

### `[token]` - Token Generation and Caching Configuration

All fields in the `[token]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `ttl_hours` | u64 | `6` | Token time-to-live in hours |
| `enable_cache` | bool | `true` | Enable token caching |
| `max_cache_entries` | usize | `1000` | Maximum number of cached entries |
| `cache_cleanup_interval` | u64 | `60` | Cache cleanup interval in minutes |
| `pot_cache_duration` | u64 | `1800` (30 min) | POT token cache duration in seconds |
| `pot_generation_timeout` | u64 | `30` | POT token generation timeout in seconds |

**Example:**
```toml
[token]
ttl_hours = 12
# All other fields will use default values
```

### `[logging]` - Logging Configuration

All fields in the `[logging]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `level` | string | `"info"` | Log level (trace, debug, info, warn, error) |
| `verbose` | bool | `false` | Enable verbose logging |
| `format` | string | `"text"` | Log format (text, json) |
| `log_requests` | bool | `true` | Enable request/response logging |

**Example:**
```toml
[logging]
level = "debug"
# All other fields will use default values
```

### `[network]` - Network and Proxy Configuration

All fields in the `[network]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `https_proxy` | string (optional) | `None` | HTTPS proxy URL |
| `http_proxy` | string (optional) | `None` | HTTP proxy URL |
| `all_proxy` | string (optional) | `None` | All protocols proxy URL |
| `connect_timeout` | u64 | `30` | Connection timeout in seconds |
| `request_timeout` | u64 | `60` | Request timeout in seconds |
| `max_retries` | u32 | `3` | Number of retry attempts |
| `retry_interval` | u64 | `5000` | Retry interval in milliseconds |
| `user_agent` | string | `"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"` | User agent string |

**Example:**
```toml
[network]
https_proxy = "https://proxy.example.com:8080"
# All other fields will use default values
```

### `[botguard]` - BotGuard Configuration

All fields in the `[botguard]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `request_key` | string | `"O43z0dpjhgX20SCx4KAo"` | Request key for BotGuard API |
| `enable_vm` | bool | `true` | Enable JavaScript VM execution |
| `vm_timeout` | u64 | `30` | VM execution timeout in seconds |
| `disable_innertube` | bool | `false` | Force disable Innertube API usage |
| `challenge_endpoint` | string (optional) | `None` | Custom challenge endpoint URL |
| `snapshot_path` | path (optional) | Platform-specific data directory | BotGuard snapshot file path |
| `user_agent` | string (optional) | `None` | Custom User Agent for BotGuard |
| `disable_snapshot` | bool | `false` | Disable snapshot functionality |

**Example:**
```toml
[botguard]
enable_vm = false
# All other fields will use default values
```

### `[cache]` - Cache Configuration

All fields in the `[cache]` section are optional.

| Field | Type | Default Value | Description |
|-------|------|---------------|-------------|
| `cache_dir` | string (optional) | `None` | Cache directory path (for script mode) |
| `enable_file_cache` | bool | `true` | Enable file-based caching |
| `memory_cache_size` | usize | `100` | Memory cache size limit |
| `enable_compression` | bool | `false` | Enable cache compression |

**Example:**
```toml
[cache]
memory_cache_size = 200
# All other fields will use default values
```

## Minimal Configuration Examples

### Example 1: Only Override Host

```toml
[server]
host = "127.0.0.1"
```

This configuration will:
- Bind to `127.0.0.1` (specified)
- Use port `4416` (default)
- Use all other default values for all sections

### Example 2: Empty Configuration File

```toml
# Empty file or file with only comments
```

This is a valid configuration that will use all default values.

### Example 3: Empty Sections

```toml
[server]

[token]

[logging]
```

This is also valid. Each section will use all its default values.

### Example 4: Mixed Partial Configuration

```toml
[server]
host = "0.0.0.0"

[token]
ttl_hours = 12

[logging]
level = "debug"

[network]
https_proxy = "https://proxy.example.com:8080"
```

This configuration only specifies a few fields. All other fields across all sections will use their default values.

## Configuration Priority

The configuration system follows this priority order (highest to lowest):

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (TOML)
4. **Default values** (lowest priority)

This means:
- If you specify a value in the TOML file, it overrides the default
- If you set an environment variable, it overrides both the TOML file and the default
- If you provide a command-line argument, it overrides everything else

## Environment Variables

The following environment variables can override configuration values:

| Environment Variable | Config Field | Example |
|---------------------|--------------|---------|
| `POT_SERVER_HOST` | `server.host` | `POT_SERVER_HOST=127.0.0.1` |
| `POT_SERVER_PORT` | `server.port` | `POT_SERVER_PORT=8080` |
| `POT_SERVER_TIMEOUT` | `server.timeout` | `POT_SERVER_TIMEOUT=60` |
| `TOKEN_TTL` | `token.ttl_hours` | `TOKEN_TTL=12` |
| `LOG_LEVEL` | `logging.level` | `LOG_LEVEL=debug` |
| `VERBOSE` | `logging.verbose` | `VERBOSE=true` |
| `HTTPS_PROXY` | `network.https_proxy` | `HTTPS_PROXY=https://proxy:8080` |
| `HTTP_PROXY` | `network.http_proxy` | `HTTP_PROXY=http://proxy:8080` |
| `ALL_PROXY` | `network.all_proxy` | `ALL_PROXY=socks5://proxy:1080` |
| `DISABLE_INNERTUBE` | `botguard.disable_innertube` | `DISABLE_INNERTUBE=true` |
| `CACHE_DIR` | `cache.cache_dir` | `CACHE_DIR=/tmp/cache` |
| `BGUTIL_CONFIG` | Configuration file path | `BGUTIL_CONFIG=/path/to/config.toml` |

## Testing

Comprehensive tests for optional fields are available in `tests/config_optional_fields.rs`. These tests verify:

1. Each field can be omitted individually
2. Sections can be empty
3. The entire configuration file can be empty
4. Partial configurations work correctly
5. Default values are applied correctly
6. Integration with environment variables

Run the tests with:
```bash
cargo test --test config_optional_fields
```

## Troubleshooting

### Issue: "missing field" error when loading configuration

**Solution:** This was a bug in versions prior to 0.5.3 and has been fixed. All fields are now properly optional. Update to the latest version.

### Issue: Configuration not loading from file

**Solution:** Check that:
1. The file path is correct (use `BGUTIL_CONFIG` environment variable or default location)
2. The file has valid TOML syntax
3. Field names match exactly (they are case-sensitive)

### Issue: Default value not what I expected

**Solution:** Check this document for the correct default values. You can also run the application with `-v` flag to see the loaded configuration in debug output.
