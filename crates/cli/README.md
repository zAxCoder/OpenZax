# OpenZax CLI

The command-line interface for OpenZax - a secure AI development assistant platform.

## Installation

### From Source

```bash
cargo install --path crates/cli
```

### From Git

```bash
cargo install --git https://github.com/zAxCoder/OpenZax openzax-cli
```

## Quick Start

```bash
# Start interactive shell
openzax shell

# Or just run
openzax
```

## Commands

### Skill Management

```bash
# Initialize a new skill
openzax skill init my-skill --language rust

# Build a skill
openzax skill build --release

# Test a skill
openzax skill test

# Package a skill
openzax skill pack

# Sign a skill
openzax skill sign my-skill.ozskill --key mykey.private.key

# Publish to marketplace
openzax skill publish my-skill.ozskill --key mykey.private.key

# Inspect a skill package
openzax skill inspect my-skill.ozskill

# Validate a skill package
openzax skill validate my-skill.ozskill
```

### Model Management

```bash
# List local models
openzax model list

# Download a model
openzax model download llama-3.3-70b

# Show model info
openzax model info llama-3.3-70b

# Remove a model
openzax model remove llama-3.3-70b
```

### MCP Server Tools

```bash
# Simulate a mock MCP server
openzax mcp simulate my-server

# Inspect an MCP server
openzax mcp inspect "npx @mcp/server-fs /tmp"

# Record MCP session
openzax mcp record "npx @mcp/server-fs /tmp" --output session.jsonl
```

### Authentication & Marketplace

```bash
# Generate Ed25519 keypair
openzax keygen --output mykey

# Login to marketplace
openzax login --token YOUR_TOKEN

# Check login status
openzax whoami

# Search marketplace
openzax search "file system" --category tools

# Install a skill
openzax install file-reader --version 1.0.0
```

### System Tools

```bash
# Run health checks
openzax doctor

# Check for updates
openzax upgrade

# Show version
openzax version
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

- `OPENZAX_API_KEY` - API key for cloud LLM providers
- `RUST_LOG` - Logging level (e.g., `debug`, `info`)

## Examples

### Create and Publish a Skill

```bash
# 1. Create new skill
openzax skill init hello-world --language rust

# 2. Build it
cd hello-world
openzax skill build --release

# 3. Package it
openzax skill pack

# 4. Generate signing key
openzax keygen --output mykey

# 5. Sign the package
openzax skill sign hello-world-0.1.0.ozskill --key mykey.private.key

# 6. Publish to marketplace
openzax skill publish hello-world-0.1.0.ozskill --key mykey.private.key
```

### Interactive Shell

```bash
# Start with default settings
openzax shell

# Use specific model
openzax shell --model gpt-4

# Use custom API key
openzax shell --api-key sk-...

# Custom database path
openzax shell --db-path ./my-data.db
```

## Features

- 🦀 **Rust-native** - Fast and memory-safe
- 🔐 **Secure** - Ed25519 signing for skills
- 🎨 **Beautiful TUI** - Rich terminal interface with ratatui
- 🔌 **MCP Support** - Model Context Protocol integration
- 📦 **Skill Marketplace** - Discover and share skills
- 🤖 **Local & Cloud LLMs** - Support for both local and cloud models
- 🛡️ **WASM Sandbox** - Secure skill execution

## Development

```bash
# Run in development mode
cargo run --manifest-path crates/cli/Cargo.toml

# With verbose logging
cargo run --manifest-path crates/cli/Cargo.toml -- --verbose

# Run specific command
cargo run --manifest-path crates/cli/Cargo.toml -- skill init test-skill
```

## License

MIT License - see LICENSE file for details
