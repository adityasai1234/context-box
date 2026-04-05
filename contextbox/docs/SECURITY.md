# ContextBox Security Guide

## Security Overview

This guide covers security best practices for running ContextBox.

## Security Layers

| Layer | Protection |
|-------|------------|
| Network | Firewall blocks unauthorized ports |
| Transport | HTTPS/TLS encrypts traffic |
| Application | API key authentication |
| Storage | Documents encrypted at rest |

## Firewall Setup

### Why Firewall?

Block external access to ports you don't need exposed.

### UFW Commands (Ubuntu/Debian)

```bash
# Install UFW if not installed
sudo apt install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (important!)
sudo ufw allow 22/tcp

# Allow HTTPS (Caddy)
sudo ufw allow 443/tcp

# Allow HTTP (Caddy for cert challenges)
sudo ufw allow 80/tcp

# Explicitly block ContextBox port
sudo ufw deny 8080/tcp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

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
443/tcp                    ALLOW       Anywhere
80/tcp                     ALLOW       Anywhere
8080/tcp                  DENY        Anywhere
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

- [ ] Firewall configured (only 22, 80, 443 open)
- [ ] Port 8080 blocked from external access
- [ ] Strong API key generated (32+ random characters)
- [ ] API key stored in environment variable, not in scripts
- [ ] Encryption key backed up securely
- [ ] Caddy HTTPS working
- [ ] Test remote access works
- [ ] Check logs for any errors

## Monitoring

### Check Server Logs

```bash
# If running in background with logs
tail -f /var/log/contextbox.log

# Or check systemd journal
journalctl -u contextbox -f
```

### Check Caddy Logs

```bash
sudo journalctl -u caddy -f
```

## Emergency Procedures

### If Compromised

1. Stop the server immediately
2. Generate new API key
3. Generate new encryption key (note: old documents won't be recoverable)
4. Update firewall rules
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
| Firewall | Block all unnecessary ports |
| Updates | Keep system and Rust updated |
| Backups | Backup encryption key securely |
| Logs | Monitor for suspicious activity |
| SSH | Use key-based authentication, disable password login |

## Network Diagram

```
                    INTERNET
                        |
                        ▼
              ┌───────────────────┐
              │   Firewall        │
              │   - Allow 22     │  (SSH)
              │   - Allow 80,443 │  (HTTPS)
              │   - Block 8080   │
              └────────┬──────────┘
                       │
              ┌──────┴──────┐
              ▼             ▼
        ┌─────────┐   ┌─────────┐
        │  Caddy  │   │  SSH    │
        │  :443    │   │  :22    │
        └────┬────┘   └────┬────┘
             │             │
             ▼             │
      ┌───────────┐      │
      │ContextBox │      │
      │ :8080    │◄─────┘
      │(local    │
      │ only)    │
      └───────────┘
```

## Summary

| Security Measure | Status |
|-----------------|--------|
| Firewall | Block ports 8080, allow 443 |
| HTTPS | Via Caddy |
| API Key | Required for all operations |
| Encryption | AES-256-GCM at rest |
| SSH Key | Use key-based auth |
| Key Backup | Store securely |
