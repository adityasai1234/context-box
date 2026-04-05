# ContextBox Setup Guide

## Quick Start

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Clone and Build

```bash
cd contextbox
cargo build --release
```

### 3. Generate Encryption Key (Important!)

```bash
./target/release/cb keygen
```

This creates your encryption key at `~/.config/contextbox/key.txt`

**WARNING**: Back up this key! If lost, documents cannot be recovered.

### 4. Start Server

```bash
# Local only (for testing)
./target/release/context-box serve

# Or with API key
./target/release/context-box serve --api-key YOUR_STRONG_KEY
```

## CLI Commands

### Local Operations

```bash
# Generate encryption key
cb keygen

# Add document
cb add --file document.md

# List documents
cb list

# Get document by ID
cb get <document-id>

# Delete document
cb delete <document-id>

# Search documents
cb search "query"

# Setup wizard
cb setup
```

### Remote Operations

```bash
# Add to remote server
cb remote add \
  --url https://yourdomain.com \
  --api-key YOUR_KEY \
  --file document.md

# List remote documents
cb remote list \
  --url https://yourdomain.com \
  --api-key YOUR_KEY

# Search remote
cb remote search \
  --url https://yourdomain.com \
  --api-key YOUR_KEY \
  "query"
```

## Security Setup

### Generate Strong API Key

```bash
# Generate random key
openssl rand -base64 32
```

### Environment Variables

Create a `.env` file:

```bash
CONTEXTBOX_API_KEY=your_generated_key_here
DATA_DIR=./data
OPENROUTER_API_KEY=sk-or-xxx  # Optional: for AI features
```

### Secure Your Key

```bash
# Set proper permissions
chmod 600 ~/.config/contextbox/key.txt

# Backup your encryption key
cp ~/.config/contextbox/key.txt ~/backup-key.txt
```

## Remote Access with HTTPS

### Using Caddy (Recommended)

See [PROXY.md](PROXY.md) for full setup.

Quick version:

1. Get a domain pointing to your server
2. Install Caddy: `sudo apt install caddy`
3. Configure Caddyfile
4. Start ContextBox: `context-box serve --host 127.0.0.1 --port 8080 --api-key YOUR_KEY`
5. Access via: `https://yourdomain.com`

### Firewall

See [SECURITY.md](SECURITY.md) for full setup.

Quick version:

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw deny 8080/tcp   # Block ContextBox port
sudo ufw enable
```

## Configuration Options

| Flag | Description | Default |
|------|-------------|---------|
| `--host` | Server bind address | `127.0.0.1` |
| `--port` | Server port | `8080` |
| `--api-key` | API authentication | None |
| `--cors` | CORS origins | `localhost:3000` |
| `--data-dir` | Data directory | `./data` |

## Troubleshooting

### "Encryption key not found"

Run: `cb keygen`

### "Connection refused"

1. Check server is running: `ps aux | grep context-box`
2. Check port: `netstat -tlnp | grep 8080`

### "API key required"

Make sure to pass `--api-key YOUR_KEY` to commands

## Files and Locations

| Path | Description |
|------|-------------|
| `~/.config/contextbox/key.txt` | Encryption key |
| `./data/documents.db` | Encrypted document database |
| `/etc/caddy/Caddyfile` | Caddy configuration |

## Next Steps

1. Generate encryption key: `cb keygen`
2. Start server: `context-box serve --api-key YOUR_KEY`
3. Add documents: `cb add --file doc.md`
4. Set up Caddy for HTTPS (see PROXY.md)
5. Configure firewall (see SECURITY.md)

## More Info

- [PROXY.md](PROXY.md) - Caddy/HTTPS setup
- [SECURITY.md](SECURITY.md) - Firewall and security
- [PROJECT_PLAN.md](PROJECT_PLAN.md) - Architecture
