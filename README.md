<p align="center">
  <img src="https://img.shields.io/badge/language-Rust-black?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/license-MIT-white?style=flat-square" />
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-gray?style=flat-square" />
  <img src="https://img.shields.io/github/stars/zAxCoder/OpenZax?style=flat-square&color=black" />
  <img src="https://img.shields.io/github/v/release/zAxCoder/OpenZax?style=flat-square&color=white" />
</p>

<h1 align="center">OpenZax</h1>

<p align="center">
  <strong>Secure AI development assistant built in Rust.</strong><br/>
  A terminal-native coding agent with zero-trust security, WASM skill plugins, and multi-model support.
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#features">Features</a> ·
  <a href="#models">Models</a> ·
  <a href="#skills">Skills</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#contributing">Contributing</a>
</p>

---

## Install

**One command** — then type `openzax` anywhere:

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/zAxCoder/OpenZax/master/install.ps1 | iex
```

Or build from source:

```bash
git clone https://github.com/zAxCoder/OpenZax.git
cd OpenZax
cargo build -p openzax-cli --release
cp target/release/openzax ~/.local/bin/  # or anywhere in PATH
```

## Quick Start

```bash
# Set a free API key (no credit card required)
export OPENROUTER_API_KEY=sk-or-v1-...

# Launch
openzax
```

Get free keys from: [openrouter.ai/keys](https://openrouter.ai/keys) · [console.groq.com](https://console.groq.com) · [cloud.cerebras.ai](https://cloud.cerebras.ai)

## Features

| Feature | Description |
|---------|-------------|
| **Terminal UI** | Monochrome TUI with command palette, model picker, skills browser |
| **Multi-Model** | 12+ free models from OpenRouter, Groq, Cerebras |
| **Build / Plan Modes** | Switch between code generation and architecture planning |
| **WASM Skills** | Sandboxed WebAssembly plugins with capability-based security |
| **MCP Client** | Model Context Protocol support for tool integration |
| **Zero-Trust Security** | Ed25519 signing, encrypted storage, audit logging |
| **Marketplace** | Discover, install, and publish skills |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch between Build and Plan mode |
| `Ctrl+T` | Cycle intelligence tier (high / max / auto) |
| `Ctrl+P` | Open command palette |
| `Ctrl+M` | Switch model |
| `Ctrl+K` | Browse skills |
| `Ctrl+N` | New session |
| `Ctrl+C` | Exit |

## Models

OpenZax works with free models out of the box — no credit card required:

| Provider | Models | Free Tier |
|----------|--------|-----------|
| **OpenRouter** | DeepSeek R1, Qwen3 235B, Llama 3.3 70B, Gemma 3 | Free with key |
| **Groq** | Llama 3.3 70B, Mixtral 8x7B, Gemma 2 9B | 14,400 req/day |
| **Cerebras** | Llama 3.3 70B, Qwen3 32B | 1M tokens/day |

## Skills

Skills are sandboxed WASM plugins that extend OpenZax capabilities:

```bash
openzax skill init my-skill --language rust
openzax skill build --release
openzax skill pack
openzax skill publish my-skill-0.1.0.ozskill --key mykey.private.key
```

Built-in skills include: `webapp-testing`, `frontend-design`, `docker-expert`, `security-audit`, `api-design-patterns`, `database-schema-designer`, and more.

## Architecture

```
OpenZax
├── openzax-cli          Terminal UI + CLI commands
├── openzax-core         Agent engine, event bus, storage
├── openzax-shell        Terminal emulation, process management
├── openzax-ai-core      Multi-model routing, Tree-of-Thought planning
├── openzax-security     Zero-trust capabilities, encrypted storage
├── openzax-wasm-runtime Wasmtime sandbox for WASM skills
├── openzax-mcp-client   Model Context Protocol client
├── openzax-skills-sdk   SDK for building skills
├── openzax-skills-macros Procedural macros for skills
├── openzax-llm-engine   Local model management (GGUF)
├── openzax-marketplace  Skill marketplace REST API
├── openzax-workflow      Workflow engine (DAG execution)
├── openzax-enterprise   SSO, RBAC, fleet management
└── openzax-test-harness Testing framework
```

See [docs/master-architecture-blueprint.md](docs/master-architecture-blueprint.md) for the full blueprint.

## CLI Commands

```
openzax                    Launch the TUI
openzax shell              Launch with options (--api-key, --model)
openzax doctor             System health check
openzax keygen             Generate Ed25519 keypair
openzax skill init <name>  Create a new skill project
openzax skill build        Build skill to WASM
openzax skill pack         Package skill as .ozskill
openzax search <query>     Search the marketplace
openzax install <skill>    Install a skill
openzax version            Show version
```

## Contributing

```bash
git clone https://github.com/zAxCoder/OpenZax.git
cd OpenZax
cargo build
cargo test
```

## License

MIT
