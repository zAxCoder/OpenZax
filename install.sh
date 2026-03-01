#!/usr/bin/env bash
set -e

# OpenZax Installer
# Usage: curl -fsSL https://openzax.dev/install | bash

REPO="zAxCoder/OpenZax"
BINARY_NAME="openzax"
INSTALL_DIR="$HOME/.openzax/bin"
GITHUB_API="https://api.github.com/repos/$REPO/releases/latest"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Detect OS and Architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        msys*|mingw*|cygwin*)
            OS="windows"
            ;;
        *)
            echo -e "${RED}Unsupported OS: $os${NC}"
            exit 1
            ;;
    esac
    
    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${CYAN}Detected platform: ${BLUE}$OS-$ARCH${NC}"
}

# Check if Rust is installed
check_rust() {
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}✓${NC} Rust is installed"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} Rust is not installed"
        return 1
    fi
}

# Install Rust
install_rust() {
    echo -e "${CYAN}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓${NC} Rust installed successfully"
}

# Install from source
install_from_source() {
    echo -e "${CYAN}Installing OpenZax from source...${NC}"
    
    # Create temp directory
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Clone repository
    echo -e "${CYAN}Cloning repository...${NC}"
    git clone "https://github.com/$REPO.git" openzax
    cd openzax
    
    # Build release
    echo -e "${CYAN}Building OpenZax (this may take a few minutes)...${NC}"
    cargo build --release --manifest-path crates/cli/Cargo.toml
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary
    cp target/release/$BINARY_NAME "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Cleanup
    cd ~
    rm -rf "$TEMP_DIR"
    
    echo -e "${GREEN}✓${NC} OpenZax installed to $INSTALL_DIR/$BINARY_NAME"
}

# Download pre-built binary (if available)
install_binary() {
    echo -e "${CYAN}Checking for pre-built binaries...${NC}"
    
    # Get latest release info
    RELEASE_INFO=$(curl -s "$GITHUB_API")
    DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep "browser_download_url.*$OS.*$ARCH" | cut -d '"' -f 4 | head -n 1)
    
    if [ -z "$DOWNLOAD_URL" ]; then
        echo -e "${YELLOW}⚠${NC} No pre-built binary found for $OS-$ARCH"
        return 1
    fi
    
    echo -e "${CYAN}Downloading OpenZax...${NC}"
    mkdir -p "$INSTALL_DIR"
    
    if [ "$OS" = "windows" ]; then
        curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BINARY_NAME.exe"
        chmod +x "$INSTALL_DIR/$BINARY_NAME.exe"
    else
        curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    echo -e "${GREEN}✓${NC} OpenZax downloaded successfully"
    return 0
}

# Add to PATH
setup_path() {
    local shell_config=""
    
    # Detect shell
    if [ -n "$BASH_VERSION" ]; then
        shell_config="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        shell_config="$HOME/.zshrc"
    else
        shell_config="$HOME/.profile"
    fi
    
    # Check if already in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo -e "${GREEN}✓${NC} $INSTALL_DIR is already in PATH"
        return
    fi
    
    # Add to shell config
    echo "" >> "$shell_config"
    echo "# OpenZax" >> "$shell_config"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_config"
    
    echo -e "${GREEN}✓${NC} Added $INSTALL_DIR to PATH in $shell_config"
    echo -e "${YELLOW}⚠${NC} Please restart your terminal or run: ${CYAN}source $shell_config${NC}"
}

# Create config directory
setup_config() {
    local config_dir="$HOME/.openzax"
    mkdir -p "$config_dir/skills"
    mkdir -p "$config_dir/models"
    
    echo -e "${GREEN}✓${NC} Created config directory at $config_dir"
}

# Print success message
print_success() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                        ║${NC}"
    echo -e "${GREEN}║  ${CYAN}OpenZax installed successfully!${GREEN}                    ║${NC}"
    echo -e "${GREEN}║                                                        ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${CYAN}Quick Start:${NC}"
    echo -e "  ${BLUE}1.${NC} Restart your terminal or run: ${CYAN}source ~/.bashrc${NC}"
    echo -e "  ${BLUE}2.${NC} Run: ${CYAN}openzax${NC}"
    echo -e "  ${BLUE}3.${NC} Or try: ${CYAN}openzax --help${NC}"
    echo ""
    echo -e "${CYAN}Documentation:${NC} ${BLUE}https://github.com/$REPO${NC}"
    echo ""
}

# Main installation flow
main() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                        ║${NC}"
    echo -e "${CYAN}║  ${BLUE}OpenZax Installer${CYAN}                                  ║${NC}"
    echo -e "${CYAN}║  ${NC}Secure AI Development Assistant                   ${CYAN}║${NC}"
    echo -e "${CYAN}║                                                        ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Detect platform
    detect_platform
    
    # Check for Rust
    if ! check_rust; then
        read -p "$(echo -e ${YELLOW}Install Rust? [Y/n]:${NC} )" -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
            install_rust
        else
            echo -e "${RED}✗${NC} Rust is required to build OpenZax"
            exit 1
        fi
    fi
    
    # Try to download binary, fallback to source
    if ! install_binary; then
        echo -e "${CYAN}Building from source...${NC}"
        install_from_source
    fi
    
    # Setup
    setup_config
    setup_path
    
    # Success
    print_success
}

# Run installer
main
