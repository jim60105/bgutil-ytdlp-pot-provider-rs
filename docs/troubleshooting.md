# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the BgUtils POT Provider Rust implementation.

## Common Issues

### BotGuard Initialization Failed

**Error Messages:**
- `BotGuard client failed to initialize`
- `Failed to create BotGuard instance`
- `BotGuard challenge timeout`

**Symptoms:**
- Server fails to start
- Token generation returns 503 Service Unavailable
- Logs show BotGuard initialization errors

**Solutions:**

1. **Check Network Connectivity**
   ```bash
   # Test connectivity to YouTube services
   curl -I https://www.youtube.com/
   curl -I https://youtubei.googleapis.com/
   ```

2. **Verify Proxy Configuration**
   ```bash
   # If using proxy, test proxy connectivity
   curl --proxy http://your-proxy:port https://www.youtube.com/
   
   # Check environment variables
   echo $HTTPS_PROXY
   echo $HTTP_PROXY
   echo $ALL_PROXY
   ```

3. **Check System Resources**
   ```bash
   # Monitor memory usage
   free -h
   
   # Check available disk space
   df -h
   
   # Monitor CPU usage
   top
   ```

4. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug bgutil-pot server --verbose
   ```

### Token Generation Timeout

**Error Messages:**
- `Token generation timed out`
- `BotGuard challenge timeout`
- `Request timeout exceeded`

**Symptoms:**
- Slow response times (>30 seconds)
- HTTP 500 errors
- Client timeouts

**Solutions:**

1. **Increase Timeout Settings**
   ```toml
   # config.toml
   [botguard]
   vm_timeout = 60  # Increase from default 30 seconds
   
   [token]
   pot_generation_timeout = 60  # Increase from default 30 seconds
   
   [network]
   request_timeout = 120  # Increase from default 60 seconds
   ```

2. **Check System Performance**
   ```bash
   # Check CPU and memory usage
   htop
   
   # Monitor network latency
   ping www.youtube.com
   ```

3. **Optimize Cache Settings**
   ```toml
   # config.toml
   [token]
   enable_cache = true
   max_cache_entries = 1000
   pot_cache_duration = 3600  # 1 hour cache
   ```

### High Memory Usage

**Symptoms:**
- Server memory usage continuously increasing
- Out of memory errors
- System becomes unresponsive

**Solutions:**

1. **Reduce Cache Size**
   ```toml
   # config.toml
   [token]
   max_cache_entries = 100  # Reduce from default 1000
   
   [cache]
   memory_cache_size = 50   # Reduce from default 100
   ```

2. **Enable Cache Compression**
   ```toml
   # config.toml
   [cache]
   enable_compression = true
   ```

3. **Monitor Memory Usage**
   ```bash
   # Check current memory usage
   ps aux | grep bgutil-pot
   
   # Monitor in real-time
   watch -n 1 'ps aux | grep bgutil-pot'
   ```

4. **Restart Server Periodically**
   ```bash
   # Example systemd timer for daily restart
   # /etc/systemd/system/bgutil-restart.timer
   [Unit]
   Description=Restart BgUtils POT Provider daily
   
   [Timer]
   OnCalendar=daily
   Persistent=true
   
   [Install]
   WantedBy=timers.target
   ```

### Connection Refused Errors

**Error Messages:**
- `Connection refused`
- `No connection could be made`
- `Failed to connect to server`

**Symptoms:**
- yt-dlp cannot connect to POT provider
- HTTP 503 errors
- Server not responding

**Solutions:**

1. **Check Server Status**
   ```bash
   # Test server health
   curl http://localhost:4416/ping
   
   # Check if server is running
   ps aux | grep bgutil-pot
   
   # Check listening ports
   netstat -tulpn | grep 4416
   ```

2. **Verify Network Configuration**
   ```bash
   # Test with different addresses
   curl http://127.0.0.1:4416/ping
   curl http://[::1]:4416/ping
   
   # Check firewall rules
   sudo ufw status
   sudo iptables -L
   ```

3. **Start Server with Correct Binding**
   ```bash
   # Bind to all interfaces
   bgutil-pot server --host 0.0.0.0 --port 4416
   
   # Bind to specific interface
   bgutil-pot server --host 127.0.0.1 --port 4416
   ```

### POT Tokens Not Working

**Symptoms:**
- yt-dlp still receives 403 errors
- "Sign in to confirm you're not a bot" message persists
- Downloads fail despite POT token

**Solutions:**

1. **Verify Token Generation**
   ```bash
   # Test token generation manually
   bgutil-pot --content-binding "dQw4w9WgXcQ" --verbose
   
   # Check token format and length
   curl -X POST http://localhost:4416/get_pot \
     -H "Content-Type: application/json" \
     -d '{"content_binding": "test"}'
   ```

2. **Try Legacy Mode**
   ```bash
   # Use with yt-dlp legacy mode
   yt-dlp --extractor-args "youtubepot-bgutilhttp:disable_innertube=1" "VIDEO_URL"
   ```

3. **Clear Caches**
   ```bash
   # Clear provider caches
   curl -X POST http://localhost:4416/invalidate_caches
   
   # Restart server to clear all state
   pkill bgutil-pot
   bgutil-pot server
   ```

4. **Check Your IP Status**
   ```bash
   # Try from different network/IP
   # Use proxy or VPN
   bgutil-pot server --config config-with-proxy.toml
   ```

5. **Update Software**
   ```bash
   # Check for updates
   git pull origin master
   cargo build --release
   
   # Check yt-dlp version
   yt-dlp --version
   ```

## Debug Mode Commands

### Enable Comprehensive Logging

```bash
# Maximum verbosity
RUST_LOG=trace bgutil-pot server --verbose

# Specific module logging
RUST_LOG=bgutil_ytdlp_pot_provider::session=debug bgutil-pot server

# Log to file
RUST_LOG=debug bgutil-pot server 2>&1 | tee pot-provider.log
```

### Health Check Commands

```bash
# Basic health check
curl http://localhost:4416/ping

# Check minter cache status
curl http://localhost:4416/minter_cache

# Test token generation
curl -X POST http://localhost:4416/get_pot \
  -H "Content-Type: application/json" \
  -d '{"content_binding": "test", "bypass_cache": true}'
```

### Performance Testing

```bash
# Benchmark token generation
time bgutil-pot --content-binding "test"

# Load test server
for i in {1..10}; do
  curl -X POST http://localhost:4416/get_pot \
    -H "Content-Type: application/json" \
    -d "{\"content_binding\": \"test$i\"}" &
done
wait
```

## Getting Help

If you continue to experience issues:

1. **Check System Requirements**
   - Rust 1.85+ for building from source
   - 512MB available memory
   - Stable internet connection
   - Current yt-dlp version (2025.05.22+)

2. **Collect Debug Information**
   ```bash
   # System information
   uname -a
   
   # Rust version (if building from source)
   rustc --version
   cargo --version
   
   # yt-dlp version
   yt-dlp --version
   
   # Server logs with debug info
   RUST_LOG=debug bgutil-pot server --verbose 2>&1 | tee debug.log
   ```

3. **Create Issue Report**
   - Visit: https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/issues
   - Include error messages, logs, and system information
   - Describe steps to reproduce the issue
   - Include your configuration (remove sensitive data)

4. **Community Support**
   - Check existing issues for similar problems
   - Search the documentation and README
   - Review the TypeScript implementation for comparison

## Environment Variables Reference

For debugging, these environment variables can be helpful:

| Variable | Purpose | Example |
|----------|---------|---------|
| `RUST_LOG` | Control logging level | `debug`, `trace` |
| `POT_SERVER_HOST` | Override server host | `127.0.0.1` |
| `POT_SERVER_PORT` | Override server port | `8080` |
| `HTTPS_PROXY` | HTTPS proxy URL | `https://proxy:8080` |
| `HTTP_PROXY` | HTTP proxy URL | `http://proxy:8080` |
| `ALL_PROXY` | All protocols proxy | `socks5://proxy:1080` |
| `DISABLE_INNERTUBE` | Disable Innertube API | `true` |
| `TOKEN_TTL` | Token TTL in hours | `12` |

## Known Limitations

1. **Geographic Restrictions**: Some regions may have additional restrictions that POT tokens cannot bypass.

2. **Rate Limiting**: Excessive requests may trigger YouTube's rate limiting regardless of POT tokens.

3. **IP Reputation**: Severely flagged IP addresses may require additional measures beyond POT tokens.

4. **Browser Requirements**: Some scenarios may require full browser simulation that POT tokens alone cannot provide.