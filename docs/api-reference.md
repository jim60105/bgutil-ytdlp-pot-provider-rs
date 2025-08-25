# API Reference

This document provides comprehensive reference for the BgUtils POT Provider Rust implementation APIs.

## HTTP API Endpoints

### POST /api/v1/pot

Generate or retrieve a cached POT token.

**Request Format:**
```json
{
  "visitor_data": "CgtVa2F6cWl6blE4QSi5",
  "content_binding": "L3KvsX8hJss",
  "po_token_context": "gvs"
}
```

**Request Fields:**
- `visitor_data` (string): YouTube visitor data identifier
- `content_binding` (string): Video ID or content identifier  
- `po_token_context` (string, optional): Token context type ("gvs", "player", "subs"). Default: "gvs"

**Response Format:**
```json
{
  "token": "QUFFLUhqbXI3OEFmTWowWWZTUFFkR3hqV1Y5Q2JFeFVFZ3xBQ3Jtc0tqVlFEUmhOelJrWVRLcFd3T1Q2aVRxZEhP",
  "expires_at": "2024-08-25T12:00:00Z",
  "content_binding": "L3KvsX8hJss",
  "context": "gvs"
}
```

**Response Fields:**
- `token` (string): The generated POT token
- `expires_at` (string): ISO 8601 timestamp when token expires
- `content_binding` (string): Echo of the request content binding
- `context` (string): Token context type

**Error Response:**
```json
{
  "error": "Invalid visitor data format",
  "category": "validation",
  "details": {
    "field": "visitor_data",
    "message": "Visitor data must be valid base64"
  }
}
```

**Status Codes:**
- `200 OK`: Token generated successfully
- `400 Bad Request`: Invalid request parameters
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error during token generation

**Example Request:**
```bash
curl -X POST http://127.0.0.1:4416/api/v1/pot \
  -H "Content-Type: application/json" \
  -d '{
    "visitor_data": "CgtVa2F6cWl6blE4QSi5",
    "content_binding": "L3KvsX8hJss",
    "po_token_context": "gvs"
  }'
```

### GET /health

Health check endpoint for monitoring and load balancers.

**Response Format:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "cache_entries": 42
}
```

**Response Fields:**
- `status` (string): Service health status ("healthy", "degraded", "unhealthy")
- `version` (string): Application version
- `uptime_seconds` (number): Server uptime in seconds
- `cache_entries` (number): Number of cached tokens

**Status Codes:**
- `200 OK`: Service is healthy
- `503 Service Unavailable`: Service is unhealthy

### GET /ping

Simple ping endpoint for basic connectivity testing.

**Response Format:**
```json
{
  "message": "pong",
  "timestamp": "2024-08-25T12:00:00Z"
}
```

### POST /api/v1/invalidate

Invalidate cached tokens for specific content.

**Request Format:**
```json
{
  "content_binding": "L3KvsX8hJss",
  "invalidation_type": "content"
}
```

**Request Fields:**
- `content_binding` (string): Content binding to invalidate
- `invalidation_type` (string): Type of invalidation ("content", "all")

**Response Format:**
```json
{
  "message": "Cache invalidated successfully",
  "invalidated_entries": 3
}
```

### GET /api/v1/cache

Get minter cache information.

**Response Format:**
```json
{
  "cache_size": 42,
  "cache_entries": [
    {
      "content_binding": "L3KvsX8hJss",
      "context": "gvs",
      "expires_at": "2024-08-25T12:00:00Z"
    }
  ]
}
```

## CLI Interface

### bgutil-pot-server

HTTP server mode for always-running POT provider service.

**Usage:**
```bash
bgutil-pot-server [OPTIONS]
```

**Options:**
- `--bind <ADDRESS>`: Bind address (default: 127.0.0.1)
- `--port <PORT>`: Listen port (default: 4416)
- `--config <FILE>`: Configuration file path
- `--log-level <LEVEL>`: Logging level (error, warn, info, debug, trace)
- `--verbose`: Enable verbose logging
- `--help`: Show help information
- `--version`: Show version information

**Examples:**
```bash
# Start with default settings
bgutil-pot-server

# Custom bind address and port
bgutil-pot-server --bind 0.0.0.0 --port 8080

# With verbose logging
bgutil-pot-server --verbose

# Using configuration file
bgutil-pot-server --config /path/to/config.toml
```

### bgutil-pot-generate

Script mode for single POT token generation.

**Usage:**
```bash
bgutil-pot-generate [OPTIONS] --content-binding <CONTENT_BINDING>
```

**Required Options:**
- `--content-binding <CONTENT_BINDING>`: Video ID or content identifier

**Optional Options:**
- `--po-token-context <CONTEXT>`: Token context (gvs, player, subs, default: gvs)
- `--output <FORMAT>`: Output format (json, token-only, default: json)
- `--proxy <PROXY_URL>`: HTTP/SOCKS proxy URL
- `--source-address <ADDRESS>`: Source IP address for outbound connections
- `--bypass-cache`: Force new token generation, bypass cache
- `--disable-tls-verification`: Disable TLS certificate verification
- `--verbose`: Enable verbose logging
- `--version`: Show version information
- `--help`: Show help information

**Output Formats:**

**JSON Format (default):**
```json
{
  "poToken": "QUFFLUhqbXI3OEFmTWowWWZTUFFkR3hqV1Y5Q2JFeFVFZ3xBQ3Jtc0tqVlFEUmhOelJrWVRLcFd3T1E2aVRxZEhP",
  "contentBinding": "L3KvsX8hJss",
  "expiresAt": "2024-08-25T12:00:00Z"
}
```

**Token-only Format:**
```
QUFFLUhqbXI3OEFmTWowWWZTUFFkR3hqV1Y5Q2JFeFVFZ3xBQ3Jtc0tqVlFEUmhOelJrWVRLcFd3T1E2aVRxZEhP
```

**Examples:**
```bash
# Basic token generation
bgutil-pot-generate --content-binding "L3KvsX8hJss"

# With specific context
bgutil-pot-generate --content-binding "L3KvsX8hJss" --po-token-context "player"

# Token-only output
bgutil-pot-generate --content-binding "L3KvsX8hJss" --output token-only

# With proxy
bgutil-pot-generate --content-binding "L3KvsX8hJss" --proxy "http://proxy.example.com:8080"

# Bypass cache for fresh token
bgutil-pot-generate --content-binding "L3KvsX8hJss" --bypass-cache

# Verbose logging
bgutil-pot-generate --content-binding "L3KvsX8hJss" --verbose
```

**Exit Codes:**
- `0`: Success
- `1`: Invalid arguments or configuration error
- `2`: Network error or API failure
- `3`: Token generation failure

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
  if curl -f http://127.0.0.1:4416/api/v1/pot -d "$request_body"; then
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