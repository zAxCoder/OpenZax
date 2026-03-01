# OpenZax Master Blueprint — Project Plan

> **Implementation Status (2026-03-01):** Phases 0–4 complete. 15 Rust crates + TypeScript/Python SDKs implemented. Zero compilation errors. Ready for Phase 5 (platform maturity).

---


> **Overview:** Design and write a comprehensive master architecture blueprint for OpenZax — a next-generation autonomous desktop AI operating system that exploits OpenClaw's critical security flaws, Node.js performance ceiling, and compromised marketplace to establish category dominance through a Rust-native, WASM-sandboxed, zero-trust architecture.

---

## Competitive Intelligence Summary

Research reveals OpenClaw has **catastrophic structural weaknesses** that define our attack surface:

- **CVE-2026-25253 (CVSS 8.8):** One-click RCE via agent visiting attacker URL
- **Auth disabled by default** on all deployments; 40,000+ instances publicly exposed
- **12% of ClawHub skills contained malware** (341 of 2,857 skills had keyloggers/credential stealers)
- **Prompt injection is architecturally unfixable** per Anthropic's own admission
- **No real plugin sandboxing** -- skills run with full system access
- **Node.js/Python runtime** -- inherent performance ceiling, high memory usage

OpenZax exploits every one of these weaknesses by design.

---

## Deliverable

A single, production-grade Markdown document saved to [docs/master-architecture-blueprint.md](master-architecture-blueprint.md) containing all 12 sections specified in the requirements, with full technical depth, architecture diagrams (Mermaid), and zero placeholders.

---

## Document Structure (12 Sections)

### 1. Executive War Strategy

- Position OpenZax as "the secure alternative" exploiting OpenClaw's security crisis
- Differentiation matrix: Security, Performance, Extensibility, UX, Ecosystem
- Three-phase ecosystem domination: Capture (security-conscious devs) -> Expand (enterprise) -> Dominate (marketplace network effects)

### 2. Ultra-Advanced Technical Architecture

Core technology stack:

- **Runtime:** Rust (core engine, agent runtime, security kernel)
- **Desktop Shell:** Tauri v2 (native WebView, ~5MB binary vs 200MB Electron)
- **UI Framework:** SolidJS or Leptos for reactive UI with near-zero overhead
- **Plugin Sandbox:** Wasmtime (WebAssembly Component Model) with WIT-defined interfaces
- **Local AI:** llama.cpp via `llama_cpp` Rust crate with router-mode multi-model
- **Storage:** SQLite (structured) + Qdrant-embedded (vector) + age-encrypted vault
- **IPC:** Cap'n Proto for zero-copy serialization on the event bus
- **MCP:** Native Rust MCP client/server with Streamable HTTP + stdio transports

Architecture diagrams will cover:

- System layer diagram (OS -> Kernel -> Runtime -> UI)
- Process model (Core, Agent Workers, WASM Sandboxes, WebView)
- Data flow diagram (events, tool calls, model routing)
- Memory architecture (ephemeral ring buffer, long-term SQLite, vector Qdrant, encrypted vault)

### 3. AI Core System

- Multi-model router with latency/cost/capability scoring
- Tree-of-thought planning engine with DAG execution
- Agent delegation via spawn/join with resource budgets
- Context compression using recursive summarization
- Deterministic mode (fixed seeds, recorded tool calls, replay capability)
- Self-healing workflows with checkpoint/retry/fallback

### 4. Visual Automation Engine

- Node-graph editor (Rust-backed, WebView-rendered)
- Trigger types: cron, filesystem watch, webhook, OS events, MCP events
- Reusable sub-workflow modules with typed ports
- Versioned workflow registry with diff/rollback
- Error handling nodes with retry policies and dead-letter queues

### 5. Skills & Marketplace 2.0

- WASM Component Model SDK with WIT interfaces
- Cryptographic Ed25519 signing of all skill packages
- Three-tier review: automated scanning -> community audit -> staff review
- Capability-scoped permissions (no skill gets full system access -- ever)
- Revenue sharing: 85/15 developer/platform split

### 6. Full MCP Supremacy

- Native MCP server + client in Rust
- MCP Apps support (ui:// scheme rendering in sandboxed iframes)
- Multi-endpoint orchestration with connection pooling
- Transaction-safe tool invocation with rollback
- Dev-mode MCP simulator with record/replay

### 7. Security Model (Enterprise-Grade)

- Zero-trust capability-based security (no ambient authority)
- Every tool call requires signed capability token
- Filesystem access via virtual FS overlay (no direct OS access for agents)
- AES-256-GCM encrypted memory store in OS keychain
- Tamper-evident audit log (append-only, hash-chained)
- Kill-switch: instant agent termination with state preservation
- Behavioral anomaly detection (statistical deviation from skill manifest)
- Automatic quarantine of skills exhibiting suspicious patterns

### 8. Premium UX & Interface

- Sub-16ms frame times via native WebView + Rust compute
- Multi-panel workspace with drag-and-drop layout
- Command palette (Raycast-speed, <50ms response)
- Live agent activity feed with token streaming
- Permission transparency dashboard
- Built-in performance monitor and debug console
- Theme engine with CSS custom properties
- WCAG 2.1 AA accessibility compliance

### 9. Developer Platform

- `openzax` CLI: init, build, test, publish, sign
- Rust + TypeScript + Python SDK for skill development
- Extension debugger with WASM inspector
- Test harness with mock MCP servers and simulated tool calls
- Auto-generated API docs from WIT interfaces
- CI templates (GitHub Actions, GitLab CI)

### 10. Monetization Engine

- **Free:** Core runtime, local AI, 5 skills, community support
- **Pro ($12/mo):** Unlimited skills, cloud model routing, priority support, advanced workflows
- **Enterprise ($49/seat/mo):** SSO/SAML, audit logs, fleet management, SLA, custom skills
- **Marketplace:** 15% platform fee on paid skills
- **Hosted Orchestration:** Managed cloud agent execution ($0.01/task-minute)

### 11. Roadmap

- Phase 0 (Weeks 1-4): Core Rust runtime, Tauri shell, basic agent loop
- Phase 1 (Months 2-4): WASM sandbox, MCP client, local LLM, command palette
- Phase 2 (Months 5-7): Skills SDK, marketplace backend, visual workflow editor
- Phase 3 (Months 8-10): Public marketplace, community skills, cloud model routing
- Phase 4 (Months 11-14): Enterprise features, SSO, fleet management, audit
- Phase 5 (Months 15+): Platform APIs, third-party integrations, mobile companion

### 12. Risk & Attack Surface Analysis

- Supply chain attacks on WASM skills (mitigated by signing + scanning)
- Local model jailbreaking (mitigated by output filtering + capability constraints)
- Side-channel attacks on encrypted memory (mitigated by OS keychain + memory zeroing)
- Ecosystem bootstrapping risk (mitigated by MCP compatibility importing existing tools)
- Competitive response from OpenClaw (mitigated by structural security advantage they cannot retrofit)

---

## Implementation Approach

The blueprint will be written as a single comprehensive Markdown file (~3000-4000 lines) with:

- Full table of contents with anchor links
- Mermaid diagrams for all architecture visualizations
- Concrete technology choices with version numbers
- No placeholders or "TBD" sections
- Git-ready formatting
