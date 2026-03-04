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
  A terminal-native coding agent with zero-trust security, multi-agent orchestration, and 45+ free AI models.
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#features">Features</a> ·
  <a href="#models">Models</a> ·
  <a href="#multi-agent">Multi-Agent</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#about">About</a>
</p>

---

## About

OpenZax was built by **zAx** — an Egyptian developer who believes powerful AI tools should be free and accessible to everyone. The entire project was developed by zAx with AI assistance, proving that a single determined developer can build production-grade software with the right tools and vision.

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
| **45+ Free Models** | OpenRouter, Groq, Cerebras — text, vision, image, embedding |
| **Multi-Agent** | Intelligent model routing — spawns sub-agents with optimal model per task |
| **Build / Plan / Agent** | Three modes: code generation, architecture planning, multi-agent orchestration |
| **Smart Model Routing** | Coordinator assigns the best model for each sub-task automatically |
| **WASM Skills** | Sandboxed WebAssembly plugins with capability-based security |
| **MCP Client** | Model Context Protocol support for tool integration |
| **Auto Updates** | Built-in update check with one-command upgrade |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch between Build, Plan, and Multi-Agent mode |
| `Ctrl+T` | Cycle intelligence tier (high / max / auto) |
| `Ctrl+P` | Open command palette |
| `Ctrl+M` | Switch model |
| `Ctrl+K` | Browse skills |
| `Ctrl+N` | New session |
| `Ctrl+C` | Exit |

## Models

OpenZax ships with 45+ free models — no credit card required:

| Tier | Models | Strengths |
|------|--------|-----------|
| **Elite** | Qwen3 235B Think, GPT-OSS 120B, DeepSeek R1 | Deep reasoning, complex code |
| **Strong** | Qwen3 Coder, Llama 3.3 70B, Hermes 3 405B | Coding, general intelligence |
| **Vision** | Qwen3 VL 4B, Nemotron Nano VL 12B | Image analysis, visual tasks |
| **Image** | Flux.1 Schnell, RiverFlow V2 Max | Image generation |
| **Fast** | Groq Llama 70B, Cerebras Qwen 32B | Ultra-low latency inference |

## Multi-Agent

In Multi-Agent mode, OpenZax coordinates multiple AI agents simultaneously:

- The coordinator analyzes your request and breaks it into sub-tasks
- Each sub-agent gets assigned the optimal model for its specific task
- Models are automatically de-duplicated — no two agents run the same model
- Rate-limited requests are retried automatically
- The sidebar shows all active agents with their assigned models and tasks

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
├── openzax-llm-engine   Local model management (GGUF)
├── openzax-marketplace  Skill marketplace REST API
├── openzax-workflow      Workflow engine (DAG execution)
├── openzax-enterprise   SSO, RBAC, fleet management
└── openzax-sdk          SDK + test harness
```

## CLI Commands

```
openzax                    Launch the TUI
openzax shell              Launch with options (--api-key, --model)
openzax upgrade            Update to latest version
openzax doctor             System health check
openzax keygen             Generate Ed25519 keypair
openzax skill init <name>  Create a new skill project
openzax skill build        Build skill to WASM
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
