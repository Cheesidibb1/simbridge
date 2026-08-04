# SimBridge Deployment Guide

## Development Deployment

### Local Development

1. Start the server:
```bash
cd server
cargo run
```

2. Start the companion app:
```bash
cd companion
flutter run
```

3. Ensure both devices are on the same network

### Docker Deployment

#### Building the Server Docker Image

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/simbridge-server /usr/local/bin/
EXPOSE 8080
CMD ["simbridge-server"]
```

Build and run:
```bash
docker build -t simbridge-server .
docker run -p 8080:8080 -v $(pwd)/data:/data simbridge-server
```

## Production Deployment

### Server Deployment

#### System Requirements

- CPU: 2+ cores
- RAM: 4GB+
- Storage: 20GB+
- Network: 1Gbps recommended

#### Installation

1. Build the server:
```bash
cd server
cargo build --release
```

2. Copy the binary:
```bash
cp target/release/simbridge-server /usr/local/bin/
chmod +x /usr/local/bin/simbridge-server
```

3. Create a service user:
```bash
useradd -r -s /bin/false simbridge
```

4. Create directories:
```bash
mkdir -p /etc/simbridge
mkdir -p /var/lib/simbridge
mkdir -p /var/log/simbridge
chown simbridge:simbridge /var/lib/simbridge
chown simbridge:simbridge /var/log/simbridge
```

5. Create configuration:
```bash
cat > /etc/simbridge/config.toml << EOF
[server]
host = "0.0.0.0"
port = 8080

[database]
path = "/var/lib/simbridge/simbridge.db"

[recordings]
path = "/var/lib/simbridge/recordings"

[security]
enable_auth = true
tls_enabled = true
tls_cert = "/etc/simbridge/cert.pem"
tls_key = "/etc/simbridge/key.pem"
EOF
```

6. Create systemd service:
```bash
cat > /etc/systemd/system/simbridge.service << EOF
[Unit]
Description=SimBridge Server
After=network.target

[Service]
Type=simple
User=simbridge
ExecStart=/usr/local/bin/simbridge-server --config /etc/simbridge/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
```

7. Enable and start the service:
```bash
systemctl daemon-reload
systemctl enable simbridge
systemctl start simbridge
```

### Reverse Proxy Configuration

#### Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name simbridge.example.com;

    ssl_certificate /etc/letsencrypt/live/simbridge.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/simbridge.example.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws {
        proxy_pass http://localhost:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### Companion App Deployment

#### Android

1. Build the APK:
```bash
cd companion
flutter build apk --release
```

2. Build the App Bundle:
```bash
flutter build appbundle --release
```

3. Upload to Google Play Store

#### iOS

1. Build the IPA:
```bash
cd companion
flutter build ios --release
```

2. Upload to App Store

### Desktop Dashboard Deployment

#### Windows

```bash
cd desktop
flutter build windows --release
```

The executable will be in `build/windows/x64/runner/Release/`

#### macOS

```bash
cd desktop
flutter build macos --release
```

The application will be in `build/macos/Build/Products/Release/`

#### Linux

```bash
cd desktop
flutter build linux --release
```

The executable will be in `build/linux/x64/release/`

## Monitoring

### Health Checks

```bash
curl http://localhost:8080/health
```

### Logs

```bash
journalctl -u simbridge -f
```

### Metrics

Enable Prometheus metrics in configuration and expose on `/metrics` endpoint.

## Backup

### Database Backup

```bash
cp /var/lib/simbridge/simbridge.db /backup/simbridge-$(date +%Y%m%d).db
```

### Configuration Backup

```bash
tar -czf /backup/simbridge-config-$(date +%Y%m%d).tar.gz /etc/simbridge
```

## Scaling

### Horizontal Scaling

For multiple server instances:

1. Use a load balancer (Nginx, HAProxy)
2. Use a shared database (PostgreSQL instead of SQLite)
3. Configure sticky sessions for WebSocket connections
4. Use Redis for session storage

### Vertical Scaling

Increase server resources based on usage:
- More CPU for encoding/decoding
- More RAM for concurrent sessions
- Faster storage for recordings

## Security

### TLS/SSL

Always use TLS in production:
- Obtain certificates from Let's Encrypt
- Configure proper cipher suites
- Enable HSTS
- Use strong key sizes (2048-bit+)

### Firewall

```bash
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
```

### Rate Limiting

Configure rate limiting in the server configuration to prevent abuse.

## Troubleshooting

### Server Won't Start

Check logs:
```bash
journalctl -u simbridge -n 50
```

### Connection Issues

1. Verify firewall rules
2. Check network connectivity
3. Validate TLS certificates
4. Review server logs

### Performance Issues

1. Monitor CPU, memory, and network usage
2. Check concurrent session count
3. Review streaming quality settings
4. Optimize database queries
