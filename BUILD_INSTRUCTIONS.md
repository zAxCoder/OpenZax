# OpenZax - Build Instructions

Complete guide for building and running OpenZax desktop application.

---

## Prerequisites

### Required Software

1. **Rust 1.82+**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Add WASM target
   rustup target add wasm32-unknown-unknown
   rustup target add wasm32-wasi
   ```

2. **Node.js 18+**
   ```bash
   # Download from https://nodejs.org/
   # Or use nvm:
   nvm install 18
   nvm use 18
   ```

3. **Trunk** (for building Leptos UI)
   ```bash
   cargo install trunk
   ```

4. **Tauri CLI**
   ```bash
   npm install -g @tauri-apps/cli
   ```

### Platform-Specific Requirements

#### Windows
- Visual Studio 2019 or later with C++ build tools
- WebView2 (usually pre-installed on Windows 10/11)

#### macOS
- Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

---

## Building the Project

### 1. Clone Repository

```bash
git clone https://github.com/openzax/openzax.git
cd openzax
```

### 2. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Verify Rust installation
cargo --version
rustc --version
```

### 3. Build Options

#### Option A: Development Build (Recommended for testing)

```bash
# This will:
# 1. Build the Leptos UI (WASM)
# 2. Start Tauri in development mode
# 3. Enable hot-reload for UI changes
npm run tauri:dev
```

The application will open automatically. Changes to UI code will hot-reload.

#### Option B: Production Build

```bash
# Build optimized production bundle
npm run tauri:build
```

Output locations:
- **Windows**: `tauri-app/target/release/bundle/msi/OpenZax_0.5.0_x64_en-US.msi`
- **macOS**: `tauri-app/target/release/bundle/dmg/OpenZax_0.5.0_universal.dmg`
- **Linux**: `tauri-app/target/release/bundle/appimage/openzax_0.5.0_amd64.AppImage`

#### Option C: CLI Only (No GUI)

```bash
# Build CLI with model management
cargo build --release --features llm-engine

# Run terminal shell
./target/release/openzax shell --api-key YOUR_API_KEY
```

---

## Development Workflow

### Running UI and Backend Separately

For faster development iteration:

**Terminal 1: Build and serve UI**
```bash
npm run dev
# or
trunk serve --port 5173
```

**Terminal 2: Run Tauri backend**
```bash
cd tauri-app
cargo tauri dev
```

### Building Individual Crates

```bash
# Build core engine
cargo build -p openzax-core

# Build WASM runtime
cargo build -p openzax-wasm-runtime

# Build MCP client
cargo build -p openzax-mcp-client

# Build LLM engine
cargo build -p openzax-llm-engine --features llama-cpp

# Build CLI
cargo build -p openzax-cli --features llm-engine
```

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run tests for specific crate
cargo test -p openzax-core

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test '*'
```

### Code Quality Checks

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features

# Check for security vulnerabilities
cargo audit

# Check dependencies
cargo deny check
```

---

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# Required: OpenAI API key
OPENZAX_API_KEY=sk-...

# Optional: Custom models directory
OPENZAX_MODELS_DIR=~/.openzax/models

# Optional: Database path
OPENZAX_DB_PATH=~/.openzax/openzax.db

# Optional: Log level
RUST_LOG=openzax=info,tauri=info
```

### Tauri Configuration

Edit `tauri-app/tauri.conf.json` for:
- Window size and position
- Application name and identifier
- Bundle settings
- Security policies

### UI Configuration

Edit `Trunk.toml` for:
- Build output directory
- Development server port
- Asset handling

---

## Troubleshooting

### Common Issues

#### 1. "cargo: command not found"
```bash
# Add Rust to PATH
source $HOME/.cargo/env
```

#### 2. "trunk: command not found"
```bash
# Install Trunk
cargo install trunk --locked
```

#### 3. "WebView2 not found" (Windows)
Download and install from: https://developer.microsoft.com/microsoft-edge/webview2/

#### 4. "failed to run custom build command for `tauri`"
```bash
# Install platform-specific dependencies (see Prerequisites)
# On Linux:
sudo apt install libwebkit2gtk-4.1-dev
```

#### 5. WASM build fails
```bash
# Ensure WASM target is installed
rustup target add wasm32-unknown-unknown

# Clear cache and rebuild
cargo clean
trunk clean
npm run tauri:dev
```

#### 6. "error: linker `cc` not found"
```bash
# Install build tools
# Ubuntu/Debian:
sudo apt install build-essential

# macOS:
xcode-select --install
```

### Debug Mode

Enable verbose logging:

```bash
# Set log level
export RUST_LOG=debug

# Run with backtrace
RUST_BACKTRACE=1 npm run tauri:dev
```

---

## Performance Optimization

### Release Build Optimizations

The `Cargo.toml` includes optimizations:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower compile
strip = true         # Remove debug symbols
```

### Reducing Binary Size

```bash
# Install cargo-bloat to analyze size
cargo install cargo-bloat

# Check what's taking space
cargo bloat --release -n 10

# Use wasm-opt for UI
wasm-opt -O3 dist/*.wasm -o dist/optimized.wasm
```

### Faster Compilation

```toml
# Add to ~/.cargo/config.toml
[build]
jobs = 8  # Adjust based on CPU cores

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

---

## Cross-Compilation

### Building for Different Platforms

#### From Linux to Windows
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install mingw
sudo apt install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

#### From macOS to Linux
```bash
# Install target
rustup target add x86_64-unknown-linux-gnu

# Build (requires Docker)
docker run --rm -v "$(pwd)":/app -w /app rust:latest \
  cargo build --release --target x86_64-unknown-linux-gnu
```

---

## CI/CD Integration

### GitHub Actions

See `.github/workflows/ci.yml` for automated builds.

### Local CI Simulation

```bash
# Run the same checks as CI
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

---

## Distribution

### Creating Installers

#### Windows (.msi)
```bash
npm run tauri:build
# Output: tauri-app/target/release/bundle/msi/
```

#### macOS (.dmg)
```bash
npm run tauri:build
# Output: tauri-app/target/release/bundle/dmg/
# Note: Requires code signing for distribution
```

#### Linux (.AppImage, .deb, .rpm)
```bash
npm run tauri:build
# Output: tauri-app/target/release/bundle/
```

### Code Signing

#### macOS
```bash
# Sign the app
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  tauri-app/target/release/bundle/macos/OpenZax.app

# Notarize
xcrun notarytool submit OpenZax.dmg \
  --apple-id your@email.com \
  --password app-specific-password \
  --team-id TEAM_ID
```

#### Windows
```bash
# Use signtool.exe from Windows SDK
signtool sign /f certificate.pfx /p password \
  /t http://timestamp.digicert.com \
  OpenZax.msi
```

---

## Development Tips

### Hot Reload

UI changes hot-reload automatically in dev mode. For backend changes:

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x 'run --bin openzax'
```

### Debugging

#### Rust Backend
```bash
# Use rust-lldb or rust-gdb
rust-lldb target/debug/openzax

# Or use VS Code with rust-analyzer extension
```

#### UI (Browser DevTools)
- Right-click in app → "Inspect Element"
- Or press `F12` in development mode

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile the application
cargo flamegraph --bin openzax
```

---

## Additional Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [WebAssembly](https://webassembly.org/)

---

## Support

If you encounter issues:

1. Check [GitHub Issues](https://github.com/openzax/openzax/issues)
2. Read [Troubleshooting](#troubleshooting) section
3. Ask in [GitHub Discussions](https://github.com/openzax/openzax/discussions)

---

**Last Updated**: 2026-03-01  
**Version**: 0.5.0
