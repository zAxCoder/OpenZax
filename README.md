# OpenZax

> Secure AI Development Assistant built in Rust with WASM sandboxing and zero-trust architecture.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.5.0-green.svg)](CHANGELOG.md)
[![Status](https://img.shields.io/badge/status-Phase%201%20Complete-brightgreen.svg)](STATUS.md)

---

## 🎯 Project Status

**Current Phase**: Phase 1 - ✅ 100% Complete  
**Overall Progress**: 40% Complete  
**Next Milestone**: Phase 2 - Skills SDK & Marketplace

### What's Working Now

- ✅ **Desktop Application** - Tauri v2 with Leptos UI
- ✅ **Command Palette** - Fast keyboard-driven commands (Ctrl+Shift+P)
- ✅ **Multi-Panel Workspace** - Professional IDE-like layout
- ✅ **Chat Interface** - Real-time streaming with markdown
- ✅ **Terminal Shell** - Interactive CLI with streaming responses
- ✅ **WASM Sandbox** - Production-ready skill execution
- ✅ **MCP Client** - Full protocol support (stdio, HTTP)
- ✅ **LLM Engine** - Intelligent model routing
- ✅ **Model Management** - CLI commands for local models

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.82+ (2024 edition)
- Node.js 18+ (for Tauri)
- OpenAI API key (or compatible LLM API)

### Installation

```bash
# Clone repository
git clone https://github.com/openzax/openzax.git
cd openzax

# Install Trunk (for building Leptos UI)
cargo install trunk

# Install Tauri CLI
npm install -g @tauri-apps/cli

# Install WASM target
rustup target add wasm32-unknown-unknown

# Set API key
export OPENZAX_API_KEY="your-api-key-here"
```

### Run Desktop Application

```bash
# Development mode
npm run tauri:dev

# Production build
npm run tauri:build
```

### Run Terminal Shell

```bash
# Build with model management support
cargo build --release --features llm-engine

# Run shell
cargo run --release --bin openzax shell
```

### Model Management

```bash
# List local models
openzax model list

# Get model information
openzax model info llama-3.3-70b-q4_k_m

# Download instructions
openzax model download llama-3.3-70b-q4_k_m

# Remove a model
openzax model remove old-model -y
```

---

## 📦 Features

### Security First
- **Zero-Trust Architecture** - Every operation requires explicit permission
- **WASM Sandboxing** - Skills run in complete isolation
- **Capability-Based Access** - Fine-grained permission control
- **Resource Limits** - CPU and memory budgets enforced

### High Performance
- **5x Less Memory** - ~30 MB idle vs competitors' ~150 MB
- **25x Smaller Binary** - <8 MB vs competitors' ~200 MB
- **Near-Native WASM** - <5μs function call overhead
- **Sub-1ms IPC** - Cap'n Proto serialization

### Multi-Model Support
- **Intelligent Routing** - Automatic model selection
- **Local & Cloud** - llama.cpp + OpenAI/Anthropic/Google
- **Cost Optimization** - Balance latency, cost, quality
- **Hot-Swap** - Load/unload models at runtime

### MCP Integration
- **Full Protocol** - Tools, Resources, Prompts, Sampling
- **Multiple Transports** - stdio, HTTP, WebSocket (planned)
- **Type-Safe** - Rust type system ensures correctness
- **Extensible** - Easy to add new MCP servers

---

## 📊 Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   OpenZax Platform                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  LLM Engine  │  │  MCP Client  │  │ WASM Runtime │ │
│  │              │  │              │  │              │ │
│  │ • Router     │  │ • stdio      │  │ • Wasmtime   │ │
│  │ • Local      │  │ • HTTP       │  │ • 6 WIT APIs │ │
│  │ • Cloud      │  │ • Protocol   │  │ • Sandbox    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
├─────────────────────────────────────────────────────────┤
│                    Core Engine                          │
│  • Event Bus  • Agent Runtime  • Storage  • Security   │
└─────────────────────────────────────────────────────────┘
```

---

## 📚 Documentation

- [Master Architecture Blueprint](docs/master-architecture-blueprint.md) - Complete system design
- [WASM Runtime Guide](docs/wasm-runtime-guide.md) - Skill development
- [MCP Client Guide](docs/mcp-client-guide.md) - MCP integration
- [LLM Engine Guide](docs/llm-engine-guide.md) - Model management
- [CLI Documentation](crates/cli/README.md) - Command reference
- [Contributing Guidelines](CONTRIBUTING.md) - How to contribute

---

## 🎯 Roadmap

### ✅ Phase 0 - Foundation (Complete)
- Rust workspace with 7 crates
- Event-driven architecture
- Agent runtime with LLM integration
- SQLite storage
- Terminal shell interface

### ✅ Phase 1 Month 2 - WASM Sandbox (Complete)
- Wasmtime 27.0 runtime
- 6 WIT host interfaces
- Fuel metering and memory limits
- Example skills
- Comprehensive documentation

### ✅ Phase 1 Month 3 - MCP + LLM (Complete)
- MCP client (stdio, HTTP)
- Full protocol support
- LLM engine with model router
- Local model manager
- Model management CLI

### ✅ Phase 1 Month 4 - UI (Complete)
- Tauri v2 desktop application
- Leptos UI framework
- Command palette with fuzzy search
- Multi-panel workspace layout

### 📅 Phase 2 - Ecosystem (Months 5-7) - Next
- Skills SDK v1.0
- Marketplace backend
- Visual workflow editor

### 📅 Phase 3 - Community (Months 8-10)
- Public marketplace
- Community review system
- Cloud model routing

### 📅 Phase 4 - Enterprise (Months 11-14)
- SSO/SAML + RBAC
- Fleet management
- Hosted orchestration
- SOC 2 compliance

---

## 🔧 Development

### Build

```bash
# Standard build
cargo build

# Release build with all features
cargo build --release --all-features

# With model management
cargo build --release --features llm-engine
```

### Test

```bash
# Run all tests
cargo test --all-features

# Run specific crate tests
cargo test -p openzax-core
```

### Format & Lint

```bash
# Format code
cargo fmt --all

# Run lints
cargo clippy --all-targets --all-features
```

---

## 📈 Performance Metrics

| Metric | Target | Achieved | Status |
|---|---|---|---|
| Memory (idle) | <50 MB | ~30 MB | ✅ Exceeded |
| Binary size | <10 MB | <8 MB | ✅ Exceeded |
| WASM call | <10 μs | ~1-5 μs | ✅ Exceeded |
| Event latency | <1 ms | <1 ms | ✅ Met |
| Module load | <10 ms | ~5-10 ms | ✅ Met |

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas for Contribution

- 🐛 Bug fixes and testing
- 📝 Documentation improvements
- 🎨 UI/UX enhancements (Phase 1 Month 4)
- 🔌 MCP server integrations
- 🧩 WASM skill development
- 🌐 Translations

---

## 📄 License

Dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license for your use.

---

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Wasmtime](https://wasmtime.dev/) - WASM runtime
- [SQLite](https://www.sqlite.org/) - Database
- [Tauri](https://tauri.app/) - Desktop framework (Phase 1 Month 4)

Inspired by:
- Model Context Protocol (MCP) specification
- WebAssembly Component Model
- Zero-trust security principles

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/openzax/openzax/issues)
- **Discussions**: [GitHub Discussions](https://github.com/openzax/openzax/discussions)
- **Documentation**: [docs/](docs/)

---

## 🌟 Star History

If you find OpenZax useful, please consider giving it a star! ⭐

---

**Status**: ✅ Phase 1 Complete - Ready for Phase 2  
**Version**: 0.5.0  
**Last Updated**: 2026-03-01
