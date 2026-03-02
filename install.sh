#!/bin/bash
# OpenZax Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/zAxCoder/OpenZax/main/install.sh | bash
set -euo pipefail

REPO="zAxCoder/OpenZax"
INSTALL_DIR="$HOME/.openzax/bin"
BINARY_NAME="openzax"

echo ""
echo "  ╔══════════════════════════════════╗"
echo "  ║     OPENZAX INSTALLER            ║"
echo "  ╚══════════════════════════════════╝"
echo ""

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)   TARGET_OS="linux" ;;
    darwin)  TARGET_OS="macos" ;;
    *)       echo "  [!] Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)  TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *)             echo "  [!] Unsupported arch: $ARCH"; exit 1 ;;
esac

echo "  [*] Detected: ${TARGET_OS} ${TARGET_ARCH}"

# Get latest release tag
echo "  [*] Finding latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "  [!] Could not find latest release."
    echo "  [*] Building from source instead..."
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "  [!] Rust not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    TMPDIR=$(mktemp -d)
    echo "  [*] Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$TMPDIR/openzax"
    cd "$TMPDIR/openzax"
    echo "  [*] Building (this may take a few minutes)..."
    cargo build -p openzax-cli --release
    mkdir -p "$INSTALL_DIR"
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    rm -rf "$TMPDIR"
else
    echo "  [*] Latest: $LATEST"
    
    # Download binary
    ASSET_NAME="openzax-${TARGET_OS}-${TARGET_ARCH}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET_NAME}"
    
    echo "  [*] Downloading ${ASSET_NAME}..."
    TMPDIR=$(mktemp -d)
    
    if curl -fsSL -o "$TMPDIR/$ASSET_NAME" "$DOWNLOAD_URL" 2>/dev/null; then
        echo "  [OK] Downloaded"
        tar -xzf "$TMPDIR/$ASSET_NAME" -C "$TMPDIR"
        mkdir -p "$INSTALL_DIR"
        cp "$TMPDIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        rm -rf "$TMPDIR"
    else
        echo "  [!] Pre-built binary not available for your platform."
        echo "  [*] Building from source..."
        
        if ! command -v cargo &> /dev/null; then
            echo "  [!] Rust not found. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            exit 1
        fi
        
        git clone --depth 1 "https://github.com/${REPO}.git" "$TMPDIR/openzax"
        cd "$TMPDIR/openzax"
        echo "  [*] Building (this may take a few minutes)..."
        cargo build -p openzax-cli --release
        mkdir -p "$INSTALL_DIR"
        cp "target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        rm -rf "$TMPDIR"
    fi
fi

echo "  [OK] Installed to $INSTALL_DIR/$BINARY_NAME"

# Add to PATH
SHELL_NAME="$(basename "$SHELL")"
PROFILE=""
case "$SHELL_NAME" in
    bash) PROFILE="$HOME/.bashrc" ;;
    zsh)  PROFILE="$HOME/.zshrc" ;;
    fish) PROFILE="$HOME/.config/fish/config.fish" ;;
    *)    PROFILE="$HOME/.profile" ;;
esac

if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    if [ "$SHELL_NAME" = "fish" ]; then
        echo "set -gx PATH \$PATH $INSTALL_DIR" >> "$PROFILE"
    else
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$PROFILE"
    fi
    echo "  [OK] Added to PATH in $PROFILE"
    echo ""
    echo "  >>> Restart your terminal or run: source $PROFILE"
else
    echo "  [OK] Already in PATH"
fi

echo ""
echo "  Installation complete!"
echo ""
echo "  Usage:"
echo "    openzax                   Open the TUI"
echo "    openzax --help            Show all commands"
echo "    openzax doctor            Check system health"
echo ""
echo "  Free API keys (no credit card):"
echo "    OpenRouter:  https://openrouter.ai/keys"
echo "    Groq:        https://console.groq.com"
echo "    Cerebras:    https://cloud.cerebras.ai"
echo ""
echo "  Set your key and run:"
echo "    export OPENROUTER_API_KEY=sk-or-v1-..."
echo "    openzax"
echo ""
