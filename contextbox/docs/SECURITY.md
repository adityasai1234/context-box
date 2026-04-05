# ContextBox Security Guide

## Security Overview

This guide covers security best practices for running ContextBox with Cloudflare Tunnel.

## Security Layers

| Layer | Protection |
|-------|------------|
| Network | No exposed ports (Tunnel handles everything) |
| Transport | HTTPS/TLS via Cloudflare |
| Application | API key authentication |
| Storage | Documents encrypted at rest |

## Firewall Setup

With Cloudflare Tunnel, you don't need to open any ports except SSH!

### UFW Commands (Ubuntu/Debian)

```bash
# Install UFW if not installed
sudo apt install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (important!)
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

That's it! No other ports needed with Cloudflare Tunnel.

### Verify Setup

```bash
sudo ufw status
```

Expected output:
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
```

## Generate Strong API Key

### Don't Use Simple Keys

Bad: `test123`, `password`, `admin`

Good: `cb_sk_3a7f9b2c4d8e1f6a...`

### Generate Random Key

```bash
# Using openssl
openssl rand -base64 32

# Using /dev/urandom
head -c 32 /dev/urandom | base64
```

### Use in Commands

```bash
context-box serve --api-key "your_generated_key_here"
```

## Environment Variables

Instead of passing API key in command line, use environment:

```bash
# Set environment variable
export CONTEXTBOX_API_KEY="your_strong_key_here"

# Start server (will read from env)
context-box serve
```

Or create a `.env` file:

```bash
# In your ContextBox directory
echo "CONTEXTBOX_API_KEY=your_strong_key_here" > .env
echo "DATA_DIR=./data" >> .env

# Start server - it will read from .env
context-box serve
```

## Secure Your Encryption Key

Your encryption key is stored at `~/.config/contextbox/key.txt`.

### Set Proper Permissions

```bash
# Only you can read/write
chmod 600 ~/.config/contextbox/key.txt

# Verify
ls -la ~/.config/contextbox/key.txt
```

### Backup Your Key

**IMPORTANT**: If you lose this key, you cannot recover your documents!

```bash
# Backup to secure location
cp ~/.config/contextbox/key.txt ~/backup-contextbox-key.txt

# Or use a password manager
# Copy the key content and store in your password manager
```

## Production Checklist

Before going live with remote access:

- [ ] Firewall configured (only SSH open)
- [ ] Cloudflare Tunnel running
- [ ] Strong API key generated (32+ random characters)
- [ ] API key stored in environment variable, not in scripts
- [ ] Encryption key backed up securely
- [ ] Test remote access works
- [ ] Check logs for any errors

## Monitoring

### Check ContextBox Logs

```bash
# If running in background
tail -f contextbox.log

# Or check systemd journal
journalctl -u contextbox -f
```

### Check Cloudflare Tunnel Logs

```bash
# Check tunnel status
cloudflared tunnel list

# Check logs
journalctl -u cloudflared -f
```

## Emergency Procedures

### If Compromised

1. Stop the tunnel immediately
2. Stop ContextBox
3. Generate new API key
4. Generate new encryption key (note: old documents won't be recoverable)
5. Investigate the breach

### If You Lose Your Encryption Key

Unfortunately, there is no recovery:
- Documents cannot be decrypted
- You will need to delete the database and start fresh
- This is by design for security

## Security Best Practices

| Practice | Recommendation |
|----------|----------------|
| API Key | Use 32+ random characters |
| Firewall | Block all incoming except SSH |
| Updates | Keep system and Rust updated |
| Backups | Backup encryption key securely |
| Logs | Monitor for suspicious activity |
| SSH | Use key-based authentication, disable password login |
| Cloudflare | Use strong Cloudflare account password + 2FA |

## Network Diagram

```
                    INTERNET
                        |
                        ▼
              ┌───────────────────┐
              │   Cloudflare     │
              │   (HTTPS/TLS)   │
              └────────┬──────────┘
                       │
              ┌──────┴──────┐
              │             │
       Cloudflare      Cloudflare
       Proxy           DNS
              │             │
              ▼             ▼
        ┌─────────┐   ┌─────────┐
        │ Tunnel  │   │ Your    │
        │ Daemon  │   │ Domain  │
        └────┬────┘   └─────────┘
             │
             ▼
      ┌───────────┐
      │ContextBox │
      │ :8080   │
      │(local   │
      │ only)   │
      └───────────┘

Firewall:
- SSH (22): ALLOW
- Everything else: DENY
```

## Summary

| Security Measure | Protection |
|-----------------|------------|
| Firewall | Block all, allow SSH only |
| Cloudflare Tunnel | No exposed ports |
| HTTPS | Via Cloudflare |
| API Key | Required for all operations |
| Encryption | AES-256-GCM at rest |
| SSH Key | Use key-based auth |
| Key Backup | Store securely |

## Quick Commands

```bash
# Start ContextBox (local only)
./context-box serve --host 127.0.0.1 --port 8080 --api-key YOUR_KEY

# Run Cloudflare Tunnel
cloudflared tunnel run contextbox

# Test access
curl -H "X-API-Key: YOUR_KEY" https://yourdomain.com/health

# Check tunnel status
cloudflared tunnel list
```
