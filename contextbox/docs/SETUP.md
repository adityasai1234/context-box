# Setup Guide

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

### 3. Configure

```bash
cp .env.example .env
# Edit .env and add your OpenRouter API key
echo "OPENROUTER_API_KEY=sk-or-xxxxx" >> .env
```

### 4. Run

```bash
# Start server
./target/release/contextbox

# Or use CLI
./target/release/cb serve
```

## CLI Usage

```bash
# Add document
cb add --file /path/to/doc.md

# Search
cb search "query"

# List
cb list

# Setup
cb setup --mcp --cli
```

## Configuration

See `.env.example` for all options.
