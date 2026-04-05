# ContextBox

Self-hosted Document AI Platform - Like Context7 but self-hosted.

## Overview

ContextBox allows you to:
- **Upload documents** via drag-drop (code docs, PDFs, Markdown, text)
- **AI access** via MCP server (Claude Desktop, Cursor, Windsurf)
- **Search** your documents semantically
- **Run on low-end Linux machines** (4GB RAM minimum)

## Features

- REST API server (Axum)
- MCP Server for AI clients
- CLI tools
- Web UI (drag-drop) - Coming soon
- RAG Chat - Coming soon
- URL Crawling - Coming soon

## Quick Start

### Prerequisites

- Rust 1.75+
- Linux server

### Build

```bash
cd contextbox
cargo build --release
```

### Configure

```bash
cp .env.example .env
# Edit .env and add your OpenRouter API key
```

### Run

```bash
# Start the server
cargo run --release

# Or use CLI
cargo run --release -- cli serve
```

### Upload Documents

```bash
# Via CLI
cargo run --release -- cli add --file /path/to/doc.md

# Via API
curl -X POST http://localhost:8080/api/documents \
  -F "file=@/path/to/doc.md"
```

### Connect AI (MCP)

Generate MCP config:
```bash
cargo run --release -- cli config mcp
```

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server host | `127.0.0.1` |
| `PORT` | Server port | `8080` |
| `OPENROUTER_API_KEY` | API key for embeddings | - |
| `DATA_DIR` | Data directory | `./data` |
| `ENABLE_MCP` | Enable MCP server | `true` |
| `ENABLE_CLI` | Enable CLI | `true` |
| `ENABLE_CHAT` | Enable chat | `false` |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/documents` | Upload document |
| GET | `/api/documents` | List documents |
| GET | `/api/documents/:id` | Get document |
| DELETE | `/api/documents/:id` | Delete document |
| POST | `/api/search` | Semantic search |
| POST | `/api/chat` | RAG chat |

## CLI Reference

All commands use the `cb` binary.

### Key management

```bash
cb keygen          # generate encryption key (run once on first install)
cb key show        # print the key contents
cb key path        # print the key file path
```

### Local document operations

```bash
cb add --file doc.md               # add a file
cb add --file doc.md --name "My Doc"  # add with custom name
cb add --content "raw text here"   # add inline text

cb list                            # list all documents
cb get <id>                        # show document content (decrypted)
cb delete <id>                     # delete a document
cb search "query"                  # keyword search across documents
```

### Remote document operations

Interact with a ContextBox server running on another machine (or via Cloudflare Tunnel):

```bash
cb remote add \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY \
  --file doc.md

cb remote add \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY \
  --content "raw text"

cb remote list \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY

cb remote search \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY \
  "search query"
```

### Config and info

```bash
cb config mcp      # print MCP server config (paste into Claude Desktop / Cursor)
cb config cli      # print CLI config (data dir, key path, env vars)
cb setup           # quick setup check (key + database)
```

### Global flags

```bash
cb --data-dir /path/to/data <command>   # override data directory
cb --api-key KEY remote list ...        # pass API key inline
cb --url http://host:8080 remote list   # pass server URL inline
```

---

## Setup

### Automated Setup (Linux)

The setup script handles everything interactively: Rust install, build, encryption key generation, systemd service, Cloudflare Tunnel, and firewall. Supports Arch Linux (pacman), Ubuntu/Debian (apt), and Fedora (dnf).

```bash
git clone https://github.com/adityasai1234/context-box.git
cd context-box/contextbox
bash setup.sh
```

The script will walk you through each step and let you skip optional ones.

### Manual Setup

**1. Install Rust**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**2. Build and install binaries**

```bash
cd contextbox
cargo build --release
sudo cp target/release/context-box /usr/local/bin/context-box
sudo cp target/release/cb /usr/local/bin/cb
```

**3. Generate encryption key**

```bash
cb keygen
```

Key is stored at `~/.config/contextbox/key.txt`. Back it up — lost key means lost documents.

**4. Create config**

```bash
mkdir -p ~/.config/contextbox
cat > ~/.config/contextbox/.env <<EOF
HOST=127.0.0.1
PORT=8080
API_KEY=$(openssl rand -base64 32)
DATA_DIR=$HOME/contextbox-data
ENABLE_MCP=true
EOF
```

**5. Run as a systemd service**

Create `/etc/systemd/system/contextbox.service`:

```ini
[Unit]
Description=ContextBox Document AI Server
After=network.target

[Service]
Type=simple
User=YOUR_USER
EnvironmentFile=/home/YOUR_USER/.config/contextbox/.env
ExecStart=/usr/local/bin/context-box serve
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable contextbox
sudo systemctl start contextbox
```

Verify:

```bash
curl http://localhost:8080/health
# {"status":"ok","service":"ContextBox"}
```

---

## Cloudflare Tunnel

Expose ContextBox over HTTPS without opening any ports. Requires a Cloudflare account and a domain on Cloudflare DNS.

**1. Install cloudflared**

```bash
curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
  -o /usr/local/bin/cloudflared
chmod +x /usr/local/bin/cloudflared
```

**2. Authenticate and create tunnel**

```bash
cloudflared tunnel login
cloudflared tunnel create contextbox
cloudflared tunnel route dns contextbox docs.yourdomain.com
```

**3. Create tunnel config**

Create `/etc/cloudflared/config.yml`:

```yaml
tunnel: contextbox
credentials-file: /root/.cloudflared/<TUNNEL-UUID>.json

ingress:
  - hostname: docs.yourdomain.com
    service: http://localhost:8080
  - service: http_status:404
```

Replace `<TUNNEL-UUID>` with the UUID shown after `cloudflared tunnel create`. Check with `cloudflared tunnel list`.

**4. Run tunnel as a systemd service**

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

```bash
sudo systemctl daemon-reload
sudo systemctl enable cloudflared
sudo systemctl start cloudflared
```

**5. Verify**

```bash
curl -H "X-API-Key: YOUR_API_KEY" https://docs.yourdomain.com/health
# {"status":"ok","service":"ContextBox"}
```

**Firewall (lock down the server)**

With Cloudflare Tunnel you only need SSH open:

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp
sudo ufw enable
```

**Add documents remotely**

```bash
cb remote add \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY \
  --file mydoc.md

cb remote list \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY

cb remote search \
  --url https://docs.yourdomain.com \
  --api-key YOUR_API_KEY \
  "search query"
```

**Connect AI clients (MCP)**

Add to your Claude Desktop / Cursor MCP config:

```json
{
  "mcpServers": {
    "contextbox": {
      "command": "cb",
      "args": ["config", "mcp"],
      "env": {
        "CONTEXTBOX_URL": "https://docs.yourdomain.com",
        "API_KEY": "YOUR_API_KEY"
      }
    }
  }
}
```

---

## License

MIT License - See [LICENSE](LICENSE)
