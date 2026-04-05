# ContextBox Caddy Reverse Proxy Setup

## Overview

This guide explains how to set up Caddy as a reverse proxy for ContextBox to enable HTTPS access.

## Why Use Caddy?

- Automatic HTTPS with Let's Encrypt
- TLS 1.3 encryption
- Simple configuration
- Automatic certificate renewal

## Prerequisites

- Linux server with ContextBox installed
- Domain name pointing to your server IP
- SSH access to your server

## Step 1: Configure DNS

Before setting up Caddy, point your domain to your server:

1. Log into your domain registrar
2. Create an A record:
   - Name: `context` (or your preferred subdomain)
   - Value: Your server's public IP address
3. Wait for DNS propagation (can take up to 24 hours)

## Step 2: Install Caddy

### On Ubuntu/Debian

```bash
sudo apt update
sudo apt install caddy
```

### On Other Systems

Download from: https://caddyserver.com/docs/install

## Step 3: Configure Caddy

Create a Caddyfile at `/etc/caddy/Caddyfile`:

```Caddyfile
# Replace with your actual domain
context.yourdomain.com {
    reverse_proxy localhost:8080
    
    # Optional: Only allow specific IP ranges
    # @allowed {
    #     remote_ip 192.168.0.0/16
    #     remote_ip 10.0.0.0/8
    # }
    # respond @allowed "Access denied" 403
}
```

Then reload Caddy:

```bash
sudo caddy reload
```

## Step 4: Verify Setup

1. Start ContextBox:
```bash
context-box serve --host 127.0.0.1 --port 8080 --api-key YOUR_STRONG_KEY
```

2. Test HTTPS access:
```bash
curl -H "X-API-Key: YOUR_STRONG_KEY" https://context.yourdomain.com/health
```

3. Expected response:
```json
{"status": "ok", "service": "ContextBox"}
```

## Step 5: Using Remote Commands

Once HTTPS is working:

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

## Troubleshooting

### Certificate Issues

If Let's Encrypt fails to issue a certificate:

1. Ensure your domain DNS is correctly pointing to your server IP
2. Check that ports 80 and 443 are open
3. Check Caddy logs: `sudo journalctl -u caddy -f`

### Connection Refused

If you can't connect:

1. Verify ContextBox is running: `ps aux | grep context-box`
2. Check it's listening on localhost:8080: `netstat -tlnp | grep 8080`
3. Check Caddy can reach it: `curl http://localhost:8080/health`

### SSL Certificate Errors

If you get certificate warnings:

1. The certificate might be self-signed (for testing)
2. Wait for Let's Encrypt to issue (can take a few minutes)
3. Check certificate status: `caddy list-certificates`

## Custom Domain

To use a different subdomain, update the Caddyfile:

```Caddyfile
your-preferred-domain.com {
    reverse_proxy localhost:8080
}
```

Then reload: `sudo caddy reload`

## Additional Security Options

### Restrict by IP (Optional)

```Caddyfile
context.yourdomain.com {
    reverse_proxy localhost:8080
    
    # Allow only specific IPs
    @allowed {
        remote_ip 192.168.1.0/24
    }
    
    handle @allowed {
        reverse_proxy localhost:8080
    }
    
    respond "Access denied" 403
}
```

### Enable HSTS

```Caddyfile
context.yourdomain.com {
    reverse_proxy localhost:8080
    header Strict-Transport-Security "max-age=31536000"
}
```

## Summary

| Component | Port | Access |
|-----------|------|--------|
| ContextBox | 8080 | localhost only (not exposed) |
| Caddy | 443 | HTTPS (public) |
| HTTP | 80 | Redirects to HTTPS |

Your domain handles HTTPS traffic → Caddy forwards to ContextBox → Your documents stay encrypted and secure.
