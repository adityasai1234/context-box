#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
#  ContextBox Setup Script
#  Runs interactively on Linux (Ubuntu/Debian)
# ─────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

ok()   { echo -e "${GREEN}[✓]${RESET} $*"; }
info() { echo -e "${CYAN}[i]${RESET} $*"; }
warn() { echo -e "${YELLOW}[!]${RESET} $*"; }
err()  { echo -e "${RED}[✗]${RESET} $*"; }
ask()  { echo -e "${YELLOW}[?]${RESET} $*"; }

trap 'err "Setup failed at line $LINENO. Fix the error above and re-run."; exit 1' ERR

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="$HOME/.config/contextbox"
ENV_FILE="$CONFIG_DIR/.env"
KEY_FILE="$CONFIG_DIR/key.txt"

# ─────────────────────────────────────────────
#  Banner
# ─────────────────────────────────────────────
print_banner() {
  echo -e "${CYAN}"
  cat <<'EOF'
   ____            _            _   ____
  / ___|___  _ __ | |_ _____  _| |_| __ )  _____  __
 | |   / _ \| '_ \| __/ _ \ \/ / __|  _ \ / _ \ \/ /
 | |__| (_) | | | | ||  __/>  <| |_| |_) | (_) >  <
  \____\___/|_| |_|\__\___/_/\_\\__|____/ \___/_/\_\

        Self-hosted Document AI Platform
EOF
  echo -e "${RESET}"
  echo -e "${BOLD}  Interactive Setup — Linux Edition${RESET}"
  echo    "  ─────────────────────────────────────"
  echo
}

# ─────────────────────────────────────────────
#  Helpers
# ─────────────────────────────────────────────
confirm() {
  local prompt="${1:-Continue?} [y/N] "
  local reply
  read -rp "$(echo -e "${YELLOW}[?]${RESET} $prompt")" reply
  [[ "${reply,,}" == "y" || "${reply,,}" == "yes" ]]
}

prompt() {
  # prompt <var_name> <question> <default>
  local var="$1" question="$2" default="$3"
  local reply
  read -rp "$(echo -e "${YELLOW}[?]${RESET} ${question} [${default}]: ")" reply
  eval "$var=\"${reply:-$default}\""
}

require_root_or_sudo() {
  if [[ $EUID -ne 0 ]] && ! sudo -n true 2>/dev/null; then
    warn "Some steps need sudo. You may be prompted for your password."
  fi
}

# ─────────────────────────────────────────────
#  Step 1: OS check
# ─────────────────────────────────────────────

# Global: set by step_os, used by step_deps / step_firewall
PKG_MANAGER=""

detect_distro() {
  if [[ -f /etc/os-release ]]; then
    # shellcheck source=/dev/null
    source /etc/os-release
    echo "${ID:-unknown}"
  elif command -v lsb_release &>/dev/null; then
    lsb_release -si | tr '[:upper:]' '[:lower:]'
  else
    echo "unknown"
  fi
}

step_os() {
  echo -e "\n${BOLD}── Step 1: System Check ──────────────────────${RESET}"
  if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    err "This script is for Linux only. Detected: $OSTYPE"
    exit 1
  fi

  local distro
  distro=$(detect_distro)
  info "Detected distro: $distro"

  case "$distro" in
    ubuntu|debian|linuxmint|pop)
      PKG_MANAGER="apt"
      ;;
    arch|manjaro|endeavouros|garuda)
      PKG_MANAGER="pacman"
      ;;
    fedora|rhel|centos|rocky|alma)
      PKG_MANAGER="dnf"
      warn "Fedora/RHEL support is best-effort — package names may differ."
      ;;
    *)
      warn "Unknown distro '$distro'. Will try to auto-detect package manager."
      if command -v pacman &>/dev/null;   then PKG_MANAGER="pacman"
      elif command -v apt-get &>/dev/null; then PKG_MANAGER="apt"
      elif command -v dnf &>/dev/null;    then PKG_MANAGER="dnf"
      else
        warn "No supported package manager found. Install deps manually if needed."
        PKG_MANAGER="none"
      fi
      ;;
  esac

  ok "Package manager: $PKG_MANAGER"
}

# ─────────────────────────────────────────────
#  Step 2: Dependencies
# ─────────────────────────────────────────────
pkg_install() {
  # pkg_install <pkg...>  — installs using the detected package manager
  case "$PKG_MANAGER" in
    apt)    sudo apt-get update -qq && sudo apt-get install -y "$@" ;;
    pacman) sudo pacman -Sy --noconfirm "$@" ;;
    dnf)    sudo dnf install -y "$@" ;;
    *)      err "Cannot auto-install: $*. Install them manually."; return 1 ;;
  esac
}

# Map generic dep names to distro-specific package names
pkg_name() {
  local dep="$1"
  case "$PKG_MANAGER" in
    pacman)
      case "$dep" in
        openssl) echo "openssl" ;;
        *)       echo "$dep" ;;
      esac
      ;;
    *)
      echo "$dep"
      ;;
  esac
}

step_deps() {
  echo -e "\n${BOLD}── Step 2: Dependencies ──────────────────────${RESET}"
  local missing_cmds=() missing_pkgs=()

  for cmd in curl git openssl; do
    if ! command -v "$cmd" &>/dev/null; then
      missing_cmds+=("$cmd")
      missing_pkgs+=("$(pkg_name "$cmd")")
    else
      ok "$cmd found"
    fi
  done

  if [[ ${#missing_cmds[@]} -gt 0 ]]; then
    warn "Missing: ${missing_cmds[*]}"
    if confirm "Install missing packages via $PKG_MANAGER?"; then
      pkg_install "${missing_pkgs[@]}"
      ok "Installed: ${missing_cmds[*]}"
    else
      err "Cannot continue without: ${missing_cmds[*]}"
      exit 1
    fi
  fi
}

# ─────────────────────────────────────────────
#  Step 3: Rust
# ─────────────────────────────────────────────
step_rust() {
  echo -e "\n${BOLD}── Step 3: Rust ──────────────────────────────${RESET}"
  if command -v cargo &>/dev/null; then
    ok "Rust already installed ($(cargo --version))"
    return
  fi

  info "Rust not found."
  if confirm "Install Rust via rustup?"; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    ok "Rust installed ($(cargo --version))"
  else
    err "Rust is required to build ContextBox."
    exit 1
  fi
}

# ─────────────────────────────────────────────
#  Step 4: Build
# ─────────────────────────────────────────────
step_build() {
  echo -e "\n${BOLD}── Step 4: Build ContextBox ──────────────────${RESET}"

  # Ensure cargo is on PATH (in case rustup just installed it)
  export PATH="$HOME/.cargo/bin:$PATH"

  if command -v context-box &>/dev/null && command -v cb &>/dev/null; then
    ok "Binaries already installed ($(context-box --version 2>/dev/null || echo 'context-box'), cb)"
    if ! confirm "Rebuild and reinstall?"; then
      return
    fi
  fi

  info "Building in release mode (this takes a few minutes)..."
  cd "$SCRIPT_DIR"
  cargo build --release 2>&1

  info "Installing binaries to /usr/local/bin..."
  sudo cp target/release/context-box /usr/local/bin/context-box
  sudo cp target/release/cb /usr/local/bin/cb
  sudo chmod +x /usr/local/bin/context-box /usr/local/bin/cb

  ok "context-box and cb installed"
}

# ─────────────────────────────────────────────
#  Step 5: Encryption Key
# ─────────────────────────────────────────────
step_keygen() {
  echo -e "\n${BOLD}── Step 5: Encryption Key ────────────────────${RESET}"
  mkdir -p "$CONFIG_DIR"

  if [[ -f "$KEY_FILE" ]]; then
    ok "Encryption key already exists at $KEY_FILE"
    warn "Generating a new key will make existing documents unrecoverable."
    if ! confirm "Generate a new key anyway?"; then
      return
    fi
  fi

  cb keygen
  chmod 600 "$KEY_FILE"
  ok "Encryption key generated: $KEY_FILE"

  echo
  warn "IMPORTANT: Back up your key now!"
  info "  cp $KEY_FILE ~/contextbox-key-backup.txt"
  echo
  confirm "I have backed up my key (or will do it now)" || true
}

# ─────────────────────────────────────────────
#  Step 6: Config
# ─────────────────────────────────────────────
step_config() {
  echo -e "\n${BOLD}── Step 6: Configuration ─────────────────────${RESET}"
  mkdir -p "$CONFIG_DIR"

  # API key
  local api_key=""
  if [[ -f "$ENV_FILE" ]] && grep -q "API_KEY=" "$ENV_FILE" 2>/dev/null; then
    api_key=$(grep "^API_KEY=" "$ENV_FILE" | cut -d= -f2-)
    ok "Existing API key found"
    if confirm "Generate a new API key?"; then
      api_key=$(openssl rand -base64 32)
      ok "New API key generated"
    fi
  else
    info "Generating API key..."
    api_key=$(openssl rand -base64 32)
    ok "API key generated"
  fi

  # Data dir
  local data_dir
  prompt data_dir "Data directory for documents" "$HOME/contextbox-data"
  mkdir -p "$data_dir"

  # OpenRouter key (optional)
  local openrouter_key=""
  if [[ -f "$ENV_FILE" ]] && grep -q "OPENROUTER_API_KEY=" "$ENV_FILE" 2>/dev/null; then
    openrouter_key=$(grep "^OPENROUTER_API_KEY=" "$ENV_FILE" | cut -d= -f2-)
  fi
  ask "OpenRouter API key (for AI/embeddings, press Enter to skip): "
  read -rp "" or_input
  if [[ -n "$or_input" ]]; then
    openrouter_key="$or_input"
  fi

  # Write .env
  cat > "$ENV_FILE" <<EOF
HOST=127.0.0.1
PORT=8080
API_KEY=${api_key}
DATA_DIR=${data_dir}
OPENROUTER_API_KEY=${openrouter_key}
ENABLE_MCP=true
ENABLE_CLI=true
ENABLE_CHAT=false
ENABLE_WEB_UI=false
LOG_LEVEL=info
EOF
  chmod 600 "$ENV_FILE"
  ok "Config written to $ENV_FILE"

  # Save API key for later steps
  SETUP_API_KEY="$api_key"
  SETUP_DATA_DIR="$data_dir"
}

# ─────────────────────────────────────────────
#  Step 7: Systemd service for ContextBox
# ─────────────────────────────────────────────
step_service() {
  echo -e "\n${BOLD}── Step 7: ContextBox Service ────────────────${RESET}"

  local service_file="/etc/systemd/system/contextbox.service"

  sudo tee "$service_file" > /dev/null <<EOF
[Unit]
Description=ContextBox Document AI Server
After=network.target

[Service]
Type=simple
User=${USER}
EnvironmentFile=${ENV_FILE}
ExecStart=/usr/local/bin/context-box serve
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

  sudo systemctl daemon-reload
  sudo systemctl enable contextbox
  sudo systemctl restart contextbox

  sleep 2
  if systemctl is-active --quiet contextbox; then
    ok "ContextBox service running"
    info "Test: curl -H \"X-API-Key: \$API_KEY\" http://localhost:8080/health"
  else
    err "Service failed to start. Check: journalctl -u contextbox -n 30"
    exit 1
  fi
}

# ─────────────────────────────────────────────
#  Step 8: Cloudflare Tunnel (optional)
# ─────────────────────────────────────────────
step_cloudflare() {
  echo -e "\n${BOLD}── Step 8: Cloudflare Tunnel (optional) ──────${RESET}"

  if ! confirm "Set up Cloudflare Tunnel for remote access?"; then
    info "Skipping Cloudflare Tunnel setup."
    SETUP_DOMAIN=""
    return
  fi

  # Install cloudflared
  if command -v cloudflared &>/dev/null; then
    ok "cloudflared already installed ($(cloudflared --version 2>&1 | head -1))"
  else
    info "Installing cloudflared..."
    curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
      -o /tmp/cloudflared
    sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
    sudo chmod +x /usr/local/bin/cloudflared
    ok "cloudflared installed"
  fi

  # Login
  if [[ ! -f "$HOME/.cloudflared/cert.pem" ]]; then
    info "You need to authenticate with Cloudflare."
    info "A browser window will open (or a URL will be shown to open manually)."
    echo
    cloudflared tunnel login
  else
    ok "Already authenticated with Cloudflare"
  fi

  # Tunnel name + domain
  local tunnel_name domain
  prompt tunnel_name "Tunnel name" "contextbox"
  prompt domain "Your domain/subdomain (e.g. docs.yourdomain.com)" ""

  if [[ -z "$domain" ]]; then
    err "Domain cannot be empty."
    exit 1
  fi

  # Create tunnel (skip if exists)
  if cloudflared tunnel list 2>/dev/null | grep -q "$tunnel_name"; then
    ok "Tunnel '$tunnel_name' already exists"
  else
    cloudflared tunnel create "$tunnel_name"
    ok "Tunnel '$tunnel_name' created"
  fi

  # Get tunnel UUID
  local tunnel_uuid
  tunnel_uuid=$(cloudflared tunnel list --output json 2>/dev/null \
    | grep -o '"id":"[^"]*"' | grep -A1 "\"name\":\"$tunnel_name\"" \
    | head -1 | cut -d'"' -f4 || true)

  if [[ -z "$tunnel_uuid" ]]; then
    # Fallback: find the credentials file
    tunnel_uuid=$(ls "$HOME/.cloudflared/"*.json 2>/dev/null | head -1 | xargs basename | sed 's/.json//' || true)
  fi

  if [[ -z "$tunnel_uuid" ]]; then
    err "Could not determine tunnel UUID. Check: cloudflared tunnel list"
    exit 1
  fi
  info "Tunnel UUID: $tunnel_uuid"

  # Route DNS
  cloudflared tunnel route dns "$tunnel_name" "$domain" || \
    warn "DNS route may already exist — continuing."

  # Write cloudflared config
  sudo mkdir -p /etc/cloudflared
  sudo tee /etc/cloudflared/config.yml > /dev/null <<EOF
tunnel: ${tunnel_name}
credentials-file: ${HOME}/.cloudflared/${tunnel_uuid}.json

ingress:
  - hostname: ${domain}
    service: http://localhost:8080
  - service: http_status:404
EOF
  ok "Cloudflare config written to /etc/cloudflared/config.yml"

  # Systemd service for cloudflared
  sudo tee /etc/systemd/system/cloudflared.service > /dev/null <<EOF
[Unit]
Description=Cloudflare Tunnel
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cloudflared --config /etc/cloudflared/config.yml tunnel run
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

  sudo systemctl daemon-reload
  sudo systemctl enable cloudflared
  sudo systemctl restart cloudflared

  sleep 3
  if systemctl is-active --quiet cloudflared; then
    ok "Cloudflare Tunnel running"
  else
    err "Tunnel failed to start. Check: journalctl -u cloudflared -n 30"
    exit 1
  fi

  SETUP_DOMAIN="$domain"
}

# ─────────────────────────────────────────────
#  Step 9: Firewall (optional)
# ─────────────────────────────────────────────
step_firewall() {
  echo -e "\n${BOLD}── Step 9: Firewall (optional) ───────────────${RESET}"

  if ! command -v ufw &>/dev/null; then
    if confirm "ufw not found. Install it?"; then
      pkg_install ufw
    else
      info "Skipping firewall setup."
      return
    fi
  fi

  # Arch: ufw service must be enabled explicitly
  if [[ "$PKG_MANAGER" == "pacman" ]]; then
    sudo systemctl enable ufw 2>/dev/null || true
  fi

  if ! confirm "Configure UFW firewall (deny all incoming except SSH)?"; then
    info "Skipping firewall setup."
    return
  fi

  sudo ufw --force reset
  sudo ufw default deny incoming
  sudo ufw default allow outgoing
  sudo ufw allow 22/tcp
  sudo ufw --force enable

  ok "Firewall configured (SSH only, all other incoming blocked)"
  sudo ufw status verbose
}

# ─────────────────────────────────────────────
#  Step 10: Summary
# ─────────────────────────────────────────────
step_summary() {
  echo
  echo -e "${GREEN}${BOLD}════════════════════════════════════════════${RESET}"
  echo -e "${GREEN}${BOLD}  ContextBox Setup Complete!${RESET}"
  echo -e "${GREEN}${BOLD}════════════════════════════════════════════${RESET}"
  echo
  echo -e "${BOLD}Config:${RESET}"
  echo    "  Config file : $ENV_FILE"
  echo    "  Encrypt key : $KEY_FILE"
  echo    "  Data dir    : ${SETUP_DATA_DIR:-$HOME/contextbox-data}"
  echo    "  API key     : ${SETUP_API_KEY:-see $ENV_FILE}"
  echo
  echo -e "${BOLD}Services:${RESET}"
  echo    "  contextbox  : $(systemctl is-active contextbox 2>/dev/null || echo unknown)"
  if systemctl is-enabled cloudflared &>/dev/null 2>&1; then
    echo  "  cloudflared : $(systemctl is-active cloudflared 2>/dev/null || echo unknown)"
  fi
  echo
  echo -e "${BOLD}Local test:${RESET}"
  echo    "  curl -H \"X-API-Key: ${SETUP_API_KEY:-<your-api-key>}\" http://localhost:8080/health"
  echo

  if [[ -n "${SETUP_DOMAIN:-}" ]]; then
    echo -e "${BOLD}Remote access:${RESET}"
    echo   "  https://${SETUP_DOMAIN}"
    echo
    echo -e "${BOLD}Add docs remotely (from any machine):${RESET}"
    echo   "  cb remote add \\"
    echo   "    --url https://${SETUP_DOMAIN} \\"
    echo   "    --api-key YOUR_API_KEY \\"
    echo   "    --file mydoc.md"
    echo
    echo -e "${BOLD}MCP config (Claude Desktop / Cursor):${RESET}"
    cat <<EOF
  {
    "mcpServers": {
      "contextbox": {
        "command": "cb",
        "args": ["config", "mcp"],
        "env": {
          "CONTEXTBOX_URL": "https://${SETUP_DOMAIN}",
          "API_KEY": "YOUR_API_KEY"
        }
      }
    }
  }
EOF
  fi

  echo
  echo -e "${BOLD}Useful commands:${RESET}"
  echo    "  systemctl status contextbox       # check server"
  echo    "  journalctl -u contextbox -f       # server logs"
  if systemctl is-enabled cloudflared &>/dev/null 2>&1; then
    echo  "  journalctl -u cloudflared -f      # tunnel logs"
    echo  "  cloudflared tunnel list           # tunnel status"
  fi
  echo    "  cb list                           # list local docs"
  echo    "  cb add --file doc.md              # add local doc"
  echo
}

# ─────────────────────────────────────────────
#  Main
# ─────────────────────────────────────────────
main() {
  print_banner
  require_root_or_sudo

  echo -e "This script will set up ContextBox on this Linux machine."
  echo -e "You can skip optional steps when prompted.\n"

  confirm "Begin setup?" || { echo "Aborted."; exit 0; }

  step_os
  step_deps
  step_rust
  step_build
  step_keygen
  step_config
  step_service
  step_cloudflare
  step_firewall
  step_summary
}

main "$@"
