# OpenZax Installation Guide

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.sh | bash
```

### Windows (PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex
```

## Alternative Installation Methods

### 1. Using Cargo (Rust Package Manager)

If you have Rust installed:

```bash
cargo install --git https://github.com/zAxCoder/OpenZax openzax-cli
```

### 2. From Source

```bash
# Clone the repository
git clone https://github.com/zAxCoder/OpenZax.git
cd OpenZax

# Build and install
cargo install --path crates/cli

# Or build manually
cargo build --release --manifest-path crates/cli/Cargo.toml
sudo cp target/release/openzax /usr/local/bin/  # Linux/macOS
# Or add to PATH on Windows
```

### 3. Download Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/zAxCoder/OpenZax/releases):

- **Linux x86_64**: `openzax-linux-x86_64`
- **macOS x86_64**: `openzax-macos-x86_64`
- **macOS ARM64**: `openzax-macos-aarch64`
- **Windows x86_64**: `openzax-windows-x86_64.exe`

Then:

```bash
# Linux/macOS
chmod +x openzax-*
sudo mv openzax-* /usr/local/bin/openzax

# Windows
# Move to a directory in your PATH
```

## Verify Installation

```bash
openzax --version
openzax doctor
```

## Configuration

OpenZax stores configuration in `~/.openzax/`:

```
~/.openzax/
├── auth.json          # Authentication token
├── skills/            # Installed skills
├── models/            # Local LLM models
└── openzax.db         # SQLite database
```

## Environment Variables

- `OPENZAX_API_KEY` - API key for cloud LLM providers (OpenAI, Anthropic, etc.)
- `RUST_LOG` - Logging level (debug, info, warn, error)

## First Run

```bash
# Start interactive shell
openzax

# Or with specific model
openzax shell --model gpt-4

# With API key
openzax shell --api-key sk-...
```

## Updating

### Using the installer

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.sh | bash

# Windows
iwr -useb https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex
```

### Using Cargo

```bash
cargo install --git https://github.com/zAxCoder/OpenZax openzax-cli --force
```

### Check for updates

```bash
openzax upgrade
```

## Uninstallation

### If installed via script

```bash
# Linux/macOS
rm -rf ~/.openzax
rm ~/.cargo/bin/openzax  # or /usr/local/bin/openzax

# Windows
Remove-Item -Recurse -Force $env:USERPROFILE\.openzax
Remove-Item $env:USERPROFILE\.openzax\bin\openzax.exe
```

### If installed via Cargo

```bash
cargo uninstall openzax-cli
rm -rf ~/.openzax
```

## Troubleshooting

### Command not found

Make sure `~/.openzax/bin` (or `~/.cargo/bin`) is in your PATH:

```bash
# Linux/macOS - Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.openzax/bin"

# Windows - Add to PATH via System Properties
```

### Rust not installed

Install Rust from [rustup.rs](https://rustup.rs/):

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download and run rustup-init.exe from rustup.rs
```

### Build errors

Make sure you have the latest Rust version:

```bash
rustup update
```

### Permission denied

```bash
# Linux/macOS
chmod +x ~/.openzax/bin/openzax

# Or use sudo for system-wide install
sudo cargo install --git https://github.com/zAxCoder/OpenZax openzax-cli
```

## Platform-Specific Notes

### Linux

- Requires `gcc` or `clang` for building
- Install build tools: `sudo apt install build-essential` (Ubuntu/Debian)

### macOS

- Requires Xcode Command Line Tools: `xcode-select --install`

### Windows

- Requires Visual Studio Build Tools or MinGW
- Install from: https://visualstudio.microsoft.com/downloads/

## Getting Help

- Documentation: [GitHub Repository](https://github.com/zAxCoder/OpenZax)
- Issues: [GitHub Issues](https://github.com/zAxCoder/OpenZax/issues)
- Run: `openzax --help` or `openzax doctor`

## Next Steps

After installation:

1. **Generate a keypair** for signing skills:
   ```bash
   openzax keygen --output mykey
   ```

2. **Create your first skill**:
   ```bash
   openzax skill init my-skill
   ```

3. **Explore the marketplace**:
   ```bash
   openzax search "tools"
   ```

4. **Start the interactive shell**:
   ```bash
   openzax
   ```

Enjoy using OpenZax! 🚀
