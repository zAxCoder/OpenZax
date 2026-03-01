# OpenZax CLI - Complete Guide

## Overview

The OpenZax CLI is a comprehensive command-line tool for managing AI skills, models, and MCP servers. Built with Rust for performance and security.

## Architecture

```
openzax-cli/
├── src/
│   ├── main.rs      # Entry point & command routing
│   ├── tui.rs       # Terminal UI (ratatui)
│   └── ui.rs        # UI helpers & formatting
├── Cargo.toml       # Dependencies
└── README.md        # User documentation
```

## Core Features

### 1. Interactive Shell (TUI)

The shell provides a rich terminal interface for interacting with AI models:

```rust
// Features:
- Real-time chat interface
- Message history
- Syntax highlighting
- Keyboard shortcuts
- Session persistence
```

**Usage:**
```bash
openzax shell
openzax shell --model gpt-4 --api-key sk-...
```

### 2. Skill Management

Complete lifecycle management for WebAssembly skills:

#### Initialize
```bash
openzax skill init my-skill --language rust
```

Creates:
- `Cargo.toml` with WASM target
- `src/lib.rs` with skill template
- `manifest.json` with metadata
- `.cargo/config.toml` for build settings
- `README.md` with instructions

#### Build
```bash
openzax skill build --release
```

Compiles to `wasm32-wasip1` target with optimizations:
- LTO enabled
- Strip symbols
- Optimize for size (`opt-level = "z"`)

#### Package
```bash
openzax skill pack
```

Creates `.ozskill` file (ZIP format):
```
skill.ozskill
├── manifest.json    # Metadata
└── skill.wasm       # Compiled WASM
```

#### Sign
```bash
openzax skill sign skill.ozskill --key mykey.private.key
```

Creates `.ozskill.sig` with:
- Ed25519 signature
- SHA-256 hash
- Public key
- Timestamp

#### Publish
```bash
openzax skill publish skill.ozskill --key mykey.private.key
```

Uploads to marketplace with:
- Package verification
- Signature validation
- Authentication check

#### Inspect
```bash
openzax skill inspect skill.ozskill
```

Shows:
- Manifest details
- Permissions
- WASM module info
- File listing

#### Validate
```bash
openzax skill validate skill.ozskill
```

Checks:
- ZIP structure
- manifest.json presence & validity
- skill.wasm presence & WASM magic bytes
- Required fields

### 3. Model Management

Local LLM model management (requires `llm-engine` feature):

```bash
# List models
openzax model list

# Download model
openzax model download llama-3.3-70b

# Show info
openzax model info llama-3.3-70b

# Remove model
openzax model remove llama-3.3-70b --yes
```

Models stored in `~/.openzax/models/`

### 4. MCP Server Tools

Model Context Protocol server utilities:

#### Simulate
```bash
openzax mcp simulate my-server
```

Creates mock MCP server with:
- `echo` tool
- `time` tool
- `mock://readme` resource
- JSON-RPC 2.0 protocol

#### Inspect
```bash
openzax mcp inspect "npx @mcp/server-fs /tmp"
```

Connects to server and lists:
- Server info & version
- Capabilities
- Available tools
- Available resources

#### Record
```bash
openzax mcp record "npx @mcp/server-fs /tmp" --output session.jsonl
```

Records session to JSONL:
```json
{
  "seq": 0,
  "direction": "client→server",
  "timestamp": "2024-01-01T00:00:00Z",
  "content": {...}
}
```

### 5. Authentication & Marketplace

#### Generate Keypair
```bash
openzax keygen --output mykey
```

Creates:
- `mykey.private.key` (Base64 Ed25519 private key)
- `mykey.public.key` (Base64 Ed25519 public key)
- Displays fingerprint (SHA-256 hash)

#### Login
```bash
openzax login --token YOUR_TOKEN
```

Stores token in `~/.openzax/auth.json`

#### Check Status
```bash
openzax whoami
```

Shows:
- Masked token
- Login timestamp

#### Search
```bash
openzax search "file system" --category tools --limit 20
```

Queries marketplace API:
- Full-text search
- Category filtering
- Result limiting

#### Install
```bash
openzax install file-reader --version 1.0.0
```

Downloads and verifies:
- Package download
- Signature verification (if available)
- Installation to `~/.openzax/skills/`

### 6. System Tools

#### Doctor
```bash
openzax doctor
```

Checks:
- Rust/Cargo installation
- wasmtime CLI
- `~/.openzax/` directory structure
- Database accessibility
- Network connectivity to marketplace

#### Upgrade
```bash
openzax upgrade
```

Checks GitHub releases:
- Compares current vs latest version
- Shows release notes
- Provides installation instructions

#### Version
```bash
openzax version
```

Shows:
- CLI version
- Description

## Configuration

### Directory Structure

```
~/.openzax/
├── auth.json              # Authentication token
├── openzax.db             # SQLite database
├── skills/                # Installed skills
│   └── *.ozskill
└── models/                # Local LLM models
    └── *.gguf
```

### Environment Variables

- `OPENZAX_API_KEY` - Default API key for cloud LLMs
- `RUST_LOG` - Logging level (debug, info, warn, error)

## Command Reference

### Global Options

```bash
--verbose, -v    # Enable verbose logging
--help, -h       # Show help
--version, -V    # Show version
```

### Skill Commands

| Command | Description |
|---------|-------------|
| `skill init <name>` | Initialize new skill |
| `skill build` | Build skill to WASM |
| `skill test` | Run skill tests |
| `skill pack` | Package skill to .ozskill |
| `skill sign <package> --key <key>` | Sign package |
| `skill publish <package>` | Publish to marketplace |
| `skill inspect <package>` | Inspect package contents |
| `skill validate <package>` | Validate package structure |

### Model Commands

| Command | Description |
|---------|-------------|
| `model list` | List local models |
| `model download <name>` | Download model |
| `model info <name>` | Show model details |
| `model remove <name>` | Remove model |

### MCP Commands

| Command | Description |
|---------|-------------|
| `mcp simulate <server>` | Start mock MCP server |
| `mcp inspect <server>` | Inspect MCP server |
| `mcp record <server> -o <file>` | Record MCP session |

### Auth Commands

| Command | Description |
|---------|-------------|
| `keygen` | Generate Ed25519 keypair |
| `login --token <token>` | Store auth token |
| `whoami` | Show login status |

### Marketplace Commands

| Command | Description |
|---------|-------------|
| `search <query>` | Search marketplace |
| `install <skill>` | Install skill |

### System Commands

| Command | Description |
|---------|-------------|
| `doctor` | Run health checks |
| `upgrade` | Check for updates |
| `version` | Show version info |

## Development

### Building

```bash
# Debug build
cargo build --manifest-path crates/cli/Cargo.toml

# Release build
cargo build --release --manifest-path crates/cli/Cargo.toml

# With LLM engine feature
cargo build --features llm-engine
```

### Testing

```bash
# Run tests
cargo test --manifest-path crates/cli/Cargo.toml

# With verbose output
cargo test --manifest-path crates/cli/Cargo.toml -- --nocapture
```

### Running

```bash
# Run directly
cargo run --manifest-path crates/cli/Cargo.toml -- <command>

# With verbose logging
RUST_LOG=debug cargo run --manifest-path crates/cli/Cargo.toml -- <command>
```

## Error Handling

The CLI uses `anyhow::Result` for error handling:

```rust
// All commands return Result
async fn handle_command() -> anyhow::Result<()> {
    // Use ? for error propagation
    let data = std::fs::read("file.txt")?;
    
    // Use bail! for custom errors
    if data.is_empty() {
        anyhow::bail!("File is empty");
    }
    
    Ok(())
}
```

## Security

### Skill Signing

1. **Key Generation**: Ed25519 keypair (32-byte private, 32-byte public)
2. **Hashing**: SHA-256 of package contents
3. **Signing**: Ed25519 signature of hash
4. **Verification**: Public key verification on install

### Package Verification

```rust
// Verification flow:
1. Download package
2. Extract signature from .sig file
3. Compute SHA-256 hash of package
4. Verify Ed25519 signature
5. Install if valid, reject if invalid
```

## Performance

### Optimizations

- **Release Profile**: LTO, strip symbols, single codegen unit
- **Async I/O**: Tokio for non-blocking operations
- **Streaming**: Large file handling with streams
- **Caching**: Model metadata caching

### Benchmarks

```
Command execution times (average):
- skill init:     ~50ms
- skill build:    ~5-30s (depends on skill size)
- skill pack:     ~100ms
- skill sign:     ~50ms
- keygen:         ~10ms
- search:         ~200ms (network dependent)
```

## Troubleshooting

### Common Issues

1. **Cargo not found**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WASM target missing**
   ```bash
   rustup target add wasm32-wasip1
   ```

3. **wasmtime not found**
   ```bash
   # Install wasmtime
   curl https://wasmtime.dev/install.sh -sSf | bash
   ```

4. **Permission denied on ~/.openzax**
   ```bash
   chmod 755 ~/.openzax
   ```

5. **Marketplace offline**
   - Check network connection
   - Verify API endpoint: https://api.openzax.dev
   - Check firewall settings

## Future Enhancements

- [ ] Shell completions (bash, zsh, fish)
- [ ] Man pages
- [ ] Interactive debugger UI
- [ ] Plugin system
- [ ] Cloud sync
- [ ] Team collaboration features
- [ ] CI/CD integration
- [ ] Docker support

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0
