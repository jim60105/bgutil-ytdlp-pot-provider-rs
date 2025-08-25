# Deployment Guide

This guide covers production deployment options for the BgUtils POT Provider Rust implementation.

## Production Requirements

### System Requirements

**Minimum Hardware:**
- CPU: 1 core (x86_64 or ARM64)
- Memory: 512MB available RAM
- Storage: 100MB (binary + cache + logs)
- Network: Stable internet connection

**Recommended Hardware:**
- CPU: 2+ cores
- Memory: 2GB+ RAM
- Storage: 1GB+ (includes log rotation and cache growth)
- Network: Low-latency connection for optimal performance

**Operating System Support:**
- **Linux**: glibc 2.17+ (CentOS 7+, Ubuntu 16.04+, Debian 9+)
- **Windows**: Windows 10+ (x86_64)
- **macOS**: macOS 10.15+ (Intel), macOS 11+ (Apple Silicon)

### Network Requirements

**Outbound Connections:**
- YouTube API endpoints (*.youtube.com)
- Port 443 (HTTPS) for YouTube BotGuard API
- Optional: HTTP/SOCKS proxy support

**Inbound Connections (HTTP Server Mode):**
- Default port 4416 (configurable)
- HTTP/1.1 and HTTP/2 support
- CORS headers for web browser access

## Deployment Options

### Option 1: Direct Binary Deployment

The simplest deployment using pre-compiled binaries.

#### 1.1 Download and Installation

```bash
# Create dedicated user
sudo useradd --system --home /opt/bgutil-pot-provider --shell /bin/false bgutil

# Create directories
sudo mkdir -p /opt/bgutil-pot-provider/{bin,config,cache,logs}
sudo chown -R bgutil:bgutil /opt/bgutil-pot-provider

# Download binary
sudo wget -O /opt/bgutil-pot-provider/bin/bgutil-pot-server \
  https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs/releases/latest/download/bgutil-ytdlp-pot-provider-rs-linux-x86_64

sudo chmod +x /opt/bgutil-pot-provider/bin/bgutil-pot-server
```

#### 1.2 Configuration

Create configuration file:

```bash
sudo tee /opt/bgutil-pot-provider/config/config.toml << EOF
[server]
bind = "127.0.0.1"
port = 4416

[logging]
level = "info"
format = "json"

[cache]
ttl_hours = 6
max_entries = 1000
enable_file_cache = true
cache_dir = "/opt/bgutil-pot-provider/cache"

[network]
connect_timeout = 30
request_timeout = 60
max_retries = 3

[token]
ttl_hours = 6
contexts = ["gvs", "player", "subs"]
EOF

sudo chown bgutil:bgutil /opt/bgutil-pot-provider/config/config.toml
```

#### 1.3 Systemd Service

Create systemd service file:

```bash
sudo tee /etc/systemd/system/bgutil-pot-provider.service << EOF
[Unit]
Description=BgUtils POT Provider
Documentation=https://github.com/jim60105/bgutil-ytdlp-pot-provider-rs
After=network.target
Wants=network.target

[Service]
Type=simple
User=bgutil
Group=bgutil
WorkingDirectory=/opt/bgutil-pot-provider
ExecStart=/opt/bgutil-pot-provider/bin/bgutil-pot-server \\
  --config /opt/bgutil-pot-provider/config/config.toml \\
  --log-level info
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=5
TimeoutStopSec=30

# Security settings
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectHome=true
ProtectSystem=strict
ReadWritePaths=/opt/bgutil-pot-provider/cache /opt/bgutil-pot-provider/logs

# Resource limits
LimitNOFILE=65536
MemoryMax=256M

# Environment
Environment=RUST_LOG=info
Environment=BGUTIL_CONFIG=/opt/bgutil-pot-provider/config/config.toml

[Install]
WantedBy=multi-user.target
EOF
```

#### 1.4 Service Management

```bash
# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable bgutil-pot-provider
sudo systemctl start bgutil-pot-provider

# Check status
sudo systemctl status bgutil-pot-provider

# View logs
sudo journalctl -u bgutil-pot-provider -f

# Restart service
sudo systemctl restart bgutil-pot-provider
```

### Option 2: Docker Deployment

#### 2.1 Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  bgutil-pot-provider:
    image: jim60105/bgutil-ytdlp-pot-provider-rs:latest
    container_name: bgutil-pot-provider
    restart: unless-stopped
    ports:
      - "4416:4416"
    volumes:
      - ./config:/app/config:ro
      - ./cache:/app/cache
      - ./logs:/app/logs
    environment:
      - RUST_LOG=info
      - BGUTIL_CONFIG=/app/config/config.toml
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4416/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - bgutil-network

networks:
  bgutil-network:
    driver: bridge
```

#### 2.2 Build Custom Image

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bgutil-pot-server /usr/local/bin/
COPY --from=builder /app/target/release/bgutil-pot-generate /usr/local/bin/

RUN useradd --system --home /app --shell /bin/false bgutil && \
    chown -R bgutil:bgutil /app

USER bgutil

EXPOSE 4416

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4416/health || exit 1

CMD ["bgutil-pot-server", "--bind", "0.0.0.0", "--port", "4416"]
```

#### 2.3 Docker Deployment

```bash
# Using pre-built image
docker run -d \
  --name bgutil-pot-provider \
  --restart unless-stopped \
  -p 4416:4416 \
  -v ./config:/app/config:ro \
  -v ./cache:/app/cache \
  -e RUST_LOG=info \
  jim60105/bgutil-ytdlp-pot-provider-rs:latest

# Using docker-compose
docker-compose up -d

# Check status
docker logs bgutil-pot-provider
docker-compose logs -f
```

### Option 3: Kubernetes Deployment

#### 3.1 ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: bgutil-pot-provider-config
  namespace: default
data:
  config.toml: |
    [server]
    bind = "0.0.0.0"
    port = 4416

    [logging]
    level = "info"
    format = "json"

    [cache]
    ttl_hours = 6
    max_entries = 1000
    enable_file_cache = true
    cache_dir = "/app/cache"

    [network]
    connect_timeout = 30
    request_timeout = 60
    max_retries = 3
```

#### 3.2 Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bgutil-pot-provider
  namespace: default
  labels:
    app: bgutil-pot-provider
spec:
  replicas: 2
  selector:
    matchLabels:
      app: bgutil-pot-provider
  template:
    metadata:
      labels:
        app: bgutil-pot-provider
    spec:
      containers:
      - name: bgutil-pot-provider
        image: jim60105/bgutil-ytdlp-pot-provider-rs:latest
        ports:
        - containerPort: 4416
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: BGUTIL_CONFIG
          value: "/app/config/config.toml"
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: cache
          mountPath: /app/cache
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 4416
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ping
            port: 4416
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: bgutil-pot-provider-config
      - name: cache
        emptyDir: {}
```

#### 3.3 Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: bgutil-pot-provider-service
  namespace: default
spec:
  type: ClusterIP
  ports:
  - port: 4416
    targetPort: 4416
    protocol: TCP
    name: http
  selector:
    app: bgutil-pot-provider
```

#### 3.4 Ingress (Optional)

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: bgutil-pot-provider-ingress
  namespace: default
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: pot-provider.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: bgutil-pot-provider-service
            port:
              number: 4416
```

## Load Balancing and High Availability

### NGINX Load Balancer

```nginx
upstream bgutil_pot_provider {
    least_conn;
    server 127.0.0.1:4416 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:4417 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:4418 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name pot-provider.example.com;

    location / {
        proxy_pass http://bgutil_pot_provider;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Health check
        proxy_next_upstream error timeout http_500 http_502 http_503;
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    location /health {
        proxy_pass http://bgutil_pot_provider;
        access_log off;
    }
}
```

### HAProxy Configuration

```
global
    daemon
    log stdout local0 info

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httplog
    option dontlognull

frontend bgutil_frontend
    bind *:80
    default_backend bgutil_backend

backend bgutil_backend
    balance roundrobin
    option httpchk GET /health
    http-check expect status 200
    server bgutil1 127.0.0.1:4416 check inter 10s
    server bgutil2 127.0.0.1:4417 check inter 10s
    server bgutil3 127.0.0.1:4418 check inter 10s
```

## Monitoring and Logging

### Health Checks

**Built-in Endpoints:**
- `/health` - Comprehensive health check with metrics
- `/ping` - Basic connectivity test

**Custom Health Check Script:**
```bash
#!/bin/bash
# /opt/bgutil-pot-provider/bin/health-check.sh

ENDPOINT="http://127.0.0.1:4416/health"
TIMEOUT=10

response=$(curl -s -w "%{http_code}" --max-time $TIMEOUT "$ENDPOINT")
http_code=${response: -3}

if [ "$http_code" = "200" ]; then
    echo "Service is healthy"
    exit 0
else
    echo "Service is unhealthy (HTTP $http_code)"
    exit 1
fi
```

### Log Management

**Structured Logging Configuration:**
```toml
[logging]
level = "info"
format = "json"  # Easier for log aggregation
```

**Log Rotation with logrotate:**
```
# /etc/logrotate.d/bgutil-pot-provider
/opt/bgutil-pot-provider/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    copytruncate
    postrotate
        systemctl reload bgutil-pot-provider
    endscript
}
```

### Monitoring with Prometheus

**Metrics Endpoint:**
The service exposes metrics at `/metrics` (if enabled in configuration).

**Prometheus Configuration:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'bgutil-pot-provider'
    static_configs:
      - targets: ['localhost:4416']
    metrics_path: '/metrics'
    scrape_interval: 30s
```

## Security Considerations

### Network Security

**Firewall Rules:**
```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 4416/tcp  # POT Provider
sudo ufw --force enable
```

**Reverse Proxy with TLS:**
```nginx
server {
    listen 443 ssl http2;
    server_name pot-provider.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;

    location / {
        proxy_pass http://127.0.0.1:4416;
        # ... proxy headers
    }
}
```

### Access Control

**IP Whitelisting:**
```toml
[server]
bind = "127.0.0.1"  # Only local access
# Or use firewall rules for external access control
```

**Rate Limiting:**
```nginx
http {
    limit_req_zone $binary_remote_addr zone=pot_limit:10m rate=10r/s;
    
    server {
        location / {
            limit_req zone=pot_limit burst=20 nodelay;
            proxy_pass http://bgutil_pot_provider;
        }
    }
}
```

### Data Protection

**File Permissions:**
```bash
# Secure configuration files
sudo chmod 600 /opt/bgutil-pot-provider/config/config.toml
sudo chown bgutil:bgutil /opt/bgutil-pot-provider/config/config.toml

# Secure cache directory
sudo chmod 700 /opt/bgutil-pot-provider/cache
sudo chown bgutil:bgutil /opt/bgutil-pot-provider/cache
```

## Performance Tuning

### Configuration Optimization

**High-Traffic Configuration:**
```toml
[server]
bind = "0.0.0.0"
port = 4416

[cache]
ttl_hours = 8          # Longer cache for reduced API calls
max_entries = 5000     # Higher cache limit
enable_file_cache = true

[network]
connect_timeout = 15   # Shorter timeouts for faster failure
request_timeout = 30
max_retries = 2        # Fewer retries for faster response
```

### System Tuning

**File Descriptor Limits:**
```bash
# /etc/security/limits.conf
bgutil soft nofile 65536
bgutil hard nofile 65536
```

**Systemd Service Limits:**
```ini
[Service]
LimitNOFILE=65536
LimitNPROC=4096
```

### Cache Optimization

**Cache Size Calculation:**
- Average token size: ~1KB
- For 1000 cached tokens: ~1MB memory
- Monitor cache hit rate via `/health` endpoint
- Adjust `max_entries` based on video diversity

## Backup and Recovery

### Configuration Backup

```bash
#!/bin/bash
# /opt/bgutil-pot-provider/bin/backup-config.sh

BACKUP_DIR="/opt/bgutil-pot-provider/backups"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup configuration
tar -czf "$BACKUP_DIR/config_$DATE.tar.gz" \
    -C /opt/bgutil-pot-provider config/

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "config_*.tar.gz" -mtime +30 -delete
```

### Cache Recovery

**Cache is ephemeral and self-healing:**
- No backup needed for cache data
- Service automatically rebuilds cache on restart
- Monitor cache warming after service restart

### Disaster Recovery

**Service Recovery Steps:**
1. Restore configuration from backup
2. Restart service
3. Verify health endpoint
4. Monitor cache population
5. Check yt-dlp integration

**Recovery Time Objectives:**
- RTO (Recovery Time): < 5 minutes
- RPO (Recovery Point): Configuration only (no data loss)

## Troubleshooting

### Common Issues

**Service Won't Start:**
```bash
# Check configuration
bgutil-pot-server --config /path/to/config.toml --validate

# Check permissions
sudo -u bgutil bgutil-pot-server --version

# Check dependencies
ldd /opt/bgutil-pot-provider/bin/bgutil-pot-server
```

**High Memory Usage:**
```bash
# Check cache size
curl http://127.0.0.1:4416/health | jq '.cache_entries'

# Restart service to clear cache
sudo systemctl restart bgutil-pot-provider
```

**Network Connectivity:**
```bash
# Test YouTube connectivity
curl -v https://www.youtube.com/youtubei/v1/player

# Test with proxy
curl -v --proxy http://proxy:8080 https://www.youtube.com/
```

### Log Analysis

**Common Log Patterns:**
```bash
# Token generation success
grep "POT token generated" /var/log/bgutil-pot-provider.log

# Cache hits
grep "returning cached token" /var/log/bgutil-pot-provider.log

# Network errors
grep "Network error" /var/log/bgutil-pot-provider.log

# Rate limiting
grep "429" /var/log/bgutil-pot-provider.log
```

### Performance Analysis

**Response Time Monitoring:**
```bash
# Monitor response times
time curl -s http://127.0.0.1:4416/health

# Load testing
ab -n 100 -c 10 http://127.0.0.1:4416/ping
```

**Memory Profiling:**
```bash
# Monitor memory usage
ps aux | grep bgutil-pot-server

# System resource usage
top -p $(pgrep bgutil-pot-server)
```