# ContextBox - Project Plan

## Overview

| Field | Value |
|-------|-------|
| **Project Name** | ContextBox |
| **Type** | Self-hosted Document AI Platform |
| **License** | MIT |
| **Language** | Rust |
| **Purpose** | Like Context7 but self-hosted - users drag-drop docs, AI accesses via MCP |

## What is ContextBox?

ContextBox is a self-hosted alternative to Context7 that allows users to:
- **Upload documents** via drag-drop (code docs, PDFs, Markdown, text files)
- **AI access** via MCP server (Claude Desktop, Cursor, Windsurf, Codeium)
- **CLI tools** for command-line interaction
- **Run on low-end Linux machines** (4GB RAM minimum)

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           ContextBox                                     │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌───────────┐ │
│  │   Web UI    │   │   REST API  │   │ MCP Server  │   │    CLI    │ │
│  │ Drag-Drop   │──▶│   (Axum)    │◀──│ (AI Access) │   │  Tools    │ │
│  └─────────────┘   └──────┬──────┘   └─────────────┘   └───────────┘ │
│                            │                                           │
│                            ▼                                           │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                 │
│  │  Document   │──▶│   Vector    │──▶│  OpenRouter  │                 │
│  │  Parser     │   │   Store     │   │  (AI/Embed) │                 │
│  │(Kreuzberg)  │   │(TinyVector) │   │    API       │                 │
│  └─────────────┘   └─────────────┘   └─────────────┘                 │
└─────────────────────────────────────────────────────────────────────────┘
```

## Tech Stack

| Component | Tool | License |
|-----------|------|---------|
| Web Server | Axum | MIT |
| Vector DB | TinyVector | MIT |
| Document Parsing | Kreuzberg | Apache2 |
| MCP Protocol | model-context-protocol | MIT |
| CLI | Clap | MIT |
| Embeddings | OpenRouter API | SaaS |

## Directory Structure

```
contextbox/
├── .gitignore
├── Cargo.toml
├── rust-toolchain.toml
├── .env.example
├── README.md
├── LICENSE
├── docs/
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── MCP.md
│   ├── SETUP.md
│   ├── CLI.md
│   └── CONTRIBUTING.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── api/
│   │   ├── mod.rs
│   │   └── routes/
│   │       ├── mod.rs
│   │       ├── documents.rs
│   │       ├── search.rs
│   │       └── chat.rs
│   ├── mcp/
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   └── tools.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── vector.rs
│   │   └── document.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   └── extractor.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── embedding.rs
│   │   └── chat.rs
│   └── cli/
│       ├── mod.rs
│       └── commands.rs
├── frontend/
│   ├── index.html
│   ├── style.css
│   └── app.js
├── scripts/
│   ├── setup.sh
│   └── install-deps.sh
└── tests/
    └── integration.rs
```

## Features

### Core Features (Always Included)
- REST API server (Axum)
- Vector storage (TinyVector)
- Document parsing (Kreuzberg)
- Configuration management

### Optional Features (Toggleable)
| Feature | Description | Default |
|---------|-------------|---------|
| MCP Server | AI access via MCP protocol | Enabled |
| CLI Tools | Command-line interface | Enabled |
| Web UI | Browser drag-drop interface | Optional |
| RAG Chat | Chat with your documents | Optional |
| URL Crawl | Fetch docs from URLs | Optional |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/documents` | Upload document |
| POST | `/api/documents/url` | Crawl URL |
| GET | `/api/documents` | List documents |
| GET | `/api/documents/:id` | Get document |
| DELETE | `/api/documents/:id` | Delete document |
| POST | `/api/search` | Semantic search |
| POST | `/api/chat` | RAG chat (optional) |
| GET | `/health` | Health check |
| GET | `/api/config` | Get enabled features |

## MCP Server Tools

```json
{
  "tools": [
    {
      "name": "search_documents",
      "description": "Search documents by semantic query",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": { "type": "string" },
          "limit": { "type": "number", "default": 5 }
        }
      }
    },
    {
      "name": "list_documents",
      "description": "List all available documents"
    },
    {
      "name": "get_document",
      "description": "Get specific document content",
      "inputSchema": {
        "type": "object",
        "properties": {
          "id": { "type": "string" }
        }
      }
    },
    {
      "name": "add_document",
      "description": "Add a new document",
      "inputSchema": {
        "type": "object",
        "properties": {
          "content": { "type": "string" },
          "metadata": { "type": "object" }
        }
      }
    }
  ]
}
```

## CLI Commands

```bash
# Start server
contextbox serve

# Add document
contextbox add <file-path>

# Search
contextbox search <query>

# List documents
contextbox list

# Delete
contextbox delete <id>

# MCP mode
contextbox mcp

# Generate MCP config
contextbox config mcp
```

## Hardware Requirements

| Level | CPU | RAM | Storage |
|-------|-----|-----|---------|
| Minimal | 2 cores | 4GB | 20GB SSD |
| Recommended | 4 cores | 8GB | 50GB SSD |

## Implementation Phases

| Phase | Components | Priority |
|-------|------------|----------|
| 1 | Config, Error types, Basic API | High |
| 2 | Storage: TinyVector, Document storage | High |
| 3 | Parser: Document extraction | High |
| 4 | MCP: Server implementation | High |
| 5 | CLI: Commands | Medium |
| 6 | AI: Embedding, Chat | Medium |
| 7 | Web UI: Drag-drop | Low |
| 8 | URL Crawl | Low |

## Configuration (.env.example)

```bash
HOST=127.0.0.1
PORT=8080
OPENROUTER_API_KEY=sk-or-xxxxx
DATA_DIR=./data
VECTOR_DB_PATH=./data/vectors.db
API_KEY=optional-api-key
CORS_ORIGINS=http://localhost:3000
ENABLE_MCP=true
ENABLE_WEB_UI=false
ENABLE_CLI=true
ENABLE_CHAT=false
ENABLE_URL_CRAWL=false
LOG_LEVEL=info
```

## Next Steps

1. Review this plan
2. Confirm architecture
3. Start implementation in phases

## Questions for Review

1. Project name: Keep "contextbox"?
2. Default features: MCP + CLI enabled?
3. Start with Phase 1 implementation?
