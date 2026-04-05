# ContextBox Cloudflare Tunnel Setup

## Overview

This guide explains how to set up ContextBox with Cloudflare Tunnel for secure remote access without opening any ports on your firewall.

## Why Cloudflare Tunnel?

- No open ports required
- Automatic HTTPS via Cloudflare
- DDoS protection included
- Simple setup
- No port forwarding on router

## Prerequisites

- Cloudflare account
- Domain using Cloudflare DNS
- Linux server/machine

## Step 1: Install cloudflared

```bash
# Download cloudflared
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared

# Make executable
chmod +x /usr/local/bin/cloudflared

# Verify installation
cloudflared --version
```

## Step 2: Authenticate with Cloudflare

```bash
# This opens your browser to log in
cloudflared tunnel login
```

Follow the browser instructions to authenticate and select your domain.

## Step 3: Create a Tunnel

```bash
# Create tunnel named "contextbox"
cloudflared tunnel create contextbox
```

This creates a tunnel and shows you a UUID like:
```
Tunnel credentials saved to: /root/.cloudflared/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.json
```

## Step 4: Point Domain to Tunnel

```bash
# Route your subdomain to the tunnel
cloudflared tunnel route dns contextbox context.yourdomain.com
```

Replace `context.yourdomain.com` with your actual domain.

## Step 5: Create Configuration File

Create `/etc/cloudflared/config.yml`:

```yaml
tunnel: contextbox
credentials-file: /root/.cloudflared/xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.json

ingress:
  - hostname: context.yourdomain.com
    service: http://localhost:8080
  - service: http_status:404
```

Replace:
- `context.yourdomain.com` with your domain
- `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.json` with your actual tunnel UUID

## Step 6: Start ContextBox

```bash
# Generate key first if not done
./cb keygen

# Start ContextBox on localhost only
./context-box serve --host 127.0.0.1 --port 8080 --api-key YOUR_STRONG_KEY
```

## Step 7: Run the Tunnel

```bash
# Run cloudflared as a service
cloudflared --config /etc/cloudflared/config.yml tunnel run
```

Or test first:
```bash
cloudflared tunnel --url http://localhost:8080 run contextbox
```

## Step 8: Verify Setup

```bash
# Test HTTPS access
curl -H "X-API-Key: YOUR_STRONG_KEY" https://context.yourdomain.com/health
```

Expected response:
```json
{"status":"ok","service":"ContextBox"}
```

## Remote Access

Once setup, access from anywhere:

```bash
# Add document to remote server
cb remote add \
  --url https://context.yourdomain.com \
  --api-key YOUR_STRONG_KEY \
  --file document.md

# List remote documents
cb remote list \
  --url https://context.yourdomain.com \
  --api-key YOUR_STRONG_KEY

# Search remote documents
cb remote search \
  --url https://context.yourdomain.com \
  --api-key YOUR_STRONG_KEY \
  "search query"
```

## Running as a Service (Recommended)

### Systemd Service

Create `/etc/systemd/system/cloudflared.service`:

```ini
[Unit]
Description=Cloudflare Tunnel
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cloudflared --config /etc/cloudflared/config.yml tunnel run
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

Then:

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable cloudflared
sudo systemctl start cloudflared

# Check status
sudo systemctl status cloudflared
```

## Troubleshooting

### Tunnel Not Starting

1. Check credentials file path in config matches actual file
2. Verify tunnel exists: `cloudflared tunnel list`
3. Check logs: `journalctl -u cloudflared -f`

### Domain Not Resolving

1. Verify DNS is set: `dig context.yourdomain.com`
2. Check Cloudflare dashboard for tunnel status
3. Ensure domain is on Cloudflare DNS

### Connection Refused

1. Verify ContextBox is running: `ps aux | grep context-box`
2. Check it's on localhost:8080: `netstat -tlnp | grep 8080`
3. Test locally: `curl http://localhost:8080/health`

### SSL Certificate Errors

Cloudflare provides free certificates automatically. If you see errors:
1. Check tunnel is running
2. Verify domain is proxied through Cloudflare (orange cloud icon)

## Firewall Setup

With Cloudflare Tunnel, you can block all incoming ports:

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp  # SSH
sudo ufw enable
```

That's it! No need to open port 8080.

## Quick Reference

| Component | Command |
|-----------|---------|
| Install | `curl -L ... -o /usr/local/bin/cloudflared` |
| Login | `cloudflared tunnel login` |
| Create | `cloudflared tunnel create contextbox` |
| Route | `cloudflared tunnel route dns contextbox domain.com` |
| Run | `cloudflared tunnel run contextbox` |
| Status | `cloudflared tunnel list` |

## Security Benefits

| Feature | Protection |
|---------|------------|
| No open ports | Not exposed to internet |
| Cloudflare proxy | DDoS protection |
| API key | Authentication required |
| HTTPS | Encrypted traffic |
| Firewall | Block all incoming |

## Summary

| Step | Action |
|------|--------|
| 1 | Install cloudflared |
| 2 | Authenticate with Cloudflare |
| 3 | Create tunnel |
| 4 | Route domain to tunnel |
| 5 | Configure ingress |
| 6 | Start ContextBox on localhost |
| 7 | Run tunnel |
| 8 | Access via HTTPS |

Your domain now points to your local ContextBox without any open ports!
