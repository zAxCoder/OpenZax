# OpenZax CLI

Command-line interface for OpenZax - Secure AI Development Assistant.

## Installation

```bash
cargo install --path crates/cli
```

Or build from source:

```bash
cargo build --release --features llm-engine
```

The binary will be available at `target/release/openzax`.

## Commands

### Shell

Start an interactive terminal shell with AI assistant:

```bash
openzax shell --api-key YOUR_API_KEY
```

Options:
- `--api-key, -a` - API key for LLM provider (or set `OPENZAX_API_KEY` env var)
- `--model, -m` - Model to use (default: `gpt-4`)
- `--db-path, -d` - Database path (default: `.openzax/openzax.db`)

Example:
```bash
export OPENZAX_API_KEY="sk-..."
openzax shell --model gpt-4-turbo
```

### Model Management

Manage local GGUF models for offline inference.

#### List Models

List all available models in the models directory:

```bash
openzax model list
```

Options:
- `--models-dir, -m` - Models directory (default: `~/.openzax/models`)

Example output:
```
Discovering models in: /home/user/.openzax/models

Found 2 model(s):

  • llama 3.3 70b q4 k m (llama-3.3-70b-q4_k_m)
    Size: 38.76 GB
    Quantization: q4_k_m
    Context: 4096 tokens
    Capabilities: [Chat, Code]
    Path: /home/user/.openzax/models/llama-3.3-70b-q4_k_m.gguf
```

#### Download Models

Get instructions for downloading models:

```bash
openzax model download llama-3.3-70b-q4_k_m
```

This command provides instructions and links to download GGUF models from:
- Hugging Face: https://huggingface.co/models?library=gguf
- TheBloke's models: https://huggingface.co/TheBloke

Popular models:
- `llama-3.3-70b-q4_k_m.gguf` - Recommended for general use
- `mistral-7b-instruct-v0.2-q4_k_m.gguf` - Smaller, faster
- `codellama-13b-instruct-q4_k_m.gguf` - Optimized for code

#### Model Info

Show detailed information about a specific model:

```bash
openzax model info llama-3.3-70b-q4_k_m
```

Options:
- `--models-dir, -m` - Models directory (default: `~/.openzax/models`)

Example output:
```
Model Information:
  ID: llama-3.3-70b-q4_k_m
  Name: llama 3.3 70b q4 k m
  Provider: Local
  Context Window: 4096 tokens
  Size: 38.76 GB (39680 MB)
  Quantization: q4_k_m
  Capabilities: [Chat, Code]
  Local: true
  Path: /home/user/.openzax/models/llama-3.3-70b-q4_k_m.gguf

GPU Information:
  CUDA: Not available
  Metal: Available
  Vulkan: Not available
```

#### Remove Model

Remove a model from the local directory:

```bash
openzax model remove llama-3.3-70b-q4_k_m
```

Options:
- `--models-dir, -m` - Models directory (default: `~/.openzax/models`)
- `--yes, -y` - Skip confirmation prompt

Example:
```bash
# With confirmation
openzax model remove llama-3.3-70b-q4_k_m

# Skip confirmation
openzax model remove llama-3.3-70b-q4_k_m -y
```

### Skill Development

Initialize a new skill project (coming in Phase 2):

```bash
openzax init my-skill --language rust
```

Options:
- `--language, -l` - Programming language (rust, typescript, python)

### Version

Display version information:

```bash
openzax version
```

Or simply:
```bash
openzax --version
```

## Global Options

- `--verbose, -v` - Enable verbose logging
- `--help, -h` - Show help information

## Environment Variables

- `OPENZAX_API_KEY` - Default API key for LLM providers
- `RUST_LOG` - Control log level (e.g., `RUST_LOG=debug`)

## Features

The CLI supports optional features that can be enabled at compile time:

### llm-engine

Enables local model management commands:

```bash
cargo build --features llm-engine
```

### llama-cpp

Enables full llama.cpp integration for local inference:

```bash
cargo build --features llama-cpp
```

Note: This requires llama.cpp to be installed on your system.

## Examples

### Basic Usage

```bash
# Start interactive shell
export OPENZAX_API_KEY="sk-..."
openzax shell

# List available models
openzax model list

# Get model information
openzax model info llama-3.3-70b-q4_k_m

# Remove a model
openzax model remove old-model -y
```

### Advanced Usage

```bash
# Use custom models directory
openzax model list --models-dir /path/to/models

# Use specific model in shell
openzax shell --model gpt-4-turbo --db-path ./my-project/.openzax/db

# Verbose logging
openzax -v shell
```

## Directory Structure

OpenZax uses the following directory structure:

```
~/.openzax/
├── models/           # Local GGUF models
├── skills/           # Installed skills (Phase 2)
├── config.toml       # User configuration (Phase 2)
└── openzax.db        # Default database (if not specified)
```

## Troubleshooting

### "API key is required" Error

Make sure to set your API key:

```bash
export OPENZAX_API_KEY="your-api-key"
```

Or pass it directly:

```bash
openzax shell --api-key your-api-key
```

### "Model management requires the 'llm-engine' feature" Error

Rebuild with the llm-engine feature:

```bash
cargo build --release --features llm-engine
```

### No Models Found

Download GGUF models and place them in `~/.openzax/models/`:

```bash
mkdir -p ~/.openzax/models
# Download models from Hugging Face
# Place .gguf files in the directory
openzax model list
```

## Development

### Building

```bash
# Standard build
cargo build

# Release build with all features
cargo build --release --all-features

# Development build with verbose output
cargo build --features llm-engine
```

### Testing

```bash
cargo test
```

### Running

```bash
# Development
cargo run -- shell --api-key test

# Release
cargo run --release --features llm-engine -- model list
```

## License

Dual-licensed under MIT OR Apache-2.0.

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines.

## Documentation

- [Project README](../../README.md)
- [Architecture Blueprint](../../docs/master-architecture-blueprint.md)
- [LLM Engine Guide](../../docs/llm-engine-guide.md)
- [MCP Client Guide](../../docs/mcp-client-guide.md)
- [WASM Runtime Guide](../../docs/wasm-runtime-guide.md)

## Support

- GitHub Issues: https://github.com/openzax/openzax/issues
- Documentation: https://docs.openzax.dev (coming soon)
