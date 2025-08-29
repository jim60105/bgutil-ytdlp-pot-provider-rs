# API Reference

This document provides comprehensive reference for the BgUtils POT Provider Rust implementation APIs.

## HTTP API Endpoints

### POST /get_pot

Generate a POT token for the specified content.

**Request Body:**
```json
{
  "content_binding": "video_id_or_content_identifier",
  "visitor_data": "optional_visitor_data",
  "data_sync_id": "optional_data_sync_id"
}
```

**Response:**
```json
{
  "po_token": "actual_bot_guard_generated_token",
  "expires_at": "2024-01-01T12:00:00Z"
}
```

**Status Codes:**
- `200 OK`: Token generated successfully
- `400 Bad Request`: Invalid request format or missing required fields
- `500 Internal Server Error`: BotGuard generation failed
- `503 Service Unavailable`: BotGuard client not initialized

### GET /ping

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "message": "BgUtils POT Provider is running"
}
```

### POST /invalidate_caches

Clear all internal caches.

**Response:**
```json
{
  "status": "ok",
  "message": "Caches invalidated successfully"
}
```

### POST /invalidate_it

Invalidate integrity tokens (legacy compatibility).

**Response:**
```json
{
  "status": "ok",
  "message": "Integrity tokens invalidated"
}
```

### GET /minter_cache

Get minter cache status information.

**Response:**
```json
{
  "cache_size": 42,
  "cache_entries": [
    {
      "content_binding": "video_id",
      "expires_at": "2024-01-01T12:00:00Z",
      "created_at": "2024-01-01T06:00:00Z"
    }
  ]
}
```

### POST /get_pot

Generate or retrieve a cached POT token.

**Request Format:**
```json
{
  "content_binding": "L3KvsX8hJss",
  "proxy": "http://proxy.example.com:8080",
  "bypass_cache": false,
  "source_address": "192.168.1.100",
  "disable_tls_verification": false,
  "disable_innertube": false
}
```

**Request Fields:**
- `content_binding` (string, optional): Video ID or content identifier
- `proxy` (string, optional): Proxy server URL  
- `bypass_cache` (boolean, optional): Force new token generation, bypassing cache
- `source_address` (string, optional): Source IP address for outbound connections
- `disable_tls_verification` (boolean, optional): Disable TLS certificate verification
- `disable_innertube` (boolean, optional): Disable Innertube API usage
- `challenge` (string, optional): Challenge parameter for token generation
- `innertube_context` (object, optional): Innertube context for API calls

**Response Format:**
```json
{
  "poToken": "QUFFLUhqbXI3OEFmTWowWWZTUFFkR3hqV1Y5Q2JFeFVFZ3xBQ3Jtc0tqVlFEUmhOelJrWVRLcFd3T1Q2aVRxZEhP",
  "expiresAt": "2024-08-25T12:00:00Z",
  "contentBinding": "L3KvsX8hJss"
}
```

**Response Fields:**
- `poToken` (string): The generated POT token
- `expiresAt` (string): ISO 8601 timestamp when token expires
- `contentBinding` (string): Content binding used for token generation

**Error Response:**
```json
{
  "error": "data_sync_id is deprecated, use content_binding instead"
}
```

**Status Codes:**
- `200 OK`: Token generated successfully
- `400 Bad Request`: Invalid request parameters (e.g., deprecated fields)
- `500 Internal Server Error`: Server error during token generation

**Example Request:**
```bash
curl -X POST http://127.0.0.1:4416/get_pot \
  -H "Content-Type: application/json" \
  -d '{
    "content_binding": "L3KvsX8hJss",
    "bypass_cache": false
  }'
```

### GET /ping

Health check endpoint for basic connectivity testing.

**Response Format:**
```json
{
  "server_uptime": 3600,
  "version": "0.1.0"
}
```

**Response Fields:**
- `server_uptime` (number): Server uptime in seconds
- `version` (string): Application version

**Status Codes:**
- `200 OK`: Service is healthy

### POST /invalidate_caches

Invalidate all cached tokens and sessions.

**Request:** No request body required.

**Response:** Returns HTTP 204 No Content on success.

**Status Codes:**
- `204 No Content`: Caches invalidated successfully
- `500 Internal Server Error`: Failed to invalidate caches

**Example Request:**
```bash
curl -X POST http://127.0.0.1:4416/invalidate_caches
```

### POST /invalidate_it

Invalidate integrity tokens to force regeneration.

**Request:** No request body required.

**Response:** Returns HTTP 204 No Content on success.

**Status Codes:**
- `204 No Content`: Integrity tokens invalidated successfully
- `500 Internal Server Error`: Failed to invalidate integrity tokens

**Example Request:**
```bash
curl -X POST http://127.0.0.1:4416/invalidate_it
```

### GET /minter_cache

Get minter cache keys for debugging purposes.

**Response Format:**
```json
["cache_key_1", "cache_key_2", "cache_key_3"]
```

**Response:** Returns an array of cache key strings.

**Status Codes:**
- `200 OK`: Cache keys retrieved successfully
- `500 Internal Server Error`: Failed to retrieve cache keys

**Example Request:**
```bash
curl http://127.0.0.1:4416/minter_cache
```

## CLI Interface

### bgutil-pot-server

HTTP server mode for always-running POT provider service.

**Usage:**
```bash
bgutil-pot-server [OPTIONS]
```

**Options:**
- `--host <HOST>`: Server bind address (default: ::)
- `--port <PORT>`: Listen port (default: 4416)
- `--config <FILE>`: Configuration file path
- `--verbose`: Enable verbose logging
- `--help`: Show help information
- `--version`: Show version information

**Examples:**
```bash
# Start with default settings (IPv6 with IPv4 fallback)
bgutil-pot-server

# Custom host and port
bgutil-pot-server --host 127.0.0.1 --port 8080

# With verbose logging
bgutil-pot-server --verbose

# Using configuration file
bgutil-pot-server --config /path/to/config.toml
```

### bgutil-pot-generate

Script mode for single POT token generation.

**Usage:**
```bash
bgutil-pot-generate [OPTIONS]
```

**Options:**
- `-c, --content-binding <CONTENT_BINDING>`: Content binding (video ID, visitor data, etc.)
- `-v, --visitor-data <VISITOR_DATA>`: Visitor data (DEPRECATED: use --content-binding instead)
- `-d, --data-sync-id <DATA_SYNC_ID>`: Data sync ID (DEPRECATED: use --content-binding instead)
- `-p, --proxy <PROXY>`: Proxy server URL (http://host:port, socks5://host:port, etc.)
- `-b, --bypass-cache`: Bypass cache and force new token generation
- `-s, --source-address <SOURCE_ADDRESS>`: Source IP address for outbound connections
- `--disable-tls-verification`: Disable TLS certificate verification
- `--version`: Show version information
- `--verbose`: Enable verbose logging
- `-h, --help`: Print help

**Output Format:**

**JSON Format (default):**
```json
{
  "poToken": "QUFFLUhqbXI3OEFmTWowWWZTUFFkR3hqV1Y5Q2JFeFVFZ3xBQ3Jtc0tqVlFEUmhOelJrWVRLcFd3T1E2aVRxZEhP",
  "contentBinding": "L3KvsX8hJss",
  "expiresAt": "2024-08-25T12:00:00Z"
}
```

**Error Output:**
```json
{}
```

**Examples:**
```bash
# Basic token generation
bgutil-pot-generate --content-binding "L3KvsX8hJss"

# With proxy
bgutil-pot-generate --content-binding "L3KvsX8hJss" --proxy "http://proxy.example.com:8080"

# Bypass cache for fresh token
bgutil-pot-generate --content-binding "L3KvsX8hJss" --bypass-cache

# With source address
bgutil-pot-generate --content-binding "L3KvsX8hJss" --source-address "192.168.1.100"

# Verbose logging
bgutil-pot-generate --content-binding "L3KvsX8hJss" --verbose

# Using deprecated parameters (will show error and exit)
bgutil-pot-generate --visitor-data "CgtVa2F6cWl6blE4QTi5"
bgutil-pot-generate --data-sync-id "abc123"
```

**Exit Codes:**
- `0`: Success
- `1`: Invalid arguments, deprecated parameters, or token generation failure

## Configuration File Format

Both binaries support TOML configuration files.

**Example Configuration:**
```toml
[server]
bind = "127.0.0.1"
port = 4416

[logging]
level = "info"
format = "pretty"

[cache]
ttl_hours = 6
max_entries = 1000
enable_file_cache = true
cache_dir = "~/.cache/bgutil-pot-provider"

[network]
connect_timeout = 30
request_timeout = 60
max_retries = 3
retry_interval = 1
user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"

[botguard]
request_key = "AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w"
vm_timeout = 5000

[token]
ttl_hours = 6
contexts = ["gvs", "player", "subs"]
```

**Configuration Sections:**

### [server]
- `bind` (string): Server bind address
- `port` (number): Server listen port

### [logging]
- `level` (string): Log level (error, warn, info, debug, trace)
- `format` (string): Log format (pretty, json)

### [cache]
- `ttl_hours` (number): Token TTL in hours
- `max_entries` (number): Maximum cache entries
- `enable_file_cache` (boolean): Enable persistent cache
- `cache_dir` (string): Cache directory path

### [network]
- `connect_timeout` (number): Connection timeout in seconds
- `request_timeout` (number): Request timeout in seconds
- `max_retries` (number): Maximum retry attempts
- `retry_interval` (number): Retry interval in seconds
- `user_agent` (string): HTTP User-Agent string

### [botguard]
- `request_key` (string): YouTube API request key
- `vm_timeout` (number): JavaScript VM timeout in milliseconds

### [token]
- `ttl_hours` (number): Default token TTL
- `contexts` (array): Supported token contexts

## Environment Variables

Configuration can also be provided via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `BGUTIL_BIND` | Server bind address | `127.0.0.1` |
| `BGUTIL_PORT` | Server port | `4416` |
| `BGUTIL_CONFIG` | Config file path | `~/.config/bgutil-pot-provider/config.toml` |
| `TOKEN_TTL` | Token TTL (hours) | `6` |
| `CACHE_DIR` | Cache directory | `~/.cache/bgutil-pot-provider` |
| `HTTP_PROXY` | HTTP proxy URL | - |
| `HTTPS_PROXY` | HTTPS proxy URL | - |
| `NO_PROXY` | No proxy hosts | - |

**Environment Variable Priority:**
1. Command line arguments (highest)
2. Environment variables
3. Configuration file
4. Default values (lowest)

## Integration with yt-dlp

### HTTP Provider Integration

When using the HTTP server mode, yt-dlp automatically detects the provider:

```bash
# Default usage (server must be running on 127.0.0.1:4416)
yt-dlp "https://www.youtube.com/watch?v=VIDEO_ID"

# Custom server URL
yt-dlp --extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:8080" "VIDEO_URL"

# With additional options
yt-dlp --extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:4416;disable_innertube=1" "VIDEO_URL"
```

### Script Provider Integration

When using script mode:

```bash
# Default location (if installed in home directory)
yt-dlp "https://www.youtube.com/watch?v=VIDEO_ID"

# Custom script path
yt-dlp --extractor-args "youtubepot-bgutilscript:script_path=/path/to/bgutil-pot-generate" "VIDEO_URL"
```

### Extractor Arguments

**HTTP Provider (`youtubepot-bgutilhttp`):**
- `base_url`: POT provider server URL
- `disable_innertube`: Disable Innertube API usage

**Script Provider (`youtubepot-bgutilscript`):**
- `script_path`: Path to bgutil-pot-generate binary

**Multiple Arguments:**
Separate multiple arguments with semicolons:
```bash
--extractor-args "youtubepot-bgutilhttp:base_url=http://127.0.0.1:8080;disable_innertube=1"
```

## Error Handling

### Error Categories

**Validation Errors (HTTP 400):**
- Missing required fields
- Invalid field formats
- Unsupported parameter values

**Network Errors (HTTP 502/503):**
- YouTube API unavailable
- Connection timeouts
- Proxy connection failures

**Rate Limiting (HTTP 429):**
- Too many requests from same IP
- API rate limits exceeded

**Server Errors (HTTP 500):**
- Internal processing errors
- BotGuard execution failures
- Cache corruption

### Error Response Format

```json
{
  "error": "Human readable error message",
  "category": "validation|network|rate_limit|server",
  "details": {
    "field": "specific_field_name",
    "code": "ERROR_CODE",
    "message": "Detailed error information"
  },
  "timestamp": "2024-08-25T12:00:00Z",
  "request_id": "req_123456789"
}
```

### Retry Recommendations

**For Client Applications:**
1. **Validation Errors**: Fix request and retry
2. **Network Errors**: Retry with exponential backoff (max 3 attempts)
3. **Rate Limiting**: Wait and retry after delay
4. **Server Errors**: Retry with exponential backoff

**Recommended Retry Logic:**
```bash
# Example retry with curl
for i in {1..3}; do
  if curl -f http://127.0.0.1:4416/get_pot -d "$request_body"; then
    break
  fi
  sleep $((i * 2))  # Exponential backoff
done
```

## Performance and Scalability

### Performance Characteristics

**Response Times (typical):**
- Cache hit: < 10ms
- New token generation: 1-2 seconds
- Cold start: < 3 seconds

**Throughput:**
- HTTP server: 100+ concurrent requests
- Script mode: Limited by process spawn overhead

**Resource Usage:**
- Memory: 20-50MB (normal operation)
- CPU: Minimal (except during token generation)
- Network: Low bandwidth requirements

### Scalability Recommendations

**For High Traffic:**
1. Use HTTP server mode (not script mode)
2. Configure appropriate cache TTL
3. Deploy behind load balancer for redundancy
4. Monitor cache hit rates
5. Use proxy rotation if needed

**Cache Optimization:**
- Increase `cache.max_entries` for high video diversity
- Adjust `cache.ttl_hours` based on usage patterns
- Enable persistent cache for restarts

**Network Optimization:**
- Use connection pooling (automatic in HTTP mode)
- Configure timeouts appropriately
- Monitor and rotate proxy endpoints