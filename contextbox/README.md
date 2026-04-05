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

## License

MIT License - See [LICENSE](LICENSE)
