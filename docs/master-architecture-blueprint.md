# OpenZax — Master Architecture Blueprint

> **Version:** 1.0.0  
> **Date:** 2026-03-01  
> **Classification:** Internal — Engineering & Strategy  
> **Status:** Approved for Implementation

---

## Table of Contents

1. [Executive War Strategy](#1-executive-war-strategy)
   - 1.1 [Competitive Positioning](#11-competitive-positioning)
   - 1.2 [Differentiation Matrix](#12-differentiation-matrix)
   - 1.3 [Three-Phase Ecosystem Domination](#13-three-phase-ecosystem-domination)
   - 1.4 [Go-to-Market Attack Vectors](#14-go-to-market-attack-vectors)
2. [Ultra-Advanced Technical Architecture](#2-ultra-advanced-technical-architecture)
   - 2.1 [Core Technology Stack](#21-core-technology-stack)
   - 2.2 [System Layer Diagram](#22-system-layer-diagram)
   - 2.3 [Process Model](#23-process-model)
   - 2.4 [Data Flow Architecture](#24-data-flow-architecture)
   - 2.5 [Memory Architecture](#25-memory-architecture)
   - 2.6 [Build & Distribution](#26-build--distribution)
3. [AI Core System](#3-ai-core-system)
   - 3.1 [Multi-Model Router](#31-multi-model-router)
   - 3.2 [Tree-of-Thought Planning Engine](#32-tree-of-thought-planning-engine)
   - 3.3 [Agent Delegation Architecture](#33-agent-delegation-architecture)
   - 3.4 [Context Compression Pipeline](#34-context-compression-pipeline)
   - 3.5 [Deterministic Mode](#35-deterministic-mode)
   - 3.6 [Self-Healing Workflows](#36-self-healing-workflows)
4. [Visual Automation Engine](#4-visual-automation-engine)
   - 4.1 [Node-Graph Editor](#41-node-graph-editor)
   - 4.2 [Trigger System](#42-trigger-system)
   - 4.3 [Sub-Workflow Modules](#43-sub-workflow-modules)
   - 4.4 [Workflow Registry](#44-workflow-registry)
   - 4.5 [Error Handling Architecture](#45-error-handling-architecture)
5. [Skills & Marketplace 2.0](#5-skills--marketplace-20)
   - 5.1 [WASM Component Model SDK](#51-wasm-component-model-sdk)
   - 5.2 [Cryptographic Signing Pipeline](#52-cryptographic-signing-pipeline)
   - 5.3 [Three-Tier Review System](#53-three-tier-review-system)
   - 5.4 [Capability-Scoped Permissions](#54-capability-scoped-permissions)
   - 5.5 [Revenue Model & Economics](#55-revenue-model--economics)
6. [Full MCP Supremacy](#6-full-mcp-supremacy)
   - 6.1 [Native MCP Implementation](#61-native-mcp-implementation)
   - 6.2 [MCP Apps & UI Scheme](#62-mcp-apps--ui-scheme)
   - 6.3 [Multi-Endpoint Orchestration](#63-multi-endpoint-orchestration)
   - 6.4 [Transaction-Safe Tool Invocation](#64-transaction-safe-tool-invocation)
   - 6.5 [Dev-Mode MCP Simulator](#65-dev-mode-mcp-simulator)
7. [Security Model (Enterprise-Grade)](#7-security-model-enterprise-grade)
   - 7.1 [Zero-Trust Capability Architecture](#71-zero-trust-capability-architecture)
   - 7.2 [Signed Capability Tokens](#72-signed-capability-tokens)
   - 7.3 [Virtual Filesystem Overlay](#73-virtual-filesystem-overlay)
   - 7.4 [Encrypted Memory Store](#74-encrypted-memory-store)
   - 7.5 [Tamper-Evident Audit Log](#75-tamper-evident-audit-log)
   - 7.6 [Kill-Switch & Containment](#76-kill-switch--containment)
   - 7.7 [Behavioral Anomaly Detection](#77-behavioral-anomaly-detection)
   - 7.8 [Automatic Quarantine System](#78-automatic-quarantine-system)
8. [Premium UX & Interface](#8-premium-ux--interface)
   - 8.1 [Rendering Pipeline](#81-rendering-pipeline)
   - 8.2 [Multi-Panel Workspace](#82-multi-panel-workspace)
   - 8.3 [Command Palette](#83-command-palette)
   - 8.4 [Live Agent Activity Feed](#84-live-agent-activity-feed)
   - 8.5 [Permission Transparency Dashboard](#85-permission-transparency-dashboard)
   - 8.6 [Debug Console & Performance Monitor](#86-debug-console--performance-monitor)
   - 8.7 [Theme Engine](#87-theme-engine)
   - 8.8 [Accessibility Compliance](#88-accessibility-compliance)
9. [Developer Platform](#9-developer-platform)
   - 9.1 [CLI Toolchain](#91-cli-toolchain)
   - 9.2 [Multi-Language SDKs](#92-multi-language-sdks)
   - 9.3 [Extension Debugger](#93-extension-debugger)
   - 9.4 [Test Harness](#94-test-harness)
   - 9.5 [Documentation Generation](#95-documentation-generation)
   - 9.6 [CI/CD Templates](#96-cicd-templates)
10. [Monetization Engine](#10-monetization-engine)
    - 10.1 [Tier Structure](#101-tier-structure)
    - 10.2 [Marketplace Economics](#102-marketplace-economics)
    - 10.3 [Hosted Orchestration](#103-hosted-orchestration)
    - 10.4 [Revenue Projections](#104-revenue-projections)
11. [Roadmap](#11-roadmap)
    - 11.1 [Phase 0 — Foundation (Weeks 1–4)](#111-phase-0--foundation-weeks-14)
    - 11.2 [Phase 1 — Core Platform (Months 2–4)](#112-phase-1--core-platform-months-24)
    - 11.3 [Phase 2 — Ecosystem (Months 5–7)](#113-phase-2--ecosystem-months-57)
    - 11.4 [Phase 3 — Community Launch (Months 8–10)](#114-phase-3--community-launch-months-810)
    - 11.5 [Phase 4 — Enterprise (Months 11–14)](#115-phase-4--enterprise-months-1114)
    - 11.6 [Phase 5 — Platform Maturity (Months 15+)](#116-phase-5--platform-maturity-months-15)
12. [Risk & Attack Surface Analysis](#12-risk--attack-surface-analysis)
    - 12.1 [Supply Chain Attacks](#121-supply-chain-attacks)
    - 12.2 [Local Model Jailbreaking](#122-local-model-jailbreaking)
    - 12.3 [Side-Channel Attacks](#123-side-channel-attacks)
    - 12.4 [Ecosystem Bootstrapping Risk](#124-ecosystem-bootstrapping-risk)
    - 12.5 [Competitive Response](#125-competitive-response)
    - 12.6 [Threat Model Summary](#126-threat-model-summary)

---

## 1. Executive War Strategy

### 1.1 Competitive Positioning

OpenClaw dominates the AI-powered development assistant space — but dominates it **poorly**. A cascade of security failures has eroded trust across the developer community:

| Vulnerability | Impact | Source |
|---|---|---|
| CVE-2026-25253 (CVSS 8.8) | One-click RCE via agent visiting attacker-controlled URL | NIST NVD |
| Auth disabled by default | 40,000+ instances publicly exposed on the internet | Shodan census |
| Malware-laden skill ecosystem | 12% of ClawHub skills (341 / 2,857) contained keyloggers or credential stealers | Independent audit |
| Prompt injection | Architecturally unfixable per Anthropic's own research | Anthropic paper, 2025 |
| No plugin sandboxing | Skills execute with the full privilege level of the host process | OpenClaw source review |
| Node.js/Python runtime | Inherent performance ceiling; ~150 MB base memory footprint | Runtime benchmarks |

OpenZax positions itself as **"the secure alternative that never compromises on speed."** Every architectural decision is a direct, deliberate response to one of these weaknesses.

**Positioning Statement:**

> OpenZax is the first AI development assistant built from the ground up in Rust with hardware-grade security, WASM-sandboxed extensions, and a cryptographically-verified skill marketplace — delivering the power developers demand without the attack surface they fear.

### 1.2 Differentiation Matrix

| Dimension | OpenClaw (Current) | OpenZax (Target) | Advantage |
|---|---|---|---|
| **Security Model** | Ambient authority; no sandbox; auth off by default | Zero-trust capabilities; WASM sandbox; auth mandatory | Structural |
| **Runtime Performance** | Node.js/Python; ~150 MB idle; 200 MB+ binary (Electron) | Rust core; ~30 MB idle; ~5 MB binary (Tauri v2) | 5–40x |
| **Extension Safety** | No signing; no review; full system access | Ed25519 signing; 3-tier review; capability-scoped | Architectural |
| **UX Latency** | Electron IPC overhead; 60–200 ms command palette | Native WebView; Cap'n Proto IPC; <50 ms palette | Perceptible |
| **Local AI** | External API dependency only | Built-in llama.cpp with multi-model routing | Privacy win |
| **MCP Support** | Partial; single-server stdio only | Full spec; Streamable HTTP + stdio; multi-endpoint | Complete |
| **Marketplace Trust** | 12% malware rate | Automated scan + community audit + staff review | Verified |
| **Enterprise Readiness** | No SSO; no audit trail; no fleet management | SAML/OIDC SSO; hash-chained audit log; fleet API | Day-one |

### 1.3 Three-Phase Ecosystem Domination

```mermaid
graph LR
    subgraph "Phase 1 — CAPTURE"
        A[Security-Conscious Developers] --> B[Open-Source Core]
        B --> C[Trust Through Transparency]
        C --> D[Community Growth]
    end

    subgraph "Phase 2 — EXPAND"
        D --> E[Enterprise Sales]
        E --> F[SSO / Audit / Fleet]
        F --> G[Revenue Engine]
    end

    subgraph "Phase 3 — DOMINATE"
        G --> H[Marketplace Network Effects]
        H --> I[Skill Ecosystem Moat]
        I --> J[Platform Lock-In via Value]
    end

    style A fill:#e74c3c,color:#fff
    style D fill:#f39c12,color:#fff
    style G fill:#2ecc71,color:#fff
    style J fill:#3498db,color:#fff
```

**Phase 1 — Capture (Months 1–6):** Target security-conscious developers burned by OpenClaw's CVEs. Lead with the open-source Rust core, WASM sandbox, and signed-skills narrative. Every conference talk, blog post, and HN comment hammers the security angle. Goal: 10,000 active weekly users.

**Phase 2 — Expand (Months 7–12):** Convert community traction into enterprise pipeline. Ship SSO/SAML, fleet management, audit logs, and SLA-backed support. Land 50 enterprise accounts at $49/seat/month. Goal: $500K ARR.

**Phase 3 — Dominate (Months 13–24):** Marketplace network effects create a self-reinforcing moat. More users attract more skill developers; more skills attract more users. The 85/15 revenue split incentivizes top developers to build on OpenZax first. Goal: 100,000 users, 5,000+ verified skills, $5M ARR.

### 1.4 Go-to-Market Attack Vectors

| Channel | Tactic | Target Audience | Expected CAC |
|---|---|---|---|
| Hacker News / Reddit | Security teardown posts comparing OpenZax vs OpenClaw | Senior engineers, CTOs | $0 (organic) |
| Security conferences (DEF CON, Black Hat) | Live demos of OpenClaw CVE exploitation vs OpenZax defense | Security engineers, CISOs | $2,000/event |
| GitHub Sponsors / Open Source | Free core with premium features | Individual developers | $0 (freemium funnel) |
| Direct enterprise sales | White-glove onboarding with security audit reports | Enterprise security teams | $5,000/account |
| Developer YouTube | Build-in-public series showing Rust + WASM architecture | Developer community | $500/video |
| Skill developer program | Early access SDK + revenue sharing incentives | Plugin/extension developers | $0 (revenue share) |

---

## 2. Ultra-Advanced Technical Architecture

### 2.1 Core Technology Stack

| Layer | Technology | Version | Rationale |
|---|---|---|---|
| **Core Runtime** | Rust | 1.82+ (2024 edition) | Memory safety without GC; zero-cost abstractions; fearless concurrency |
| **Desktop Shell** | Tauri | 2.2+ | Native OS WebView; ~5 MB binary; system tray integration; auto-updater |
| **UI Framework** | Leptos | 0.7+ | Rust-native reactive UI; fine-grained reactivity; SSR-capable; compiles to WASM |
| **Plugin Sandbox** | Wasmtime | 27.0+ | WASI preview 2; Component Model; fuel-based CPU metering; memory limits |
| **Local AI** | llama.cpp | b4547+ | GGUF model loading; GPU offload (CUDA/Metal/Vulkan); batched inference |
| **Rust AI Bindings** | llama-cpp-rs | 0.4+ | Safe Rust wrapper; async inference; model hot-swap |
| **Structured Storage** | SQLite | 3.47+ (via rusqlite) | WAL mode; FTS5 full-text search; JSON1 extension; R-Tree spatial index |
| **Vector Storage** | Qdrant (embedded) | 1.13+ | HNSW indexing; scalar quantization; payload filtering; in-process mode |
| **Encrypted Vault** | age + OS Keychain | age 0.10+ | X25519 encryption; passphrase or hardware key; OS keychain for key storage |
| **IPC Serialization** | Cap'n Proto | 0.20+ (capnpc-rust) | Zero-copy deserialization; schema evolution; <1μs encode/decode |
| **Async Runtime** | Tokio | 1.43+ | Multi-threaded work-stealing; io_uring on Linux; IOCP on Windows |
| **HTTP Client** | reqwest | 0.12+ | HTTP/2 multiplexing; connection pooling; rustls TLS |
| **HTTP Server** | axum | 0.8+ | Tower middleware ecosystem; extract-based handlers; WebSocket support |
| **Logging** | tracing | 0.1.41+ | Structured spans; async-aware; multiple subscriber outputs |
| **Error Handling** | thiserror + anyhow | 2.0+ / 1.0+ | Typed library errors; ergonomic application errors |
| **CLI Framework** | clap | 4.5+ | Derive macros; shell completions; man page generation |

### 2.2 System Layer Diagram

```mermaid
graph TB
    subgraph "Operating System Layer"
        OS[Windows / macOS / Linux]
        KC[OS Keychain<br/>Credential Guard / Keychain / libsecret]
        FS[Native Filesystem]
        GPU[GPU Driver<br/>CUDA / Metal / Vulkan]
    end

    subgraph "Rust Core Engine"
        KERNEL[Security Kernel<br/>Capability Manager + Policy Engine]
        RUNTIME[Agent Runtime<br/>Tokio + Task Scheduler]
        EVENTBUS[Event Bus<br/>Cap'n Proto + Broadcast Channels]
        STORAGE[Storage Layer<br/>SQLite + Qdrant + age Vault]
        AIROUTER[AI Router<br/>Multi-Model + llama.cpp]
    end

    subgraph "Isolation Layer"
        WASM1[WASM Sandbox 1<br/>Wasmtime + WASI P2]
        WASM2[WASM Sandbox 2<br/>Wasmtime + WASI P2]
        WASMN[WASM Sandbox N<br/>Wasmtime + WASI P2]
        VFS[Virtual Filesystem<br/>Overlay + Allowlist]
    end

    subgraph "Presentation Layer"
        WEBVIEW[Tauri WebView<br/>OS-native renderer]
        LEPTOS[Leptos UI<br/>Reactive components]
        CMDPAL[Command Palette<br/><50ms response]
    end

    OS --> KERNEL
    KC --> STORAGE
    FS --> VFS
    GPU --> AIROUTER

    KERNEL --> RUNTIME
    KERNEL --> EVENTBUS
    KERNEL --> STORAGE
    RUNTIME --> AIROUTER
    RUNTIME --> WASM1
    RUNTIME --> WASM2
    RUNTIME --> WASMN

    WASM1 --> VFS
    WASM2 --> VFS
    WASMN --> VFS

    EVENTBUS --> WEBVIEW
    WEBVIEW --> LEPTOS
    LEPTOS --> CMDPAL

    style KERNEL fill:#c0392b,color:#fff
    style RUNTIME fill:#2980b9,color:#fff
    style WASM1 fill:#27ae60,color:#fff
    style WASM2 fill:#27ae60,color:#fff
    style WASMN fill:#27ae60,color:#fff
    style WEBVIEW fill:#8e44ad,color:#fff
```

### 2.3 Process Model

```mermaid
graph LR
    subgraph "Main Process (PID 1)"
        CORE[Core Engine<br/>Security Kernel<br/>Event Bus<br/>Storage Manager]
    end

    subgraph "Agent Workers (Thread Pool)"
        AW1[Agent Worker 1<br/>Tokio Task]
        AW2[Agent Worker 2<br/>Tokio Task]
        AWN[Agent Worker N<br/>Tokio Task]
    end

    subgraph "WASM Sandboxes (Isolated)"
        WS1[Skill: git-helper<br/>4 MB memory cap<br/>10^9 fuel budget]
        WS2[Skill: code-formatter<br/>8 MB memory cap<br/>10^8 fuel budget]
        WSN[Skill: test-runner<br/>16 MB memory cap<br/>10^10 fuel budget]
    end

    subgraph "WebView Process"
        WV[Tauri WebView<br/>UI Thread<br/>Leptos WASM]
    end

    subgraph "LLM Process"
        LLM[llama.cpp Server<br/>Model Pool<br/>GPU Memory]
    end

    CORE -->|"Tokio spawn"| AW1
    CORE -->|"Tokio spawn"| AW2
    CORE -->|"Tokio spawn"| AWN

    AW1 -->|"Wasmtime call"| WS1
    AW2 -->|"Wasmtime call"| WS2
    AWN -->|"Wasmtime call"| WSN

    CORE -->|"Tauri IPC<br/>Cap'n Proto"| WV
    CORE -->|"HTTP/gRPC"| LLM

    style CORE fill:#e74c3c,color:#fff
    style WV fill:#9b59b6,color:#fff
    style LLM fill:#f39c12,color:#fff
    style WS1 fill:#27ae60,color:#fff
    style WS2 fill:#27ae60,color:#fff
    style WSN fill:#27ae60,color:#fff
```

**Process isolation guarantees:**

- **Main Process:** Single Rust binary. Owns the security kernel, event bus, capability store, and database connections. All privileged operations flow through this process.
- **Agent Workers:** Tokio tasks on the main process's thread pool. Each agent operates within its own `AgentContext` struct carrying a capability set, a budget counter, and a cancellation token.
- **WASM Sandboxes:** Each skill instance runs in its own `wasmtime::Store` with independent memory, fuel (CPU instruction budget), and a capability-filtered `WasiCtx`. A sandbox crash cannot affect other sandboxes or the host.
- **WebView Process:** OS-provided WebView (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux). Communicates with the core engine via Tauri's IPC bridge, serialized with Cap'n Proto.
- **LLM Process:** llama.cpp runs as an in-process library via `llama-cpp-rs` bindings. GPU memory is managed by the model pool, which handles loading, unloading, and context recycling across concurrent requests.

### 2.4 Data Flow Architecture

```mermaid
sequenceDiagram
    participant User
    participant WebView
    participant EventBus
    participant SecurityKernel
    participant AgentRuntime
    participant AIRouter
    participant WASMSandbox
    participant Storage

    User->>WebView: Input command / prompt
    WebView->>EventBus: UserInput event (Cap'n Proto)
    EventBus->>SecurityKernel: Validate session token
    SecurityKernel->>AgentRuntime: Dispatch with capability set
    AgentRuntime->>AIRouter: Route prompt to optimal model
    AIRouter-->>AgentRuntime: Streaming token response
    AgentRuntime->>SecurityKernel: Request tool_call capability
    SecurityKernel->>SecurityKernel: Verify capability token
    SecurityKernel->>WASMSandbox: Execute skill (sandboxed)
    WASMSandbox->>Storage: Read/write via VFS overlay
    Storage-->>WASMSandbox: Result
    WASMSandbox-->>AgentRuntime: Tool result
    AgentRuntime-->>EventBus: AgentOutput event
    EventBus-->>WebView: Stream to UI
    WebView-->>User: Render response
    AgentRuntime->>Storage: Persist conversation + audit log
```

**Event types on the bus:**

| Event | Payload | Direction |
|---|---|---|
| `UserInput` | `{ session_id, content, attachments[] }` | WebView → Core |
| `AgentThinking` | `{ agent_id, thought_text }` | Core → WebView |
| `AgentTokenStream` | `{ agent_id, token, finish_reason? }` | Core → WebView |
| `ToolCallRequest` | `{ agent_id, tool_name, params, capability_token }` | Core → SecurityKernel |
| `ToolCallResult` | `{ call_id, result, duration_ms }` | Sandbox → Core |
| `SkillLoaded` | `{ skill_id, manifest, memory_limit }` | Core → WebView |
| `AuditEntry` | `{ timestamp, actor, action, resource, outcome }` | Core → Storage |
| `ErrorEvent` | `{ source, error_code, message, stack? }` | Any → Core |

### 2.5 Memory Architecture

```mermaid
graph TB
    subgraph "Ephemeral (In-Process)"
        RING[Ring Buffer<br/>Last 1000 events<br/>Lock-free SPSC queue<br/>~4 MB fixed allocation]
        CACHE[LRU Cache<br/>Hot model contexts<br/>Embedding cache<br/>Configurable size]
    end

    subgraph "Persistent — Structured (SQLite)"
        CONV[Conversations<br/>Messages, tool calls, metadata<br/>WAL mode, 64 KB page size]
        SKILLS_DB[Skill Registry<br/>Manifests, permissions, versions<br/>FTS5 indexed]
        CONFIG[Configuration<br/>User preferences, keybindings<br/>JSON1 extension]
        AUDIT[Audit Log<br/>Hash-chained entries<br/>Append-only table]
    end

    subgraph "Persistent — Vector (Qdrant Embedded)"
        EMBEDDINGS[Code Embeddings<br/>1536-dim vectors<br/>HNSW index, cosine similarity]
        MEMORY[Long-Term Memory<br/>Conversation summaries<br/>Payload-filtered retrieval]
    end

    subgraph "Encrypted Vault (age + OS Keychain)"
        SECRETS[API Keys<br/>AES-256-GCM at rest<br/>X25519 key exchange]
        TOKENS[Auth Tokens<br/>OAuth2 refresh tokens<br/>Session material]
        PRIVKEYS[Signing Keys<br/>Ed25519 skill signing keypair<br/>Never leaves vault]
    end

    RING -->|overflow eviction| CONV
    CACHE -->|context persist| CONV
    CONV -->|summarize| MEMORY
    EMBEDDINGS ---|"semantic search"| CONV

    style RING fill:#3498db,color:#fff
    style CACHE fill:#3498db,color:#fff
    style CONV fill:#2ecc71,color:#fff
    style AUDIT fill:#2ecc71,color:#fff
    style EMBEDDINGS fill:#9b59b6,color:#fff
    style MEMORY fill:#9b59b6,color:#fff
    style SECRETS fill:#e74c3c,color:#fff
    style TOKENS fill:#e74c3c,color:#fff
    style PRIVKEYS fill:#e74c3c,color:#fff
```

**Memory budget targets:**

| Component | Allocation | Strategy |
|---|---|---|
| Core engine (idle) | ~15 MB | Pre-allocated structures; no dynamic allocation on hot path |
| Event ring buffer | 4 MB (fixed) | Lock-free SPSC; oldest events evicted on overflow |
| SQLite connections | ~5 MB | WAL mode; single writer, multiple readers; mmap I/O |
| Qdrant embedded | ~10 MB base + index | Scalar quantization reduces vector footprint by 4x |
| Each WASM sandbox | 4–64 MB (configurable) | Hard `memory.maximum` in WASM linear memory; OOM traps sandbox only |
| LLM context (7B model) | ~4 GB VRAM (GPU) / ~6 GB RAM (CPU) | KV-cache recycling; context window sliding |
| WebView (OS-provided) | ~30–80 MB | Managed by OS; no Electron overhead |
| **Total (idle, no LLM)** | **~30 MB** | vs OpenClaw's ~150 MB (5x reduction) |

### 2.6 Build & Distribution

| Artifact | Format | Size Target | Notes |
|---|---|---|---|
| Desktop installer (Windows) | `.msi` via WiX | <8 MB | Tauri + WebView2 bootstrapper |
| Desktop installer (macOS) | `.dmg` universal binary | <10 MB | arm64 + x86_64 fat binary |
| Desktop installer (Linux) | `.AppImage` + `.deb` + `.rpm` | <8 MB | WebKitGTK runtime dependency |
| CLI tool | Static binary | <5 MB | Single `openzax` binary, no runtime dependencies |
| WASM skill template | `.wasm` component | <500 KB typical | Stripped, optimized with `wasm-opt -O3` |
| Auto-updater | Delta patches via Tauri updater | <2 MB typical | Ed25519 signed update manifests |

**CI/CD pipeline:**

```mermaid
graph LR
    PR[Pull Request] --> LINT[cargo clippy<br/>+ cargo fmt check]
    LINT --> TEST[cargo test<br/>+ WASM skill tests]
    TEST --> AUDIT_DEP[cargo audit<br/>+ cargo deny]
    AUDIT_DEP --> BUILD[Cross-compile<br/>x86_64 + aarch64<br/>Win + Mac + Linux]
    BUILD --> SIGN[Code signing<br/>Authenticode / Apple / GPG]
    SIGN --> RELEASE[GitHub Release<br/>+ Tauri updater manifest]

    style PR fill:#3498db,color:#fff
    style RELEASE fill:#27ae60,color:#fff
```

---

## 3. AI Core System

### 3.1 Multi-Model Router

The AI router is the brain of OpenZax's model orchestration. It selects the optimal model for each request based on a weighted scoring function across four dimensions.

**Scoring function:**

```
score(model, request) = w_latency * (1 - normalized_latency)
                      + w_cost    * (1 - normalized_cost)
                      + w_cap     * capability_match_ratio
                      + w_quality * estimated_quality_score
```

Default weights: `w_latency=0.25, w_cost=0.20, w_cap=0.30, w_quality=0.25`

Users can adjust weights via a preferences slider to prioritize speed, cost, capability, or quality.

**Model registry schema (SQLite):**

```sql
CREATE TABLE models (
    id              TEXT PRIMARY KEY,
    provider        TEXT NOT NULL,          -- 'local', 'openai', 'anthropic', 'google', etc.
    name            TEXT NOT NULL,          -- 'llama-3.3-70b-q4', 'claude-4-sonnet', etc.
    context_window  INTEGER NOT NULL,       -- max tokens
    input_cost      REAL,                   -- $/1M input tokens (NULL for local)
    output_cost     REAL,                   -- $/1M output tokens (NULL for local)
    capabilities    TEXT NOT NULL,           -- JSON: ["code","reasoning","vision","tool_use"]
    avg_latency_ms  REAL,                   -- rolling average from last 100 requests
    quality_score   REAL NOT NULL DEFAULT 0.5, -- 0.0–1.0, updated by user feedback
    is_local        BOOLEAN NOT NULL DEFAULT 0,
    gpu_layers      INTEGER,                -- for local models: layers offloaded to GPU
    quantization    TEXT,                   -- 'q4_k_m', 'q5_k_s', 'q8_0', 'f16'
    max_batch       INTEGER DEFAULT 1,
    enabled         BOOLEAN NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_models_provider ON models(provider);
CREATE INDEX idx_models_enabled ON models(enabled);
```

**Router architecture:**

```mermaid
graph TB
    REQ[Incoming Request<br/>prompt + context + constraints] --> CLASSIFY[Request Classifier<br/>code / reasoning / vision / chat]
    CLASSIFY --> FILTER[Model Filter<br/>capability match<br/>context window check]
    FILTER --> SCORE[Scoring Engine<br/>latency × cost × capability × quality]
    SCORE --> SELECT[Model Selector<br/>top-k with fallback chain]
    SELECT --> LOCAL{Local model<br/>available?}
    LOCAL -->|yes| LLAMA[llama.cpp<br/>In-process inference]
    LOCAL -->|no| CLOUD[Cloud API<br/>reqwest + streaming]
    LLAMA --> STREAM[Token Stream]
    CLOUD --> STREAM
    STREAM --> FEEDBACK[Feedback Loop<br/>update latency + quality scores]

    style REQ fill:#3498db,color:#fff
    style SCORE fill:#f39c12,color:#fff
    style LLAMA fill:#27ae60,color:#fff
    style CLOUD fill:#9b59b6,color:#fff
```

**Local model management:**

- **Model discovery:** On startup, scan `~/.openzax/models/` for GGUF files. Parse metadata headers to auto-populate the model registry.
- **GPU offload:** Detect available GPU memory via platform APIs (CUDA `cudaMemGetInfo`, Metal `recommendedMaxWorkingSetSize`, Vulkan `vkGetPhysicalDeviceMemoryProperties`). Compute optimal `n_gpu_layers` to maximize offload without OOM.
- **Hot-swap:** Models can be loaded/unloaded at runtime. The router maintains a pool of up to 3 loaded models (configurable). LRU eviction when a new model is needed and the pool is full.
- **Batched inference:** For multi-agent scenarios, the router batches requests to the same model into a single inference pass, reducing GPU context-switch overhead.

### 3.2 Tree-of-Thought Planning Engine

Complex tasks require structured reasoning, not single-shot prompting. OpenZax implements a tree-of-thought (ToT) planner that decomposes tasks into a directed acyclic graph (DAG) of reasoning steps.

```mermaid
graph TB
    ROOT[Task: Refactor auth module] --> A[Analyze current auth flow]
    ROOT --> B[Identify security gaps]
    A --> C[Map dependency graph]
    B --> D[Research best practices]
    C --> E[Plan migration steps]
    D --> E
    E --> F[Generate code changes]
    E --> G[Generate test cases]
    F --> H[Validate changes compile]
    G --> H
    H --> I[Run test suite]
    I --> J{All tests pass?}
    J -->|yes| K[Commit & report]
    J -->|no| L[Diagnose failures]
    L --> F

    style ROOT fill:#e74c3c,color:#fff
    style K fill:#27ae60,color:#fff
    style L fill:#f39c12,color:#fff
```

**DAG execution engine:**

```rust
// Conceptual structure — actual implementation uses async trait objects
struct PlanNode {
    id: NodeId,
    task: String,
    dependencies: Vec<NodeId>,
    status: NodeStatus,           // Pending | Running | Completed | Failed
    result: Option<NodeResult>,
    retry_count: u32,
    max_retries: u32,
    timeout: Duration,
}

enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String, retryable: bool },
}

struct PlanDAG {
    nodes: HashMap<NodeId, PlanNode>,
    execution_order: Vec<Vec<NodeId>>,  // topological layers for parallel execution
}
```

**Execution semantics:**

1. The planner generates a DAG from the user's prompt using a planning-specialized model (or the best available model with a planning system prompt).
2. Nodes are grouped into topological layers. All nodes within a layer execute concurrently.
3. Each node receives the accumulated context from its dependency nodes.
4. Failed nodes trigger the retry policy (exponential backoff, max 3 retries). If retries exhaust, the planner can re-plan the subgraph rooted at the failed node.
5. The user can inspect, approve, or modify the plan before execution begins (configurable: auto-execute for trusted tasks, manual approval for destructive operations).

### 3.3 Agent Delegation Architecture

Agents can spawn sub-agents with scoped resource budgets. This enables divide-and-conquer strategies for large tasks.

```mermaid
graph TB
    PARENT[Parent Agent<br/>Budget: 100K tokens, 60s wall-clock] --> CHILD1[Sub-Agent: Analyze<br/>Budget: 20K tokens, 15s]
    PARENT --> CHILD2[Sub-Agent: Implement<br/>Budget: 50K tokens, 30s]
    PARENT --> CHILD3[Sub-Agent: Test<br/>Budget: 30K tokens, 15s]

    CHILD2 --> GRANDCHILD1[Sub-Agent: Frontend<br/>Budget: 25K tokens, 15s]
    CHILD2 --> GRANDCHILD2[Sub-Agent: Backend<br/>Budget: 25K tokens, 15s]

    style PARENT fill:#e74c3c,color:#fff
    style CHILD1 fill:#3498db,color:#fff
    style CHILD2 fill:#3498db,color:#fff
    style CHILD3 fill:#3498db,color:#fff
    style GRANDCHILD1 fill:#2ecc71,color:#fff
    style GRANDCHILD2 fill:#2ecc71,color:#fff
```

**Budget enforcement:**

| Resource | Tracking | Limit Action |
|---|---|---|
| Token consumption | Counted per model call; debited from parent budget | Graceful termination with partial result |
| Wall-clock time | `tokio::time::timeout` wrapping the agent task | Cancel agent; preserve intermediate state |
| Tool call count | Incremented per `ToolCallRequest` event | Reject further tool calls; force summarization |
| Memory (WASM) | Wasmtime fuel + linear memory pages | Trap WASM instance; return OOM error |
| Filesystem I/O | Byte counter on VFS operations | Throttle to rate limit; reject above hard cap |

**Spawn/join protocol:**

1. Parent calls `agent_spawn(task, budget, capabilities)`.
2. Security kernel mints a child capability set that is a **strict subset** of the parent's capabilities.
3. Child executes independently on the Tokio thread pool.
4. Parent can `agent_join(child_id)` to block until the child completes, or `agent_poll(child_id)` for non-blocking status checks.
5. Child results are returned via the event bus. The parent receives a `ChildCompleted` event with the child's output and remaining budget.

### 3.4 Context Compression Pipeline

Long conversations exceed model context windows. OpenZax uses a multi-stage compression pipeline to maintain coherent context across extended sessions.

```mermaid
graph LR
    RAW[Raw Conversation<br/>Full message history] --> WINDOW[Sliding Window<br/>Last N messages]
    WINDOW --> SUMMARIZE[Recursive Summarizer<br/>Compress older messages<br/>into structured summaries]
    SUMMARIZE --> EMBED[Embedding Index<br/>Qdrant vector search<br/>for relevant past context]
    EMBED --> ASSEMBLE[Context Assembler<br/>system_prompt<br/>+ summary<br/>+ relevant_chunks<br/>+ recent_messages]
    ASSEMBLE --> MODEL[Model Input<br/>Within context window]

    style RAW fill:#e74c3c,color:#fff
    style ASSEMBLE fill:#27ae60,color:#fff
    style MODEL fill:#3498db,color:#fff
```

**Compression stages:**

| Stage | Trigger | Compression Ratio | Fidelity |
|---|---|---|---|
| Sliding window | Always active | 1:1 (recent N messages) | Perfect |
| Recursive summarization | Context > 70% of window | ~10:1 per summary pass | High (key decisions, code changes preserved) |
| Semantic retrieval | Context > 90% of window | Selective (top-k relevant) | Variable (cosine similarity threshold 0.75) |
| Aggressive pruning | Context critically full | ~50:1 | Low (only task-critical facts retained) |

**Summarization prompt template:**

```
You are a conversation summarizer for a coding assistant. Preserve:
1. All code changes made (file paths, function names, line ranges)
2. All user decisions and preferences expressed
3. All errors encountered and their resolutions
4. Current task state and next planned steps

Compress everything else. Output structured JSON:
{
  "code_changes": [...],
  "decisions": [...],
  "errors_resolved": [...],
  "current_state": "...",
  "next_steps": [...]
}
```

### 3.5 Deterministic Mode

For debugging, testing, and auditing, OpenZax supports fully deterministic agent execution.

**Deterministic mode components:**

| Component | Mechanism |
|---|---|
| LLM seed | Fixed `seed` parameter in inference request; temperature set to 0 |
| Tool call recording | Every tool invocation is logged with inputs, outputs, and timing |
| Event replay | Recorded event streams can be replayed against a fresh agent instance |
| Filesystem snapshots | VFS overlay captures pre/post state for each tool call |
| Randomness | All `rand` calls routed through a seeded PRNG (`ChaCha8Rng` with fixed seed) |

**Recording format (JSONL):**

```json
{"type":"model_call","timestamp":"2026-03-01T12:00:00Z","model":"llama-3.3-70b-q4","prompt_hash":"sha256:abc...","seed":42,"response_hash":"sha256:def..."}
{"type":"tool_call","timestamp":"2026-03-01T12:00:01Z","tool":"fs_read","input":{"path":"src/main.rs"},"output_hash":"sha256:ghi...","duration_ms":2}
{"type":"tool_call","timestamp":"2026-03-01T12:00:02Z","tool":"fs_write","input":{"path":"src/main.rs","content_hash":"sha256:jkl..."},"output":{"success":true},"duration_ms":5}
```

**Replay verification:**

The replay engine feeds recorded model responses (by hash) and verifies that the agent produces identical tool call sequences. Any divergence is flagged as a non-determinism bug.

### 3.6 Self-Healing Workflows

Agents encounter failures — compilation errors, test failures, API timeouts, model hallucinations. OpenZax builds resilience into the execution model.

```mermaid
stateDiagram-v2
    [*] --> Executing
    Executing --> Checkpoint: periodic / pre-tool-call
    Checkpoint --> Executing: continue
    Executing --> Failed: error detected
    Failed --> Retry: retryable + attempts < max
    Retry --> Executing: backoff wait
    Failed --> Fallback: attempts exhausted
    Fallback --> AlternateStrategy: re-plan with error context
    AlternateStrategy --> Executing: new plan
    Fallback --> UserEscalation: no viable alternate
    Executing --> Completed: success
    Completed --> [*]
    UserEscalation --> [*]
```

**Checkpoint strategy:**

- **Pre-tool-call checkpoint:** Before any side-effecting tool call (file write, shell command, API request), the agent saves a checkpoint containing current plan state, conversation history, and VFS snapshot.
- **Periodic checkpoint:** Every 30 seconds of wall-clock execution time, regardless of tool calls.
- **Checkpoint storage:** SQLite table with BLOB columns for serialized agent state. Old checkpoints pruned after successful task completion (keep last 5).

**Retry policies:**

| Error Class | Max Retries | Backoff | Strategy |
|---|---|---|---|
| Compilation error | 3 | None (immediate) | Re-read error output; modify code; recompile |
| Test failure | 3 | None | Analyze failure; patch code or test; re-run |
| Model API timeout | 5 | Exponential (1s, 2s, 4s, 8s, 16s) | Retry same model; then fallback to alternate |
| Model rate limit | 10 | Respect `Retry-After` header | Queue request; switch to local model if available |
| WASM sandbox trap | 1 | None | Increase fuel/memory budget by 2x; retry once |
| Network error | 3 | Exponential (2s, 4s, 8s) | Retry; then switch to offline-capable alternative |

---

## 4. Visual Automation Engine

### 4.1 Node-Graph Editor

The visual automation engine provides a node-based programming interface for creating complex workflows without writing code.

```mermaid
graph LR
    subgraph "Node-Graph Canvas"
        TRIGGER[Trigger Node<br/>On file save in src/] --> LINT[Tool Node<br/>Run ESLint]
        LINT --> BRANCH{Branch Node<br/>Errors found?}
        BRANCH -->|yes| FIX[Agent Node<br/>Auto-fix lint errors]
        BRANCH -->|no| TEST[Tool Node<br/>Run test suite]
        FIX --> TEST
        TEST --> BRANCH2{Branch Node<br/>Tests pass?}
        BRANCH2 -->|yes| COMMIT[Tool Node<br/>Git commit]
        BRANCH2 -->|no| NOTIFY[Output Node<br/>Desktop notification]
    end

    style TRIGGER fill:#e74c3c,color:#fff
    style COMMIT fill:#27ae60,color:#fff
    style NOTIFY fill:#f39c12,color:#fff
```

**Node types:**

| Type | Description | Ports |
|---|---|---|
| **Trigger** | Initiates workflow execution | 0 inputs, 1+ outputs |
| **Tool** | Executes a single tool (MCP tool, shell command, API call) | 1+ inputs, 1+ outputs |
| **Agent** | Delegates to an AI agent with a prompt template | 1+ inputs, 1+ outputs |
| **Branch** | Conditional routing based on expression evaluation | 1 input, 2+ outputs (labeled) |
| **Merge** | Joins multiple branches back into a single flow | 2+ inputs, 1 output |
| **Loop** | Iterates over a collection, executing sub-graph per item | 1 input (collection), 1 output (results) |
| **Sub-Workflow** | Invokes a reusable workflow as a single node | Defined by sub-workflow's interface |
| **Transform** | Data transformation using JSONPath / Rhai scripting | 1 input, 1 output |
| **Output** | Terminal node: notification, file write, API response | 1 input, 0 outputs |
| **Error Handler** | Catches errors from connected nodes; defines recovery | 1 error input, 1+ outputs |

**Rendering architecture:**

The node graph is rendered in the Tauri WebView using an HTML5 Canvas with a Rust-backed layout engine. The layout engine (written in Rust, compiled to WASM) handles:

- Force-directed graph layout with constraint solving (Sugiyama-style for DAGs)
- Hit testing and selection (spatial index using R-Tree)
- Connection routing with orthogonal edge routing and crossing minimization
- Zoom, pan, and minimap rendering at 60 FPS

The Rust WASM layout engine communicates with the Leptos UI via shared memory (WASM linear memory), avoiding serialization overhead for the tight render loop.

### 4.2 Trigger System

| Trigger Type | Implementation | Example |
|---|---|---|
| **Cron** | `tokio-cron-scheduler` with timezone-aware scheduling | "Every weekday at 9am: summarize overnight PRs" |
| **Filesystem Watch** | `notify` crate (inotify / FSEvents / ReadDirectoryChangesW) | "On `*.rs` change in `src/`: run `cargo check`" |
| **Webhook** | axum HTTP endpoint with HMAC-SHA256 signature verification | "On GitHub push event: run CI pipeline" |
| **OS Event** | Platform-specific APIs (dbus on Linux, NSWorkspace on macOS, WMI on Windows) | "On screen unlock: sync latest emails" |
| **MCP Event** | Subscribe to MCP server notifications via SSE/WebSocket | "On Jira ticket assigned: create branch and scaffold" |
| **Manual** | Command palette / keyboard shortcut / API call | "Ctrl+Shift+W: run active workflow" |
| **Chain** | Output of another workflow triggers this one | "After deploy-workflow completes: run smoke-tests" |
| **Schedule Interval** | `tokio::time::interval` with drift compensation | "Every 5 minutes: check API health" |

**Trigger configuration (TOML):**

```toml
[[triggers]]
id = "on-rs-save"
type = "filesystem"
watch_paths = ["src/"]
patterns = ["*.rs"]
events = ["modify", "create"]
debounce_ms = 500
workflow = "lint-test-commit"

[[triggers]]
id = "nightly-summary"
type = "cron"
expression = "0 0 9 * * MON-FRI"
timezone = "America/New_York"
workflow = "morning-summary"

[[triggers]]
id = "github-webhook"
type = "webhook"
path = "/hooks/github"
secret_env = "GITHUB_WEBHOOK_SECRET"
events = ["push", "pull_request"]
workflow = "ci-pipeline"
```

### 4.3 Sub-Workflow Modules

Workflows can be composed from reusable sub-workflow modules with typed input/output ports.

**Module interface definition (WIT-inspired):**

```
module lint-and-fix {
    input {
        file-paths: list<string>,
        auto-fix: bool,
        severity-threshold: enum { error, warning, info },
    }

    output {
        fixed-count: u32,
        remaining-errors: list<lint-error>,
        modified-files: list<string>,
    }

    record lint-error {
        file: string,
        line: u32,
        column: u32,
        message: string,
        severity: string,
        rule: string,
    }
}
```

**Module versioning:**

- Each module has a semantic version (`major.minor.patch`).
- Breaking changes (input/output port additions/removals/type changes) require a major version bump.
- Workflows pin module versions in their manifest. Auto-upgrade is available for patch versions.
- The registry stores all versions; rollback to any previous version is one click.

### 4.4 Workflow Registry

```mermaid
graph TB
    subgraph "Workflow Registry (SQLite)"
        WF1[Workflow: lint-test-commit<br/>v2.3.1<br/>3 triggers, 7 nodes]
        WF2[Workflow: morning-summary<br/>v1.0.0<br/>1 trigger, 4 nodes]
        WF3[Workflow: ci-pipeline<br/>v5.1.2<br/>1 trigger, 12 nodes]
    end

    subgraph "Version History"
        V1[v5.1.2 — current]
        V2[v5.1.1 — bug fix]
        V3[v5.0.0 — breaking change]
        V4[v4.3.7 — previous stable]
    end

    WF3 --> V1
    V1 --> V2
    V2 --> V3
    V3 --> V4

    style WF1 fill:#3498db,color:#fff
    style WF2 fill:#3498db,color:#fff
    style WF3 fill:#3498db,color:#fff
    style V1 fill:#27ae60,color:#fff
```

**Registry schema:**

```sql
CREATE TABLE workflows (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    description TEXT,
    graph_json  TEXT NOT NULL,     -- serialized node graph
    triggers    TEXT NOT NULL,     -- JSON array of trigger configs
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    author      TEXT,
    tags        TEXT               -- JSON array of tags
);

CREATE TABLE workflow_versions (
    workflow_id TEXT NOT NULL REFERENCES workflows(id),
    version     TEXT NOT NULL,
    graph_json  TEXT NOT NULL,
    diff_json   TEXT,              -- JSON diff from previous version
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    changelog   TEXT,
    PRIMARY KEY (workflow_id, version)
);

CREATE TABLE workflow_runs (
    id          TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL REFERENCES workflows(id),
    version     TEXT NOT NULL,
    trigger_id  TEXT,
    status      TEXT NOT NULL,     -- 'running', 'completed', 'failed', 'cancelled'
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at TEXT,
    duration_ms INTEGER,
    node_results TEXT,             -- JSON object: { node_id: { status, output, duration_ms } }
    error       TEXT
);
```

### 4.5 Error Handling Architecture

```mermaid
graph TB
    NODE[Executing Node] -->|error| HANDLER{Error Handler<br/>attached?}
    HANDLER -->|yes| CATCH[Error Handler Node]
    HANDLER -->|no| PROPAGATE[Propagate to Parent]

    CATCH --> RETRY{Retry Policy}
    RETRY -->|retries remaining| BACKOFF[Backoff Wait]
    BACKOFF --> NODE

    RETRY -->|exhausted| FALLBACK{Fallback<br/>defined?}
    FALLBACK -->|yes| ALT[Alternate Node Path]
    FALLBACK -->|no| DLQ[Dead-Letter Queue<br/>SQLite table]

    PROPAGATE --> PARENT_HANDLER{Parent has<br/>handler?}
    PARENT_HANDLER -->|yes| PARENT_CATCH[Parent Error Handler]
    PARENT_HANDLER -->|no| WORKFLOW_FAIL[Workflow Failed<br/>Notification sent]

    DLQ --> INSPECT[Manual Inspection<br/>via Debug Console]

    style NODE fill:#3498db,color:#fff
    style DLQ fill:#e74c3c,color:#fff
    style WORKFLOW_FAIL fill:#e74c3c,color:#fff
    style ALT fill:#27ae60,color:#fff
```

**Dead-letter queue schema:**

```sql
CREATE TABLE dead_letter_queue (
    id            TEXT PRIMARY KEY,
    workflow_id   TEXT NOT NULL,
    workflow_run  TEXT NOT NULL,
    node_id       TEXT NOT NULL,
    error_type    TEXT NOT NULL,
    error_message TEXT NOT NULL,
    node_input    TEXT,              -- JSON snapshot of node input at failure time
    retry_count   INTEGER NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    resolved      BOOLEAN NOT NULL DEFAULT 0,
    resolved_at   TEXT,
    resolution    TEXT               -- 'retried', 'skipped', 'fixed', 'ignored'
);
```

**Retry policies per error type:**

| Error Type | Max Retries | Backoff | Notification |
|---|---|---|---|
| Network timeout | 5 | Exponential (1s base, 30s cap) | After 3rd retry |
| Authentication failure | 1 | None | Immediate (likely invalid credentials) |
| Rate limit | 10 | Respect `Retry-After` or exponential (5s base) | After 5th retry |
| Tool execution error | 3 | Linear (2s interval) | After exhaustion |
| Agent error (hallucination) | 2 | None (re-prompt with error context) | After exhaustion |
| WASM trap | 1 | None (increase resource limits) | Immediate |

---

## 5. Skills & Marketplace 2.0

### 5.1 WASM Component Model SDK

Skills are compiled to WebAssembly using the Component Model specification. Each skill is defined by a WIT (WASM Interface Types) interface that declares its capabilities, inputs, and outputs.

**WIT interface example (skill: `code-formatter`):**

```wit
package openzax:code-formatter@1.0.0;

interface formatter {
    record format-request {
        source: string,
        language: string,
        config: option<string>,
    }

    record format-result {
        formatted: string,
        changes-made: u32,
        diagnostics: list<diagnostic>,
    }

    record diagnostic {
        line: u32,
        column: u32,
        message: string,
        severity: severity-level,
    }

    enum severity-level {
        info,
        warning,
        error,
    }

    format: func(request: format-request) -> result<format-result, string>;
    supported-languages: func() -> list<string>;
}

world code-formatter {
    import openzax:host/logging;
    import openzax:host/config;

    export formatter;
}
```

**Host-provided interfaces (available to all skills):**

| Interface | Functions | Description |
|---|---|---|
| `openzax:host/logging` | `log(level, message)`, `span-start(name)`, `span-end()` | Structured logging forwarded to host tracing |
| `openzax:host/config` | `get(key) -> option<string>`, `set(key, value)` | Scoped key-value config per skill instance |
| `openzax:host/http-client` | `fetch(request) -> response` | HTTP requests (subject to URL allowlist in manifest) |
| `openzax:host/fs` | `read(path) -> bytes`, `write(path, bytes)`, `list(dir) -> entries` | Virtual filesystem (scoped to skill's sandbox directory) |
| `openzax:host/kv-store` | `get(key) -> option<bytes>`, `put(key, bytes)`, `delete(key)` | Persistent key-value storage (SQLite-backed, per-skill) |
| `openzax:host/events` | `emit(event-name, payload)`, `subscribe(pattern) -> stream` | Pub/sub on the host event bus (capability-gated) |

**SDK toolchain:**

```bash
# Install the SDK
cargo install openzax-sdk

# Create a new skill project
openzax skill init my-formatter --language rust
# Also supported: --language typescript, --language python

# Build to WASM component
openzax skill build
# Output: target/wasm32-wasip2/release/my_formatter.wasm

# Test locally with mock host
openzax skill test

# Package for marketplace
openzax skill pack
# Output: my-formatter-1.0.0.ozpkg (signed archive)
```

### 5.2 Cryptographic Signing Pipeline

Every skill package distributed through the marketplace is cryptographically signed using Ed25519 signatures.

```mermaid
sequenceDiagram
    participant Dev as Skill Developer
    participant CLI as openzax CLI
    participant KS as OS Keychain
    participant Reg as Marketplace API
    participant Scanner as Security Scanner
    participant Store as Package Store

    Dev->>CLI: openzax skill publish
    CLI->>CLI: Build WASM component
    CLI->>CLI: Compute SHA-256 of .wasm + manifest
    CLI->>KS: Retrieve Ed25519 signing key
    KS-->>CLI: Private key (never exported)
    CLI->>CLI: Sign package (Ed25519 detached signature)
    CLI->>Reg: Upload .ozpkg + signature + public key fingerprint
    Reg->>Scanner: Submit for automated scanning
    Scanner->>Scanner: Static analysis + behavioral sandbox
    Scanner-->>Reg: Scan result (pass/fail + report)
    Reg->>Store: Store verified package
    Store-->>Dev: Published! (version URL)
```

**Package format (`.ozpkg`):**

```
my-formatter-1.0.0.ozpkg
├── manifest.toml           # Skill metadata, version, capabilities, permissions
├── component.wasm           # Compiled WASM component (stripped, optimized)
├── signature.sig            # Ed25519 detached signature over SHA-256(manifest + wasm)
├── pubkey.pem               # Developer's Ed25519 public key
├── README.md                # Skill documentation
├── LICENSE                  # License file
└── assets/                  # Optional static assets (icons, configs)
    └── icon.svg
```

**Manifest schema (`manifest.toml`):**

```toml
[skill]
name = "code-formatter"
version = "1.0.0"
description = "Multi-language code formatter with configurable rules"
author = "jane@example.com"
license = "MIT"
repository = "https://github.com/jane/openzax-code-formatter"
homepage = "https://openzax.dev/skills/code-formatter"
keywords = ["formatter", "code-quality", "linting"]
min_openzax_version = "1.0.0"

[permissions]
filesystem = { read = ["**/*.rs", "**/*.ts", "**/*.py"], write = ["**/*.rs", "**/*.ts", "**/*.py"] }
network = { allow = [] }  # no network access needed
environment = []           # no env var access
subprocess = false         # cannot spawn processes
max_memory_mb = 8
max_cpu_fuel = 1_000_000_000

[dependencies]
"openzax:host/logging" = "1.0"
"openzax:host/config" = "1.0"
"openzax:host/fs" = "1.0"
```

### 5.3 Three-Tier Review System

```mermaid
graph TB
    SUBMIT[Developer Submits Skill] --> TIER1[Tier 1: Automated Scanning]

    subgraph "Tier 1 — Automated (Minutes)"
        TIER1 --> STATIC[Static Analysis<br/>WASM bytecode inspection<br/>Import/export validation]
        TIER1 --> SANDBOX[Behavioral Sandbox<br/>Run with instrumented host<br/>Monitor syscall patterns]
        TIER1 --> VULN[Vulnerability Scan<br/>Known CVE patterns<br/>Dependency audit]
    end

    STATIC --> GATE1{Pass automated<br/>checks?}
    SANDBOX --> GATE1
    VULN --> GATE1
    GATE1 -->|fail| REJECT1[Rejected with Report<br/>Developer notified]
    GATE1 -->|pass| TIER2[Tier 2: Community Audit]

    subgraph "Tier 2 — Community (Days)"
        TIER2 --> PEER[Peer Review<br/>3 community reviewers<br/>must approve]
        TIER2 --> REPUTATION[Reputation Check<br/>Developer history<br/>previous skill quality]
    end

    PEER --> GATE2{Community<br/>approved?}
    REPUTATION --> GATE2
    GATE2 -->|fail| REJECT2[Rejected with Feedback]
    GATE2 -->|pass| TIER3[Tier 3: Staff Review]

    subgraph "Tier 3 — Staff (Days)"
        TIER3 --> MANUAL[Manual Code Review<br/>OpenZax security team]
        TIER3 --> PENTEST[Permission Audit<br/>Verify least-privilege]
    end

    MANUAL --> GATE3{Staff<br/>approved?}
    PENTEST --> GATE3
    GATE3 -->|fail| REJECT3[Rejected with Guidance]
    GATE3 -->|pass| PUBLISH[Published to Marketplace<br/>Verified badge ✓]

    style SUBMIT fill:#3498db,color:#fff
    style PUBLISH fill:#27ae60,color:#fff
    style REJECT1 fill:#e74c3c,color:#fff
    style REJECT2 fill:#e74c3c,color:#fff
    style REJECT3 fill:#e74c3c,color:#fff
```

**Automated scanning checks (Tier 1):**

| Check | What It Detects | Severity |
|---|---|---|
| Import validation | Undeclared host imports; attempts to import blocked interfaces | Critical |
| Bytecode analysis | Known malicious WASM patterns (crypto miners, data exfiltration) | Critical |
| Manifest consistency | Declared permissions vs actual imports mismatch | High |
| Size analysis | Unreasonably large binaries (>10 MB) indicating embedded data | Medium |
| Dependency audit | Known vulnerable crate versions in build metadata | High |
| Behavioral sandbox | Network connections to unexpected hosts; excessive file I/O | Critical |
| Resource profiling | CPU and memory usage during test execution | Medium |

**Community reviewer incentives:**

- Reviewers earn reputation points for each completed review.
- Top reviewers (>100 reviews, >95% accuracy) receive "Trusted Reviewer" badge and early access to new features.
- Reviewers who miss a malicious skill lose reputation; reviewers who catch malicious skills gain bonus reputation.
- Monthly leaderboard with prizes (free Pro subscriptions, marketplace credits).

### 5.4 Capability-Scoped Permissions

No skill ever receives full system access. Every permission is explicitly declared in the manifest and enforced at runtime by the security kernel.

**Permission hierarchy:**

```mermaid
graph TB
    ROOT[System Root<br/>Full OS access<br/>NEVER granted to skills] --> FS[Filesystem<br/>Read / Write / Execute]
    ROOT --> NET[Network<br/>HTTP / WebSocket / DNS]
    ROOT --> PROC[Process<br/>Spawn / Signal / Inspect]
    ROOT --> ENV[Environment<br/>Variables / System Info]

    FS --> FS_READ[fs:read<br/>Scoped by glob pattern]
    FS --> FS_WRITE[fs:write<br/>Scoped by glob pattern]
    FS --> FS_EXEC[fs:execute<br/>Scoped by binary path]

    NET --> NET_HTTP[net:http<br/>Scoped by URL allowlist]
    NET --> NET_WS[net:websocket<br/>Scoped by host allowlist]
    NET --> NET_DNS[net:dns<br/>Resolve only, no exfiltration]

    PROC --> PROC_SPAWN[proc:spawn<br/>DISABLED by default<br/>Requires staff review]

    ENV --> ENV_READ[env:read<br/>Scoped by variable name list]

    style ROOT fill:#e74c3c,color:#fff
    style PROC_SPAWN fill:#e74c3c,color:#fff
    style FS_READ fill:#27ae60,color:#fff
    style NET_HTTP fill:#f39c12,color:#fff
```

**Runtime enforcement:**

1. On skill load, the security kernel parses the manifest's `[permissions]` table.
2. A `WasiCtx` is configured with only the declared capabilities:
   - Filesystem mounts are restricted to declared glob patterns using a virtual overlay.
   - Network access is filtered through a URL allowlist proxy.
   - Environment variable access is filtered to the declared list.
3. Any undeclared access attempt results in an immediate `Err(PermissionDenied)` return, logged to the audit trail.
4. Users can further restrict a skill's permissions below what the manifest declares (but never expand them).

### 5.5 Revenue Model & Economics

| Role | Revenue Split | Notes |
|---|---|---|
| Skill developer | 85% | Industry-leading split (vs 70% on most platforms) |
| Platform (OpenZax) | 15% | Covers hosting, CDN, review infrastructure |
| Community reviewers | Reputation + credits | Non-monetary initially; cash payout at scale |

**Pricing models for paid skills:**

| Model | Description | Example |
|---|---|---|
| One-time purchase | Pay once, own forever (including minor updates) | $9.99 for a premium formatter |
| Subscription | Monthly/annual recurring for ongoing updates | $2.99/mo for an AI code reviewer |
| Usage-based | Pay per invocation above a free tier | $0.001/call for an API integration skill |
| Freemium | Free base functionality; paid premium features | Free lint, $4.99/mo for auto-fix |

**Payout mechanics:**

- Payouts processed monthly via Stripe Connect.
- Minimum payout threshold: $10.
- Developers receive detailed analytics: installs, active users, revenue per day, retention curves.

---

## 6. Full MCP Supremacy

### 6.1 Native MCP Implementation

OpenZax implements the full Model Context Protocol specification natively in Rust, supporting both client and server modes.

**Transport layer support:**

| Transport | Role | Implementation |
|---|---|---|
| **stdio** | Connect to local MCP servers | `tokio::process::Command` with stdin/stdout pipes |
| **Streamable HTTP** | Connect to remote MCP servers / expose OpenZax as MCP server | axum with SSE streams (JSON-RPC over HTTP) |
| **WebSocket** | Persistent bidirectional connections | `tokio-tungstenite` with automatic reconnection |

**MCP capability matrix:**

| MCP Feature | Client Support | Server Support |
|---|---|---|
| Tools | Full (discovery, invocation, streaming results) | Full (register, handle, return) |
| Resources | Full (list, read, subscribe to changes) | Full (expose, notify on change) |
| Prompts | Full (list, get with arguments) | Full (register, template rendering) |
| Sampling | Full (request model completion from server) | Full (handle sampling requests) |
| Logging | Full (receive server logs) | Full (emit structured logs) |
| Roots | Full (provide workspace roots to servers) | N/A |
| Notifications | Full (resource changes, tool list changes) | Full (emit notifications) |

**MCP connection manager:**

```mermaid
graph TB
    subgraph "MCP Connection Pool"
        CM[Connection Manager<br/>Health checks + reconnection]
        CM --> S1[MCP Server: GitHub<br/>Transport: stdio<br/>Status: Connected ✓]
        CM --> S2[MCP Server: Database<br/>Transport: HTTP<br/>Status: Connected ✓]
        CM --> S3[MCP Server: Jira<br/>Transport: WebSocket<br/>Status: Reconnecting ↻]
        CM --> S4[MCP Server: Custom<br/>Transport: stdio<br/>Status: Connected ✓]
    end

    subgraph "Tool Registry (Unified)"
        TR[Aggregated Tool List<br/>Namespace: server/tool_name<br/>Deduplication + priority]
    end

    CM --> TR

    subgraph "Agent Runtime"
        AGENT[Agent<br/>Sees unified tool list<br/>Calls tools by namespace]
    end

    TR --> AGENT

    style CM fill:#3498db,color:#fff
    style TR fill:#27ae60,color:#fff
    style AGENT fill:#9b59b6,color:#fff
    style S3 fill:#f39c12,color:#fff
```

### 6.2 MCP Apps & UI Scheme

OpenZax supports MCP Apps — rich UI components served by MCP servers and rendered within the OpenZax interface.

**`ui://` scheme:**

```
ui://server-name/app-path?param=value
```

MCP servers can expose UI resources via the `ui://` scheme. These resources are HTML/CSS/JS bundles rendered in sandboxed iframes within the OpenZax workspace.

**Security model for MCP Apps:**

| Constraint | Enforcement |
|---|---|
| Sandboxed iframe | `sandbox="allow-scripts allow-forms"` (no `allow-same-origin`) |
| CSP header | `Content-Security-Policy: default-src 'self'; script-src 'self'` |
| Communication | `postMessage` API only, validated by origin check |
| Storage | No access to host `localStorage`/`indexedDB`; server-side storage only |
| Network | No direct network access from iframe; all requests proxied through MCP server |

**MCP App lifecycle:**

```mermaid
sequenceDiagram
    participant User
    participant OpenZax
    participant MCPServer
    participant Iframe

    User->>OpenZax: Open MCP App (ui://github/pr-dashboard)
    OpenZax->>MCPServer: resources/read { uri: "ui://github/pr-dashboard" }
    MCPServer-->>OpenZax: HTML bundle + metadata
    OpenZax->>Iframe: Render in sandboxed iframe
    Iframe->>OpenZax: postMessage({ type: "tool_call", tool: "github/list_prs" })
    OpenZax->>MCPServer: tools/call { name: "list_prs", args: {...} }
    MCPServer-->>OpenZax: Tool result
    OpenZax->>Iframe: postMessage({ type: "tool_result", data: [...] })
    Iframe-->>User: Rendered PR dashboard
```

### 6.3 Multi-Endpoint Orchestration

Agents can invoke tools across multiple MCP servers in a single workflow. The connection pool handles routing, load balancing, and failover.

**Connection pool configuration (`mcp-servers.toml`):**

```toml
[servers.github]
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_TOKEN = "${vault:github_token}" }
auto_connect = true
health_check_interval_s = 30
max_retries = 3

[servers.postgres]
transport = "http"
url = "http://localhost:3100/mcp"
headers = { Authorization = "Bearer ${vault:db_token}" }
auto_connect = true
connection_timeout_ms = 5000
request_timeout_ms = 30000

[servers.custom-api]
transport = "websocket"
url = "wss://api.example.com/mcp"
auto_reconnect = true
reconnect_backoff_ms = 1000
max_reconnect_attempts = 10
```

**Multi-server tool resolution:**

When multiple MCP servers expose tools with the same name, the connection manager uses a priority system:

1. **Explicit namespace:** `github/create_issue` always routes to the GitHub server.
2. **Priority ordering:** Servers are ordered by priority in config; first match wins for unqualified names.
3. **User disambiguation:** If ambiguity is detected, the command palette shows server name alongside tool name.

### 6.4 Transaction-Safe Tool Invocation

Tool calls that modify state (file writes, database mutations, API calls) are wrapped in a transaction-like protocol with rollback support.

```mermaid
sequenceDiagram
    participant Agent
    participant TxManager as Transaction Manager
    participant Tool1
    participant Tool2
    participant Journal

    Agent->>TxManager: begin_transaction()
    TxManager->>Journal: Create TX record (status: active)
    Agent->>TxManager: call_tool("fs_write", { path, content })
    TxManager->>Journal: Log pre-state (original file content)
    TxManager->>Tool1: Execute fs_write
    Tool1-->>TxManager: Success
    TxManager->>Journal: Log post-state (new file content)
    Agent->>TxManager: call_tool("git_commit", { message })
    TxManager->>Journal: Log pre-state (git HEAD)
    TxManager->>Tool2: Execute git_commit
    Tool2-->>TxManager: Failure (merge conflict)
    TxManager->>Journal: Log failure
    TxManager->>TxManager: ROLLBACK
    TxManager->>Tool1: Reverse fs_write (restore original content)
    TxManager->>Journal: Update TX record (status: rolled_back)
    TxManager-->>Agent: Transaction failed (with error context)
```

**Journal schema:**

```sql
CREATE TABLE tx_journal (
    tx_id       TEXT NOT NULL,
    seq         INTEGER NOT NULL,
    tool_name   TEXT NOT NULL,
    pre_state   BLOB,                -- serialized pre-invocation state
    post_state  BLOB,                -- serialized post-invocation state
    status      TEXT NOT NULL,        -- 'pending', 'committed', 'rolled_back'
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tx_id, seq)
);
```

**Rollback strategies per tool type:**

| Tool Type | Rollback Mechanism |
|---|---|
| File write | Restore original file content from journal `pre_state` |
| File delete | Recreate file from journal `pre_state` |
| File rename | Reverse rename |
| Git commit | `git reset --soft HEAD~1` |
| Git branch | `git branch -d` |
| HTTP POST/PUT | Log only (mark as non-reversible; warn user) |
| Database mutation | Execute compensating query from journal |
| Shell command | Log only (mark as non-reversible; warn user) |

### 6.5 Dev-Mode MCP Simulator

The MCP simulator enables skill and workflow development without connecting to real external services.

**Simulator capabilities:**

| Feature | Description |
|---|---|
| **Record mode** | Proxy real MCP server; record all requests/responses to JSONL file |
| **Replay mode** | Serve recorded responses for deterministic testing |
| **Mock mode** | Define mock responses in TOML for tools that don't exist yet |
| **Latency simulation** | Add configurable delays to simulate network latency |
| **Failure injection** | Randomly fail tool calls at configurable rates for resilience testing |
| **Schema validation** | Validate tool inputs against declared JSON Schema |

**Mock definition (`mocks/github.toml`):**

```toml
[[tools]]
name = "list_prs"
description = "List open pull requests"

[[tools.responses]]
match = { state = "open" }
response = '''
[
  { "id": 1, "title": "Add auth module", "author": "alice", "status": "review" },
  { "id": 2, "title": "Fix memory leak", "author": "bob", "status": "approved" }
]
'''
latency_ms = 150

[[tools.responses]]
match = { state = "closed" }
response = '''
[
  { "id": 3, "title": "Upgrade deps", "author": "carol", "status": "merged" }
]
'''
latency_ms = 100
```

**CLI usage:**

```bash
# Record mode: proxy a real server and save traffic
openzax mcp simulate --record --server github --output recordings/github.jsonl

# Replay mode: serve recorded responses
openzax mcp simulate --replay recordings/github.jsonl --port 3200

# Mock mode: serve hand-crafted mocks
openzax mcp simulate --mock mocks/github.toml --port 3200

# Chaos mode: inject failures at 10% rate with 200ms added latency
openzax mcp simulate --replay recordings/github.jsonl --fail-rate 0.1 --latency 200 --port 3200
```

---

## 7. Security Model (Enterprise-Grade)

### 7.1 Zero-Trust Capability Architecture

OpenZax operates on the principle of **zero ambient authority**. No component — agent, skill, workflow, or UI — has any implicit access to system resources. Every action requires an explicitly granted, scoped, time-limited capability token.

```mermaid
graph TB
    subgraph "Capability Authority"
        CA[Security Kernel<br/>Root Capability Store<br/>Policy Engine]
    end

    subgraph "Agents"
        A1[Agent 1<br/>Capabilities:<br/>fs:read(src/**)<br/>tool:call(lint)]
        A2[Agent 2<br/>Capabilities:<br/>fs:read(*)<br/>fs:write(src/**)<br/>net:http(api.github.com)]
    end

    subgraph "Skills (WASM)"
        SK1[Skill: linter<br/>Capabilities:<br/>fs:read(src/**)]
        SK2[Skill: deployer<br/>Capabilities:<br/>net:http(deploy.example.com)<br/>proc:spawn(docker)]
    end

    subgraph "Workflows"
        WF[Workflow: CI<br/>Inherits capabilities<br/>from trigger context]
    end

    CA -->|mint & delegate| A1
    CA -->|mint & delegate| A2
    A1 -->|subset delegation| SK1
    A2 -->|subset delegation| SK2
    CA -->|mint for trigger| WF

    style CA fill:#e74c3c,color:#fff
    style A1 fill:#3498db,color:#fff
    style A2 fill:#3498db,color:#fff
    style SK1 fill:#27ae60,color:#fff
    style SK2 fill:#27ae60,color:#fff
```

**Capability token structure:**

```rust
struct CapabilityToken {
    id: Uuid,
    issuer: ActorId,              // who minted this token (usually SecurityKernel)
    subject: ActorId,             // who this token is for (agent, skill, workflow)
    permissions: Vec<Permission>, // scoped resource access grants
    not_before: DateTime<Utc>,    // valid from
    expires_at: DateTime<Utc>,    // valid until (max 1 hour, renewable)
    parent_token: Option<Uuid>,   // delegation chain (for audit)
    signature: Ed25519Signature,  // signed by issuer's key
}

enum Permission {
    FsRead { glob: String },
    FsWrite { glob: String },
    FsExecute { path: String },
    NetHttp { url_pattern: String, methods: Vec<HttpMethod> },
    NetWebSocket { host: String },
    ToolCall { tool_name: String },
    AgentSpawn { max_budget: Budget },
    EnvRead { var_names: Vec<String> },
}
```

### 7.2 Signed Capability Tokens

Every capability token is cryptographically signed using Ed25519, preventing forgery and enabling audit trail verification.

**Token lifecycle:**

| Phase | Action | Verification |
|---|---|---|
| **Minting** | Security kernel creates token for authenticated request | Signed with kernel's Ed25519 private key |
| **Delegation** | Parent delegates subset to child (agent → skill) | New token signed by parent; references parent token ID |
| **Presentation** | Actor presents token with each privileged operation | Kernel verifies signature, expiry, scope match |
| **Revocation** | Kernel revokes token (on kill-switch, anomaly, expiry) | Token ID added to revocation set (bloom filter for O(1) lookup) |
| **Audit** | All token operations logged to tamper-evident audit log | Hash-chained entries; token ID + action + timestamp |

**Token verification (hot path, <1μs target):**

```rust
fn verify_capability(token: &CapabilityToken, required: &Permission) -> Result<(), SecurityError> {
    // 1. Check revocation (bloom filter, O(1))
    if self.revocation_set.might_contain(&token.id) {
        if self.revocation_list.contains(&token.id) {
            return Err(SecurityError::TokenRevoked);
        }
    }

    // 2. Check temporal validity
    let now = Utc::now();
    if now < token.not_before || now > token.expires_at {
        return Err(SecurityError::TokenExpired);
    }

    // 3. Verify signature (Ed25519, ~50μs but cached)
    if !self.verify_signature(token) {
        return Err(SecurityError::InvalidSignature);
    }

    // 4. Check permission scope
    if !token.permissions.iter().any(|p| p.satisfies(required)) {
        return Err(SecurityError::InsufficientPermission);
    }

    Ok(())
}
```

### 7.3 Virtual Filesystem Overlay

Agents and skills never interact with the host filesystem directly. All filesystem access goes through a virtual overlay that enforces access policies.

```mermaid
graph TB
    subgraph "Virtual FS Layer"
        VFS[Virtual FS Router<br/>Permission check per operation]
        VFS --> ALLOW[Allowlist Check<br/>Match path against<br/>capability token globs]
    end

    subgraph "Overlay Stack"
        ALLOW -->|allowed| OVERLAY[Union Overlay<br/>Merge sandbox writes<br/>with host reads]
        OVERLAY --> SANDBOX_FS[Sandbox FS<br/>Per-skill tmpdir<br/>Copy-on-write]
        OVERLAY --> HOST_FS[Host FS<br/>Read-only mount<br/>of allowed paths]
    end

    subgraph "Denied"
        ALLOW -->|denied| DENY[PermissionDenied<br/>Logged to audit trail]
    end

    style VFS fill:#e74c3c,color:#fff
    style SANDBOX_FS fill:#27ae60,color:#fff
    style HOST_FS fill:#3498db,color:#fff
    style DENY fill:#c0392b,color:#fff
```

**Overlay semantics:**

| Operation | Behavior |
|---|---|
| `read(path)` | Check allowlist → read from sandbox if modified, else read from host |
| `write(path)` | Check allowlist → write to sandbox copy (host file untouched until commit) |
| `delete(path)` | Check allowlist → mark as deleted in overlay (whiteout file) |
| `list(dir)` | Merge sandbox entries with host entries; apply whiteouts |
| `stat(path)` | Check sandbox first, fall back to host |
| `commit()` | Agent explicitly commits sandbox changes to host (after user approval) |

**Key security properties:**

- Skills can never read files outside their declared glob patterns.
- Skills can never modify host files directly; all writes go to a sandbox.
- The agent (or user) must explicitly commit sandbox changes to the host filesystem.
- Failed or aborted operations leave the host filesystem untouched.
- Symbolic links are resolved within the overlay, preventing symlink traversal attacks.

### 7.4 Encrypted Memory Store

Sensitive data (API keys, tokens, signing keys) is stored in an encrypted vault backed by the OS keychain.

**Encryption architecture:**

```mermaid
graph LR
    subgraph "Application Layer"
        APP[OpenZax Core<br/>Requests secret by key name]
    end

    subgraph "Vault Layer"
        VAULT[age Vault<br/>AES-256-GCM encrypted file<br/>~/.openzax/vault.age]
        VAULT --> DECRYPT[Decrypt with<br/>master key]
    end

    subgraph "Key Management"
        DECRYPT --> KEYCHAIN[OS Keychain<br/>Master key stored in:<br/>Windows: Credential Guard<br/>macOS: Keychain Services<br/>Linux: libsecret / KWallet]
    end

    APP --> VAULT
    DECRYPT --> APP

    style APP fill:#3498db,color:#fff
    style VAULT fill:#e74c3c,color:#fff
    style KEYCHAIN fill:#f39c12,color:#fff
```

**Vault operations:**

| Operation | Description | Security Property |
|---|---|---|
| `vault.get(key)` | Retrieve decrypted secret | Secret is decrypted in-memory only; zeroed after use via `zeroize` crate |
| `vault.set(key, value)` | Encrypt and store secret | Value encrypted with AES-256-GCM before writing to disk |
| `vault.delete(key)` | Remove secret | Overwrite with zeros before deletion; re-encrypt vault file |
| `vault.rotate_master_key()` | Re-encrypt vault with new master key | New X25519 keypair generated; old key material zeroed |
| `vault.export(path, passphrase)` | Export vault with passphrase encryption | For backup/migration; scrypt KDF with N=2^20 |
| `vault.import(path, passphrase)` | Import vault backup | Merge with existing vault; conflict resolution prompt |

**Memory safety for secrets:**

- All secret values are wrapped in a `Secret<T>` type that implements `Zeroize + ZeroizeOnDrop`.
- Secrets are never logged, serialized, or included in error messages.
- The `tracing` subscriber is configured with a `SecretRedactor` layer that replaces any accidentally-logged secret patterns with `[REDACTED]`.
- Core dumps are disabled on process startup (`prctl(PR_SET_DUMPABLE, 0)` on Linux, `SetProcessMitigationPolicy` on Windows).

### 7.5 Tamper-Evident Audit Log

Every privileged operation in OpenZax is recorded in a tamper-evident, append-only audit log.

**Log entry structure:**

```rust
struct AuditEntry {
    sequence: u64,                // monotonically increasing sequence number
    timestamp: DateTime<Utc>,
    actor: ActorId,               // who performed the action (agent, user, skill, system)
    action: AuditAction,          // what was done
    resource: String,             // what was affected
    outcome: AuditOutcome,        // success, denied, error
    capability_token_id: Option<Uuid>,
    details: Option<String>,      // additional context (JSON)
    prev_hash: [u8; 32],          // SHA-256 of previous entry (hash chain)
    entry_hash: [u8; 32],         // SHA-256 of this entry (including prev_hash)
}

enum AuditAction {
    FileRead, FileWrite, FileDelete,
    ToolCall, ToolResult,
    AgentSpawn, AgentTerminate,
    SkillLoad, SkillUnload,
    SecretAccess, SecretModify,
    ConfigChange,
    AuthAttempt, SessionCreate, SessionDestroy,
    CapabilityMint, CapabilityRevoke,
    WorkflowStart, WorkflowComplete, WorkflowFail,
}
```

**Hash chain integrity:**

```
Entry[0].entry_hash = SHA-256(Entry[0].data)
Entry[1].prev_hash  = Entry[0].entry_hash
Entry[1].entry_hash = SHA-256(Entry[1].data || Entry[1].prev_hash)
Entry[N].prev_hash  = Entry[N-1].entry_hash
Entry[N].entry_hash = SHA-256(Entry[N].data || Entry[N].prev_hash)
```

To verify integrity, traverse the chain and confirm that each entry's `prev_hash` matches the prior entry's `entry_hash`. Any tampering (modification, deletion, insertion) breaks the chain.

**Storage schema:**

```sql
CREATE TABLE audit_log (
    sequence        INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp       TEXT NOT NULL,
    actor_type      TEXT NOT NULL,
    actor_id        TEXT NOT NULL,
    action          TEXT NOT NULL,
    resource        TEXT NOT NULL,
    outcome         TEXT NOT NULL,
    capability_id   TEXT,
    details         TEXT,
    prev_hash       BLOB NOT NULL,
    entry_hash      BLOB NOT NULL
);

-- Append-only: no UPDATE or DELETE triggers
CREATE TRIGGER audit_no_update
BEFORE UPDATE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit log is append-only');
END;

CREATE TRIGGER audit_no_delete
BEFORE DELETE ON audit_log
BEGIN
    SELECT RAISE(ABORT, 'audit log is append-only');
END;

CREATE INDEX idx_audit_actor ON audit_log(actor_id, timestamp);
CREATE INDEX idx_audit_action ON audit_log(action, timestamp);
CREATE INDEX idx_audit_resource ON audit_log(resource, timestamp);
```

### 7.6 Kill-Switch & Containment

The kill-switch enables instant termination of any agent, skill, or workflow with full state preservation.

**Kill-switch triggers:**

| Trigger | Automation Level | Example |
|---|---|---|
| User hotkey | Manual (`Ctrl+Shift+K`) | User sees suspicious agent behavior |
| Anomaly detection | Automatic | Skill exceeds declared file I/O by 10x |
| Budget exhaustion | Automatic | Agent exceeds token or time budget |
| Policy violation | Automatic | Agent attempts to access path outside capability scope |
| Watchdog timeout | Automatic | Agent unresponsive for >30 seconds |

**Kill-switch execution sequence:**

```mermaid
sequenceDiagram
    participant Trigger
    participant SecurityKernel
    participant AgentRuntime
    participant WASMSandbox
    participant Storage
    participant UI

    Trigger->>SecurityKernel: KILL signal (target: agent-42)
    SecurityKernel->>SecurityKernel: Revoke all capability tokens for agent-42
    SecurityKernel->>AgentRuntime: Cancel agent-42 task
    AgentRuntime->>AgentRuntime: Set CancellationToken
    AgentRuntime->>WASMSandbox: Interrupt WASM execution (epoch interrupt)
    WASMSandbox->>WASMSandbox: Trap & halt
    AgentRuntime->>Storage: Save agent state checkpoint
    Storage-->>AgentRuntime: Checkpoint saved
    AgentRuntime->>Storage: Log kill event to audit trail
    AgentRuntime-->>SecurityKernel: Agent terminated
    SecurityKernel->>UI: Push notification (agent killed, reason, checkpoint ID)
    UI-->>Trigger: Confirmation displayed
```

**State preservation on kill:**

- Current plan DAG state (completed/in-progress/pending nodes)
- Conversation history up to termination point
- VFS sandbox state (uncommitted file changes)
- Tool call journal (for potential rollback)
- Agent's internal scratchpad / working memory

The user can inspect the preserved state, decide whether to commit or rollback filesystem changes, and optionally resume the agent from the checkpoint.

### 7.7 Behavioral Anomaly Detection

OpenZax continuously monitors agent and skill behavior, comparing runtime metrics against expected baselines derived from skill manifests.

**Monitored metrics:**

| Metric | Baseline Source | Anomaly Threshold |
|---|---|---|
| File read count/rate | Manifest `permissions.filesystem.read` | >5x declared glob count per minute |
| File write count/rate | Manifest `permissions.filesystem.write` | >3x declared glob count per minute |
| Network request count | Manifest `permissions.network.allow` | Any request to undeclared host |
| CPU fuel consumption | Manifest `max_cpu_fuel` | >80% of budget in first 10% of execution |
| Memory growth rate | Manifest `max_memory_mb` | >50% of limit in first 20% of execution |
| Tool call diversity | Historical usage profile | >3 standard deviations from mean tool call pattern |
| Data exfiltration proxy | Network payload size | Outbound payload >10x inbound (potential exfiltration) |

**Detection algorithm:**

The anomaly detector uses a simple but effective statistical model:

1. **Baseline calculation:** For each metric, compute the rolling mean (μ) and standard deviation (σ) over the last 100 invocations of the skill.
2. **Z-score monitoring:** On each measurement, compute `z = (x - μ) / σ`. If `|z| > 3.0`, flag as anomalous.
3. **Compound scoring:** Multiple low-severity anomalies compound. If 3+ metrics are simultaneously at `|z| > 2.0`, escalate to high severity.
4. **Cooldown:** After triggering, a 5-minute cooldown prevents alert fatigue.

### 7.8 Automatic Quarantine System

Skills that trigger anomaly detection are automatically quarantined pending investigation.

**Quarantine process:**

```mermaid
stateDiagram-v2
    [*] --> Active: Skill installed
    Active --> Flagged: Anomaly detected
    Flagged --> Quarantined: Auto-quarantine (severity >= HIGH)
    Flagged --> Active: False positive (severity LOW, auto-clear after cooldown)
    Quarantined --> Review: Manual review triggered
    Review --> Reinstated: Review passed (clear anomaly, update baseline)
    Review --> Removed: Review failed (malicious / buggy)
    Reinstated --> Active: Restored
    Removed --> [*]: Uninstalled + reported to marketplace
```

**Quarantine enforcement:**

- Quarantined skills have all capability tokens revoked immediately.
- The WASM sandbox is suspended (not destroyed) to preserve forensic state.
- All pending tool calls from the quarantined skill are rejected with `SkillQuarantined` error.
- The user receives a notification with the anomaly report and options: "Review", "Uninstall", "Whitelist (expert mode)".
- Quarantine events are reported to the marketplace for aggregate analysis (e.g., if 10+ users quarantine the same skill version, trigger marketplace-wide review).

---

## 8. Premium UX & Interface

### 8.1 Rendering Pipeline

OpenZax achieves sub-16ms frame times (60+ FPS) by leveraging the OS-native WebView and minimizing IPC overhead.

**Rendering architecture:**

```mermaid
graph LR
    subgraph "Rust Core (Main Thread)"
        EVENTS[Event Bus<br/>Cap'n Proto messages]
        SERIALIZE[Serialize UI State<br/>Cap'n Proto → Tauri IPC]
    end

    subgraph "WebView (UI Thread)"
        DESERIALIZE[Deserialize<br/>Cap'n Proto → JS objects]
        LEPTOS[Leptos Reactive Runtime<br/>Fine-grained DOM updates]
        DOM[DOM<br/>Minimal mutations]
        COMPOSITOR[OS Compositor<br/>Hardware-accelerated]
    end

    EVENTS --> SERIALIZE
    SERIALIZE -->|"Tauri IPC<br/>~100μs per message"| DESERIALIZE
    DESERIALIZE --> LEPTOS
    LEPTOS --> DOM
    DOM --> COMPOSITOR

    style EVENTS fill:#e74c3c,color:#fff
    style LEPTOS fill:#3498db,color:#fff
    style COMPOSITOR fill:#27ae60,color:#fff
```

**Performance budget:**

| Component | Budget | Actual Target |
|---|---|---|
| IPC serialization (Cap'n Proto) | <1ms | ~100μs (zero-copy) |
| IPC transfer (Tauri bridge) | <1ms | ~200μs |
| Reactive update (Leptos) | <5ms | ~2ms (fine-grained signals) |
| DOM mutation | <5ms | ~3ms (batch updates) |
| Paint + composite | <5ms | ~3ms (OS compositor) |
| **Total frame time** | **<16ms** | **~8ms typical** |

**Optimization techniques:**

- **Fine-grained reactivity:** Leptos uses signals that update only the specific DOM nodes that depend on changed data, not entire component subtrees.
- **Virtualized lists:** Chat messages, file trees, and search results use virtual scrolling. Only visible items are rendered in the DOM (~30 items), regardless of list length.
- **Debounced updates:** High-frequency events (token streaming, file watch) are debounced to a maximum update rate of 60 FPS.
- **Web Worker offload:** Syntax highlighting and Markdown rendering run in a dedicated Web Worker to avoid blocking the UI thread.

### 8.2 Multi-Panel Workspace

```mermaid
graph TB
    subgraph "OpenZax Workspace Layout"
        subgraph "Left Sidebar"
            EXPLORER[File Explorer<br/>Tree view]
            SKILLS_PANEL[Skills Panel<br/>Installed + marketplace]
            MCP_PANEL[MCP Servers<br/>Connection status]
        end

        subgraph "Center Panel (Tabs)"
            CHAT[Chat View<br/>Agent conversation]
            EDITOR[Code Editor<br/>Monaco-based]
            WORKFLOW[Workflow Editor<br/>Node graph canvas]
            DIFF[Diff Viewer<br/>Side-by-side / unified]
        end

        subgraph "Right Sidebar"
            CONTEXT[Context Panel<br/>Active files, symbols]
            ACTIVITY[Agent Activity<br/>Live feed]
            PERMISSIONS[Permission Dashboard<br/>Active capabilities]
        end

        subgraph "Bottom Panel"
            TERMINAL[Terminal<br/>Integrated shell]
            OUTPUT[Output<br/>Build / test logs]
            DEBUG[Debug Console<br/>Performance monitor]
            AUDIT_VIEW[Audit Log<br/>Real-time entries]
        end
    end

    style CHAT fill:#3498db,color:#fff
    style WORKFLOW fill:#9b59b6,color:#fff
    style PERMISSIONS fill:#e74c3c,color:#fff
    style DEBUG fill:#f39c12,color:#fff
```

**Layout system:**

- **Drag-and-drop panels:** Users can drag panels to any position, creating custom layouts. Layouts are persisted per-workspace.
- **Split views:** Any panel can be split horizontally or vertically (up to 4 levels deep).
- **Floating panels:** Panels can be detached into floating windows (Tauri multi-window support).
- **Presets:** Built-in layout presets: "Focus" (chat only), "Code" (editor + terminal), "Workflow" (graph + output), "Debug" (all panels).
- **Responsive:** Panels auto-collapse below minimum width thresholds; sidebar toggles via keyboard shortcuts.

### 8.3 Command Palette

The command palette is the fastest way to access any OpenZax functionality, targeting sub-50ms response time from keystroke to rendered results.

**Architecture:**

```mermaid
graph LR
    KEYSTROKE[User types<br/>Ctrl+Shift+P] --> INPUT[Input Field<br/>Focus + debounce 16ms]
    INPUT --> FUZZY[Fuzzy Matcher<br/>Rust WASM module<br/>Nucleo algorithm]
    FUZZY --> RANK[Rank Results<br/>Recency + frequency<br/>+ match quality]
    RANK --> RENDER[Render List<br/>Top 10 results<br/>Virtual scroll]
    RENDER --> SELECT[User selects<br/>Enter / Click]
    SELECT --> DISPATCH[Command Dispatch<br/>Execute action]

    style KEYSTROKE fill:#3498db,color:#fff
    style FUZZY fill:#27ae60,color:#fff
    style DISPATCH fill:#9b59b6,color:#fff
```

**Command sources (unified):**

| Source | Example Commands | Count |
|---|---|---|
| Built-in commands | "Toggle sidebar", "Open settings", "Kill agent" | ~100 |
| MCP tools | "github/create_issue", "postgres/query" | Dynamic (per connected server) |
| Installed skills | "Format code", "Run linter", "Generate tests" | Dynamic (per installed skill) |
| Workflows | "Run: lint-test-commit", "Edit: morning-summary" | Dynamic (per defined workflow) |
| Recent files | "Open: src/main.rs", "Open: docs/readme.md" | Last 50 files |
| Symbols | "Go to: fn main", "Go to: struct Agent" | Indexed from workspace |

**Fuzzy matching implementation:**

The fuzzy matcher uses the Nucleo algorithm (same as used by Helix editor) compiled to WASM for in-WebView execution. This avoids IPC round-trips for keystroke-level interactions.

- **Indexing:** On workspace load, all command sources are indexed into a flat list with metadata (category, icon, last-used timestamp, frequency count).
- **Scoring:** Nucleo provides character-level match quality. OpenZax adds a composite score: `final_score = 0.6 * match_quality + 0.2 * recency + 0.2 * frequency`.
- **Incremental:** As the user types, results are filtered incrementally (not recomputed from scratch).

### 8.4 Live Agent Activity Feed

The activity feed provides real-time visibility into what agents are doing, enabling trust through transparency.

**Feed entry types:**

| Entry Type | Display | Example |
|---|---|---|
| Thinking | Collapsible thought bubble with spinner | "Analyzing the authentication flow..." |
| Tool call | Tool icon + name + expandable params/result | "fs_read(src/auth.rs) → 142 lines" |
| Code change | Inline diff with file path | "+15 -3 lines in src/auth.rs" |
| Model switch | Model badge with reason | "Switched to Claude 4 Sonnet (reasoning task)" |
| Sub-agent spawn | Nested card with budget indicator | "Spawned sub-agent: 'Write tests' (20K tokens)" |
| Error | Red banner with error message | "Compilation failed: missing semicolon on line 42" |
| Checkpoint | Checkpoint icon with restore button | "Checkpoint saved (ID: cp-abc123)" |
| Permission request | Amber card with approve/deny buttons | "Agent requests: fs:write(src/**)" |

**Streaming implementation:**

Token streaming from the AI model is rendered character-by-character using a `ReadableStream` bridge from the Tauri IPC:

1. Rust core receives tokens from the AI router via async channel.
2. Tokens are batched (max 16ms / 60 FPS) and serialized as Cap'n Proto `TokenBatch` messages.
3. Tauri IPC pushes batches to the WebView via event emission.
4. Leptos signal updates trigger fine-grained DOM text node appends.
5. Auto-scroll keeps the latest content visible (unless the user has scrolled up to read history).

### 8.5 Permission Transparency Dashboard

```mermaid
graph TB
    subgraph "Permission Dashboard"
        subgraph "Active Agents"
            AG1[Agent: Code Assistant<br/>[ok] fs:read(src/**)<br/>[ok] tool:call(compile)<br/>net:http — not granted]
        end

        subgraph "Active Skills"
            SK1[Skill: linter v2.1.0<br/>[ok] fs:read(**/*.rs)<br/>Expires in 45m]
            SK2[Skill: git-helper v1.3.2<br/>[ok] proc:spawn(git)<br/>[ok] fs:read(.git/**)<br/>Expires in 30m]
        end

        subgraph "Recent Permission Events"
            EVT1["12:01 — Agent granted fs:read(src/**)"]
            EVT2["12:02 — Skill 'formatter' denied net:http(evil.com)"]
            EVT3["[warn] 12:03 — Agent capability token renewed (45m)"]
        end
    end

    style AG1 fill:#3498db,color:#fff
    style SK1 fill:#27ae60,color:#fff
    style SK2 fill:#27ae60,color:#fff
    style EVT2 fill:#e74c3c,color:#fff
```

**Dashboard features:**

- **Real-time capability view:** Shows every active capability token, its scope, and remaining TTL.
- **Permission history:** Timeline of all grant/deny/revoke events with filtering by actor, action, and time range.
- **Anomaly alerts:** Inline alerts when behavioral anomaly detection flags suspicious activity.
- **One-click revoke:** Revoke any active capability token directly from the dashboard.
- **Export:** Export permission history as CSV or JSON for compliance auditing.

### 8.6 Debug Console & Performance Monitor

**Debug Console features:**

| Feature | Description |
|---|---|
| Event inspector | Browse and filter events on the Cap'n Proto event bus in real-time |
| WASM profiler | CPU fuel consumption and memory usage per skill instance |
| IPC latency graph | Rolling graph of IPC message latency (target: <1ms) |
| Model request log | Every AI model request with prompt size, latency, token count, cost |
| SQLite query log | All database queries with execution time and query plan |
| Network monitor | HTTP/WebSocket requests with timing, status, and payload size |

**Performance monitor widgets:**

```
┌─────────────────────────────────────────────────────────┐
│ Performance Monitor                                      │
├──────────────┬──────────────┬──────────────┬────────────┤
│ CPU          │ Memory       │ Frame Time   │ IPC        │
│ ████░░ 38%  │ ██░░░░ 28MB  │ ██░░░░ 8ms   │ █░░░░ 0.2ms│
│ (agent task) │ (of 128MB)   │ (target 16ms)│ (avg)      │
├──────────────┴──────────────┴──────────────┴────────────┤
│ AI Router                                                │
│ Tokens/s: 45.2 │ Active model: llama-3.3-70b-q4         │
│ Queue depth: 0  │ GPU VRAM: 4.2 GB / 8 GB               │
├──────────────────────────────────────────────────────────┤
│ WASM Sandboxes (3 active)                                │
│ linter:     fuel 2.3M / 1B  │ mem 1.2 MB / 8 MB        │
│ formatter:  fuel 890K / 1B  │ mem 0.8 MB / 8 MB        │
│ git-helper: fuel 150K / 1B  │ mem 0.4 MB / 4 MB        │
└─────────────────────────────────────────────────────────┘
```

### 8.7 Theme Engine

The theme engine uses CSS custom properties for near-instant theme switching with zero layout recalculation.

**Theme schema:**

```css
:root {
    /* Core palette */
    --oz-bg-primary: #1a1a2e;
    --oz-bg-secondary: #16213e;
    --oz-bg-tertiary: #0f3460;
    --oz-text-primary: #e0e0e0;
    --oz-text-secondary: #a0a0a0;
    --oz-text-accent: #00d4ff;

    /* Semantic colors */
    --oz-success: #2ecc71;
    --oz-warning: #f39c12;
    --oz-error: #e74c3c;
    --oz-info: #3498db;

    /* Interactive elements */
    --oz-button-bg: #0f3460;
    --oz-button-hover: #1a4a7a;
    --oz-button-active: #245a8a;
    --oz-input-bg: #16213e;
    --oz-input-border: #2a3a5e;
    --oz-input-focus: #00d4ff;

    /* Layout */
    --oz-sidebar-width: 260px;
    --oz-panel-border: 1px solid #2a3a5e;
    --oz-border-radius: 6px;
    --oz-spacing-unit: 4px;

    /* Typography */
    --oz-font-mono: 'JetBrains Mono', 'Fira Code', monospace;
    --oz-font-sans: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    --oz-font-size-sm: 12px;
    --oz-font-size-md: 14px;
    --oz-font-size-lg: 16px;

    /* Transitions */
    --oz-transition-fast: 100ms ease-out;
    --oz-transition-normal: 200ms ease-out;
}
```

**Built-in themes:**

| Theme | Description | Target Audience |
|---|---|---|
| **Midnight** (default) | Dark blue-black with cyan accents | Most developers |
| **Daylight** | Warm light theme with high contrast | Light-mode users |
| **Solarized Dark** | Ethan Schoonover's Solarized palette | Solarized fans |
| **High Contrast** | Maximum contrast, bold borders, large text | Accessibility / visual impairment |
| **Monochrome** | Grayscale with single accent color | Minimalists |

**Custom theme creation:**

Users can create custom themes by overriding CSS custom properties in a TOML file:

```toml
[theme]
name = "My Custom Theme"
base = "midnight"

[colors]
bg-primary = "#0d1117"
bg-secondary = "#161b22"
text-accent = "#58a6ff"
success = "#3fb950"
error = "#f85149"
```

### 8.8 Accessibility Compliance

OpenZax targets WCAG 2.1 Level AA compliance across all UI components.

**Compliance checklist:**

| Criterion | Implementation |
|---|---|
| **1.1.1 Non-text content** | All icons have `aria-label`; images have `alt` text |
| **1.3.1 Info and relationships** | Semantic HTML5 elements; ARIA landmarks for regions |
| **1.4.3 Contrast (AA)** | Minimum 4.5:1 text contrast; 3:1 for large text; enforced in theme engine |
| **1.4.4 Resize text** | UI scales to 200% without loss of content or functionality |
| **2.1.1 Keyboard** | All functionality accessible via keyboard; visible focus indicators |
| **2.1.2 No keyboard trap** | Focus management ensures escape from all modals and panels |
| **2.4.3 Focus order** | Tab order follows logical reading order; skip-to-content link |
| **2.4.7 Focus visible** | Custom focus ring (`outline: 2px solid var(--oz-input-focus)`) on all interactive elements |
| **3.1.1 Language** | `lang` attribute on `<html>`; language detection for code blocks |
| **4.1.2 Name, role, value** | All custom components expose ARIA roles and states |

**Screen reader support:**

- Live regions (`aria-live="polite"`) for agent activity feed updates.
- `aria-live="assertive"` for error notifications and kill-switch confirmations.
- Custom `role="log"` for the chat conversation view.
- `role="treegrid"` for the file explorer with keyboard navigation.

**Keyboard shortcuts:**

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+P` | Command palette |
| `Ctrl+Shift+K` | Kill active agent |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+J` | Toggle bottom panel |
| `Ctrl+\` | Split editor |
| `F6` | Cycle focus between panels |
| `Escape` | Close modal / deselect / cancel |
| `Ctrl+,` | Open settings |
| `Ctrl+Shift+D` | Toggle debug console |

---

## 9. Developer Platform

### 9.1 CLI Toolchain

The `openzax` CLI is a single static binary (no runtime dependencies) that provides the complete skill development lifecycle.

**Command reference:**

```
openzax 1.0.0 — OpenZax Developer CLI

USAGE:
    openzax <COMMAND>

COMMANDS:
    init        Create a new skill project from template
    build       Compile skill to WASM component
    test        Run skill tests with mock host
    sign        Sign a skill package with Ed25519 key
    publish     Build, sign, and upload to marketplace
    pack        Create .ozpkg without publishing
    inspect     Inspect a .wasm or .ozpkg file (imports, exports, manifest)
    validate    Validate manifest.toml against schema
    keygen      Generate Ed25519 signing keypair
    login       Authenticate with marketplace API
    whoami      Display current authenticated identity
    search      Search marketplace for skills
    install     Install a skill from marketplace to local OpenZax
    mcp         MCP development tools (simulate, inspect, record)
    doctor      Diagnose common development environment issues
    upgrade     Self-update to latest CLI version

GLOBAL FLAGS:
    -v, --verbose    Increase log verbosity (repeat for more: -vvv)
    -q, --quiet      Suppress non-error output
    --json           Output in JSON format (for scripting)
    --color <WHEN>   Color output: auto, always, never [default: auto]
    -h, --help       Print help
    -V, --version    Print version
```

**Detailed command usage:**

```bash
# Initialize a new Rust skill project
openzax init my-skill --language rust --template tool
# Templates: tool, resource-provider, workflow-node, mcp-bridge

# Build with optimization
openzax build --release
# Runs: cargo build --target wasm32-wasip2 --release
# Then: wasm-opt -O3 on the output
# Output: target/wasm32-wasip2/release/my_skill.wasm (typically <500 KB)

# Test with mock host and coverage
openzax test --coverage
# Spins up in-process Wasmtime with mock host interfaces
# Runs #[openzax_test] annotated functions
# Reports: pass/fail + coverage percentage

# Sign the package
openzax sign target/wasm32-wasip2/release/my_skill.wasm
# Reads Ed25519 private key from OS keychain
# Produces: my_skill.sig (detached signature)

# One-command publish
openzax publish
# Equivalent to: build --release && sign && pack && upload
# Requires: openzax login (one-time)

# Inspect a skill package
openzax inspect my-skill-1.0.0.ozpkg
# Output: manifest, WASM imports/exports, signature verification, size breakdown
```

### 9.2 Multi-Language SDKs

While the core runtime is Rust, skill developers can write skills in multiple languages that compile to WASM.

**Rust SDK (`openzax-sdk` crate):**

```rust
use openzax_sdk::prelude::*;

#[openzax_skill]
struct MyFormatter;

#[openzax_export]
impl Formatter for MyFormatter {
    fn format(&self, request: FormatRequest) -> Result<FormatResult, String> {
        let formatted = apply_formatting_rules(&request.source, &request.language)?;
        Ok(FormatResult {
            formatted,
            changes_made: 1,
            diagnostics: vec![],
        })
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["rust".into(), "typescript".into(), "python".into()]
    }
}
```

**TypeScript SDK (`@openzax/sdk` npm package):**

```typescript
import { defineSkill, FormatRequest, FormatResult } from "@openzax/sdk";

export default defineSkill({
  format(request: FormatRequest): FormatResult {
    const formatted = applyFormattingRules(request.source, request.language);
    return {
      formatted,
      changesMade: 1,
      diagnostics: [],
    };
  },

  supportedLanguages(): string[] {
    return ["rust", "typescript", "python"];
  },
});
```

TypeScript skills are compiled to WASM using ComponentizeJS (Bytecode Alliance project) which bundles a JavaScript engine (StarlingMonkey) into the WASM component.

**Python SDK (`openzax-sdk` PyPI package):**

```python
from openzax_sdk import skill, FormatRequest, FormatResult

@skill
class MyFormatter:
    def format(self, request: FormatRequest) -> FormatResult:
        formatted = apply_formatting_rules(request.source, request.language)
        return FormatResult(
            formatted=formatted,
            changes_made=1,
            diagnostics=[],
        )

    def supported_languages(self) -> list[str]:
        return ["rust", "typescript", "python"]
```

Python skills are compiled to WASM using componentize-py (Bytecode Alliance project) which bundles a CPython interpreter into the WASM component.

**SDK size comparison:**

| Language | WASM Binary Size (optimized) | Cold Start | Memory Overhead |
|---|---|---|---|
| Rust | ~100–500 KB | ~1ms | ~1 MB |
| TypeScript | ~3–5 MB (includes JS engine) | ~50ms | ~8 MB |
| Python | ~10–15 MB (includes CPython) | ~200ms | ~20 MB |

### 9.3 Extension Debugger

The extension debugger provides full visibility into WASM skill execution with breakpoint-level inspection.

**Debugger capabilities:**

| Feature | Implementation |
|---|---|
| **Breakpoints** | Set breakpoints on WASM function entry/exit via Wasmtime debug hooks |
| **Step execution** | Step through WASM instructions (or source-mapped lines with DWARF info) |
| **Memory inspector** | View WASM linear memory as hex dump, typed values, or string view |
| **Fuel monitor** | Real-time fuel consumption graph with per-function breakdown |
| **Import/export inspector** | List all host imports and skill exports with call counts |
| **Host call trace** | Trace all calls to host interfaces (fs, http, kv-store) with timing |
| **Source maps** | DWARF debug info maps WASM instructions back to Rust/TS/Python source lines |

**Debugger UI (integrated into bottom panel):**

```
┌──────────────────────────────────────────────────────────────┐
│ Extension Debugger — linter v2.1.0                           │
├──────────────────────┬───────────────────────────────────────┤
│ Call Stack           │ Source (src/lib.rs:42)                 │
│ ▸ format()           │ 40│ fn format(&self, req: FormatReq) { │
│   └ apply_rules()    │ 41│   let rules = self.load_rules()?;  │
│     └ check_indent() │►42│   for rule in &rules {             │
│                      │ 43│     rule.apply(&mut source)?;       │
│──────────────────────│ 44│   }                                │
│ Variables            │                                       │
│ req.language: "rust" │───────────────────────────────────────│
│ rules.len(): 12      │ Fuel: 234,567 / 1,000,000,000        │
│ source.len(): 1,542  │ Memory: 1.2 MB / 8 MB                │
└──────────────────────┴───────────────────────────────────────┘
```

### 9.4 Test Harness

The test harness provides a controlled environment for testing skills without connecting to real services or affecting the host system.

**Test harness architecture:**

```mermaid
graph TB
    subgraph "Test Runner (openzax test)"
        RUNNER[Test Runner<br/>Discovers and executes test functions]
    end

    subgraph "Mock Host Environment"
        MOCK_FS[Mock Filesystem<br/>In-memory VFS with preloaded fixtures]
        MOCK_HTTP[Mock HTTP Client<br/>Responds with recorded/configured responses]
        MOCK_KV[Mock KV Store<br/>In-memory HashMap]
        MOCK_EVENTS[Mock Event Bus<br/>Captures emitted events for assertions]
        MOCK_MCP[Mock MCP Server<br/>Configurable tool responses]
    end

    subgraph "Assertion Library"
        ASSERT[Assertions<br/>assert_output_eq, assert_event_emitted<br/>assert_fs_written, assert_no_errors]
    end

    RUNNER --> MOCK_FS
    RUNNER --> MOCK_HTTP
    RUNNER --> MOCK_KV
    RUNNER --> MOCK_EVENTS
    RUNNER --> MOCK_MCP
    RUNNER --> ASSERT

    style RUNNER fill:#3498db,color:#fff
    style ASSERT fill:#27ae60,color:#fff
```

**Test example (Rust):**

```rust
#[cfg(test)]
mod tests {
    use openzax_sdk::testing::*;

    #[openzax_test]
    async fn test_format_rust_code() {
        let mut harness = TestHarness::new()
            .with_fs_fixture("src/main.rs", "fn  main ( ) { }")
            .build();

        let result = harness.call::<Formatter>("format", FormatRequest {
            source: "fn  main ( ) { }".into(),
            language: "rust".into(),
            config: None,
        }).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.formatted, "fn main() {}");
        assert_eq!(result.changes_made, 1);
        assert!(result.diagnostics.is_empty());
    }

    #[openzax_test]
    async fn test_permission_denied_for_undeclared_path() {
        let mut harness = TestHarness::new()
            .with_permissions(Permissions {
                fs_read: vec!["src/**/*.rs".into()],
                ..Default::default()
            })
            .build();

        let result = harness.host_call::<FsRead>("/etc/passwd").await;
        assert!(matches!(result, Err(HostError::PermissionDenied)));
    }
}
```

### 9.5 Documentation Generation

API documentation is auto-generated from WIT interfaces with zero manual effort.

**Documentation pipeline:**

```mermaid
graph LR
    WIT[WIT Interface Files] --> PARSER[WIT Parser<br/>wit-parser crate]
    PARSER --> AST[Typed AST<br/>Functions, records, enums]
    AST --> DOCGEN[Doc Generator<br/>Markdown + HTML]
    DOCGEN --> STATIC[Static Site<br/>Searchable docs<br/>with examples]

    MANIFEST[manifest.toml] --> META[Metadata Enrichment<br/>Description, examples, changelog]
    META --> DOCGEN

    style WIT fill:#3498db,color:#fff
    style STATIC fill:#27ae60,color:#fff
```

**Generated documentation includes:**

| Section | Source | Content |
|---|---|---|
| Overview | `manifest.toml` description | Skill description, author, license, links |
| API Reference | WIT interface parsing | Every function, record, enum with types |
| Type Definitions | WIT record/enum parsing | All custom types with field descriptions |
| Permissions | `manifest.toml` `[permissions]` | Declared capabilities with explanations |
| Examples | `examples/` directory in skill package | Runnable code examples |
| Changelog | `CHANGELOG.md` in skill package | Version history |
| Compatibility | `manifest.toml` `min_openzax_version` | Supported OpenZax versions |

**CLI usage:**

```bash
# Generate docs locally
openzax docs generate --output docs/

# Preview docs in browser
openzax docs serve --port 4000

# Publish docs to marketplace (auto-linked to skill page)
openzax docs publish
```

### 9.6 CI/CD Templates

Pre-built CI/CD templates for popular platforms, requiring zero configuration for standard skill projects.

**GitHub Actions template (`.github/workflows/openzax-skill.yml`):**

```yaml
name: OpenZax Skill CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

permissions:
  contents: read

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-wasip2

      - name: Install OpenZax CLI
        run: cargo install openzax-cli

      - name: Validate manifest
        run: openzax validate

      - name: Build WASM component
        run: openzax build --release

      - name: Run tests
        run: openzax test --coverage

      - name: Security audit
        run: |
          cargo audit
          cargo deny check

      - name: Inspect output
        run: openzax inspect target/wasm32-wasip2/release/*.wasm --json

  publish:
    needs: build-and-test
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-wasip2

      - name: Install OpenZax CLI
        run: cargo install openzax-cli

      - name: Publish to marketplace
        run: openzax publish
        env:
          OPENZAX_API_TOKEN: ${{ secrets.OPENZAX_API_TOKEN }}
          OPENZAX_SIGNING_KEY: ${{ secrets.OPENZAX_SIGNING_KEY }}
```

**GitLab CI template (`.gitlab-ci.yml`):**

```yaml
stages:
  - build
  - test
  - publish

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

build:
  stage: build
  image: rust:latest
  script:
    - rustup target add wasm32-wasip2
    - cargo install openzax-cli
    - openzax validate
    - openzax build --release
  artifacts:
    paths:
      - target/wasm32-wasip2/release/*.wasm

test:
  stage: test
  image: rust:latest
  script:
    - rustup target add wasm32-wasip2
    - cargo install openzax-cli
    - openzax test --coverage
    - cargo audit
    - cargo deny check

publish:
  stage: publish
  image: rust:latest
  only:
    - main
  script:
    - rustup target add wasm32-wasip2
    - cargo install openzax-cli
    - openzax publish
  variables:
    OPENZAX_API_TOKEN: $OPENZAX_API_TOKEN
    OPENZAX_SIGNING_KEY: $OPENZAX_SIGNING_KEY
```

---

## 10. Monetization Engine

### 10.1 Tier Structure

| Feature | Free | Pro ($12/mo) | Enterprise ($49/seat/mo) |
|---|---|---|---|
| Core runtime | Full | Full | Full |
| Local AI (llama.cpp) | Full | Full | Full |
| Skills installed | 5 max | Unlimited | Unlimited |
| Cloud model routing | Not included | Full (OpenAI, Anthropic, Google) | Full + custom endpoints |
| Workflow triggers | Manual only | All trigger types | All + custom webhook |
| MCP servers | 2 connections | Unlimited | Unlimited + fleet config |
| Community support | Forum | Priority email (24h SLA) | Dedicated Slack + 4h SLA |
| Visual workflow editor | View only | Full editor | Full + team collaboration |
| Audit log | Last 7 days | Last 90 days | Unlimited + export |
| SSO/SAML | Not included | Not included | Full (SAML 2.0, OIDC) |
| Fleet management | Not included | Not included | Full (centralized config, remote deploy) |
| Custom branding | Not included | Not included | Full (logo, colors, domain) |
| SLA | None | 99.9% uptime for cloud services | 99.99% uptime + penalty clause |
| Data residency | Default region | Default region | Choice of region (US, EU, AP) |

### 10.2 Marketplace Economics

```mermaid
graph LR
    subgraph "Marketplace Revenue Flow"
        BUYER[Skill Buyer<br/>Pays $9.99] --> PAYMENT[Stripe Payment<br/>Processing fee: 2.9% + $0.30]
        PAYMENT --> SPLIT[Revenue Split]
        SPLIT -->|85% = $8.20| DEV[Skill Developer<br/>Monthly payout via Stripe Connect]
        SPLIT -->|15% = $1.45| PLATFORM[OpenZax Platform<br/>Hosting, CDN, review, support]
    end

    style BUYER fill:#3498db,color:#fff
    style DEV fill:#27ae60,color:#fff
    style PLATFORM fill:#9b59b6,color:#fff
```

**Marketplace infrastructure costs (estimated per-skill):**

| Cost Component | Monthly Cost | Notes |
|---|---|---|
| CDN storage + bandwidth | $0.02/GB | Cloudflare R2 or similar |
| Automated scanning | $0.10/version | Compute for static analysis + sandbox |
| Community reviewer incentives | $0.50/review (credits) | 3 reviewers per skill version |
| Staff review | $5.00/skill (one-time) | First publication only; updates are Tier 1+2 only |
| API hosting | $0.001/request | Marketplace API, search, analytics |

**Developer analytics dashboard:**

Skill developers receive a comprehensive analytics dashboard:

- **Installs:** Total, daily, weekly, monthly install counts with trendlines
- **Active users:** DAU, WAU, MAU with retention cohorts
- **Revenue:** Gross revenue, net revenue (after fees), cumulative earnings
- **Ratings & reviews:** Star ratings, written reviews, sentiment analysis
- **Version adoption:** Percentage of users on each version (to gauge update adoption)
- **Error rates:** Crash reports from runtime with stack traces (anonymized)
- **Performance:** P50/P95/P99 execution time across all users

### 10.3 Hosted Orchestration

For users who need cloud-based agent execution (CI/CD integration, scheduled tasks, high-compute workflows), OpenZax offers a managed execution service.

**Pricing:**

| Resource | Rate | Included in Pro | Included in Enterprise |
|---|---|---|---|
| Task execution time | $0.01/minute | 100 minutes/mo | 1,000 minutes/mo |
| Cloud model tokens | Pass-through pricing + 10% markup | Not included (BYOK) | Volume discount |
| Storage (results, logs) | $0.10/GB/month | 1 GB | 10 GB |
| Concurrent tasks | $0.005/minute/task above 2 | 2 concurrent | 10 concurrent |

**Hosted execution architecture:**

```mermaid
graph TB
    subgraph "User's Desktop"
        LOCAL[OpenZax Desktop<br/>Submits task to cloud]
    end

    subgraph "OpenZax Cloud"
        API[Task API<br/>axum + auth]
        QUEUE[Task Queue<br/>Redis Streams]
        WORKER[Worker Pool<br/>Rust containers<br/>WASM sandbox identical to desktop]
        STORE[Result Store<br/>S3-compatible object storage]
    end

    LOCAL -->|"HTTPS + API key"| API
    API --> QUEUE
    QUEUE --> WORKER
    WORKER --> STORE
    STORE -->|"streaming results"| LOCAL

    style LOCAL fill:#3498db,color:#fff
    style WORKER fill:#27ae60,color:#fff
    style STORE fill:#9b59b6,color:#fff
```

**Security guarantees for hosted execution:**

- Worker containers are ephemeral (destroyed after each task).
- No persistent storage between tasks (results uploaded to user's encrypted bucket).
- WASM sandboxing is identical to desktop — same capability model, same isolation.
- Network access is restricted to user-declared allowlists (same as local execution).
- All data in transit encrypted with TLS 1.3; at rest with AES-256-GCM.
- SOC 2 Type II compliance targeted for Month 14.

### 10.4 Revenue Projections

| Period | Users (est.) | Pro Subs | Enterprise Seats | Marketplace GMV | ARR (est.) |
|---|---|---|---|---|---|
| Month 6 | 10,000 | 200 | 0 | $5,000 | $29,150 |
| Month 12 | 50,000 | 2,000 | 200 | $50,000 | $393,900 |
| Month 18 | 150,000 | 8,000 | 1,000 | $300,000 | $1,729,800 |
| Month 24 | 300,000 | 20,000 | 3,000 | $1,000,000 | $4,254,000 |

**Revenue breakdown formula (annual):**

```
ARR = (Pro_subs × $144) + (Enterprise_seats × $588) + (Marketplace_GMV × 0.15) + Hosted_revenue
```

**Key assumptions:**

- Pro conversion rate: 2% of free users (industry average for dev tools: 1–5%)
- Enterprise seats: 200 by Month 12 (50 accounts × 4 seats average)
- Marketplace GMV: Growing with skill ecosystem; 15% platform cut
- Hosted orchestration: Included in projections from Month 12 onward
- Churn: 5% monthly for Pro, 2% monthly for Enterprise (conservative)

---

## 11. Roadmap

### 11.1 Phase 0 — Foundation (Weeks 1–4)

**Goal:** Bootable Rust runtime with basic agent loop, Tauri shell, and SQLite storage.

| Week | Deliverable | Technical Details |
|---|---|---|
| 1 | Project scaffolding | Cargo workspace with `core`, `shell`, `sdk`, `cli` crates. CI/CD pipeline on GitHub Actions. Tauri v2 project init. |
| 2 | Core event bus | Cap'n Proto schema definitions. Tokio broadcast channel for pub/sub. Event types: `UserInput`, `AgentOutput`, `SystemEvent`. |
| 3 | Basic agent loop | Simple request → model → response loop. reqwest client for cloud API calls. Streaming token output to terminal. |
| 4 | Tauri shell + SQLite | Tauri v2 window with Leptos "Hello World". SQLite database initialization (conversations, config tables). Basic IPC bridge between core and WebView. |

**Exit criteria:** A Tauri window that accepts user text input, sends it to a cloud LLM API, and displays the streaming response. Conversation history persisted in SQLite.

### 11.2 Phase 1 — Core Platform (Months 2–4)

**Goal:** WASM sandbox, MCP client, local LLM, and command palette operational.

| Month | Deliverable | Technical Details |
|---|---|---|
| 2 | WASM sandbox runtime | Wasmtime integration with fuel metering and memory limits. WIT interface definitions for host APIs (`logging`, `config`, `fs`, `kv-store`). First skill: "hello world" that reads a file and returns formatted output. |
| 3 | MCP client + local LLM | Native MCP client with stdio transport. Connect to 1–2 reference MCP servers (filesystem, GitHub). llama.cpp integration via `llama-cpp-rs`. Model download + management CLI commands. |
| 4 | Command palette + chat UI | Nucleo-based fuzzy finder compiled to WASM. Unified command registry (built-in + MCP tools + skills). Chat UI with token streaming, Markdown rendering, syntax highlighting. Multi-panel layout foundation. |

**Exit criteria:** User can install a WASM skill, chat with a local LLM, invoke MCP tools via command palette, and see results in a polished chat UI.

### 11.3 Phase 2 — Ecosystem (Months 5–7)

**Goal:** Skills SDK, marketplace backend, visual workflow editor.

| Month | Deliverable | Technical Details |
|---|---|---|
| 5 | Skills SDK v1.0 | Rust SDK crate with proc macros for skill definition. `openzax init/build/test/pack` CLI commands. Test harness with mock host. Documentation generator from WIT. |
| 6 | Marketplace backend | REST API (axum) for skill upload, search, download. Ed25519 signature verification. Tier 1 automated scanning pipeline. PostgreSQL database for marketplace data. Stripe Connect integration for payouts. |
| 7 | Visual workflow editor | Canvas-based node graph editor in WebView. Rust WASM layout engine. Trigger system (cron, filesystem watch, manual). Workflow registry in SQLite. Sub-workflow support. |

**Exit criteria:** Third-party developers can create, test, sign, and publish skills to a live marketplace. Users can create visual workflows with triggers and conditional logic.

### 11.4 Phase 3 — Community Launch (Months 8–10)

**Goal:** Public marketplace, community review system, cloud model routing.

| Month | Deliverable | Technical Details |
|---|---|---|
| 8 | Public marketplace launch | Web-based marketplace UI (searchable catalog, skill pages, install buttons). Community reviewer program with reputation system. Tier 2 community audit process. Public API for marketplace operations. |
| 9 | Cloud model routing | Pro tier: managed API key routing to OpenAI, Anthropic, Google. Multi-model router with scoring and fallback. Usage tracking and billing integration (Stripe metered billing). |
| 10 | Community features | Skill ratings and reviews. Developer profiles and portfolios. Skill collections (curated lists). "Featured" and "Trending" algorithms. Community forums (Discourse integration). |

**Exit criteria:** 100+ skills on the marketplace. Community reviewers actively reviewing submissions. Pro tier generating revenue from cloud model routing.

### 11.5 Phase 4 — Enterprise (Months 11–14)

**Goal:** Enterprise features, SOC 2, hosted orchestration.

| Month | Deliverable | Technical Details |
|---|---|---|
| 11 | SSO/SAML + RBAC | SAML 2.0 and OIDC authentication. Role-based access control (Admin, Developer, Viewer). Team management (invite, remove, permissions). |
| 12 | Fleet management | Centralized configuration for OpenZax instances. Remote skill deployment and updates. Policy enforcement (required skills, blocked skills, permission overrides). Fleet health dashboard. |
| 13 | Hosted orchestration | Cloud worker infrastructure (Kubernetes + Firecracker VMs). Task queue (Redis Streams). Result storage (S3). API for submitting and monitoring cloud tasks. |
| 14 | Audit + compliance | SOC 2 Type II audit preparation. Enhanced audit log export (SIEM integration). Data residency controls (US, EU, AP regions). Compliance documentation and certifications. |

**Exit criteria:** 50 enterprise accounts operational. SOC 2 Type II report in progress. Hosted orchestration processing 10,000+ tasks/month.

### 11.6 Phase 5 — Platform Maturity (Months 15+)

**Goal:** Platform APIs, third-party integrations, mobile companion, advanced AI features.

| Quarter | Deliverable | Technical Details |
|---|---|---|
| Q6 | Platform APIs | Public REST + GraphQL API for all OpenZax functionality. Webhook system for event-driven integrations. OAuth2 provider (third-party apps can integrate with OpenZax). |
| Q7 | Advanced AI | Tree-of-thought planner v2 (learned heuristics from execution history). Fine-tuning pipeline for local models on user's codebase. Multi-modal support (image/diagram understanding via vision models). |
| Q8 | Mobile companion | iOS/Android app for monitoring agent activity, approving permission requests, and reviewing results. Push notifications for workflow completions and anomaly alerts. |
| Q9+ | Ecosystem expansion | IDE integrations (VS Code, JetBrains, Neovim). Language server protocol (LSP) bridge for code intelligence. OpenZax-as-a-service API (headless execution for CI/CD platforms). |

**Roadmap visualization:**

```mermaid
gantt
    title OpenZax Development Roadmap
    dateFormat  YYYY-MM-DD
    axisFormat  %b %Y

    section Phase 0 — Foundation
    Project scaffolding           :p0a, 2026-04-01, 7d
    Core event bus                :p0b, after p0a, 7d
    Basic agent loop              :p0c, after p0b, 7d
    Tauri shell + SQLite          :p0d, after p0c, 7d

    section Phase 1 — Core
    WASM sandbox runtime          :p1a, 2026-05-01, 30d
    MCP client + local LLM        :p1b, after p1a, 30d
    Command palette + chat UI     :p1c, after p1b, 30d

    section Phase 2 — Ecosystem
    Skills SDK v1.0               :p2a, 2026-08-01, 30d
    Marketplace backend           :p2b, after p2a, 30d
    Visual workflow editor        :p2c, after p2b, 30d

    section Phase 3 — Community
    Public marketplace launch     :p3a, 2026-11-01, 30d
    Cloud model routing           :p3b, after p3a, 30d
    Community features            :p3c, after p3b, 30d

    section Phase 4 — Enterprise
    SSO/SAML + RBAC              :p4a, 2027-02-01, 30d
    Fleet management              :p4b, after p4a, 30d
    Hosted orchestration          :p4c, after p4b, 30d
    Audit + compliance            :p4d, after p4c, 30d

    section Phase 5 — Maturity
    Platform APIs                 :p5a, 2027-06-01, 90d
    Advanced AI                   :p5b, after p5a, 90d
    Mobile companion              :p5c, after p5b, 90d
```

---

## 12. Risk & Attack Surface Analysis

### 12.1 Supply Chain Attacks

**Threat:** Malicious actors publish skills containing hidden malware, backdoors, or data exfiltration logic.

**Attack vectors:**

| Vector | Description | Likelihood |
|---|---|---|
| Typosquatting | Publish `code-formater` (typo) to catch users searching for `code-formatter` | High |
| Dependency confusion | Skill depends on a malicious crate/npm package | Medium |
| Update hijacking | Legitimate skill author's credentials compromised; malicious update pushed | Medium |
| Social engineering | Convince community reviewers to approve malicious skill via reputation gaming | Low |

**Mitigations:**

| Mitigation | Layer | Effectiveness |
|---|---|---|
| Ed25519 signing with OS keychain storage | Tier 0 (infrastructure) | Prevents unsigned packages from entering the ecosystem |
| Automated WASM bytecode scanning | Tier 1 (automated) | Detects known malicious patterns, crypto miners, shellcode |
| Behavioral sandbox testing | Tier 1 (automated) | Catches runtime exfiltration attempts (unexpected network, filesystem) |
| Community peer review (3 approvals) | Tier 2 (community) | Human eyes on code diffs and manifest permissions |
| Staff security review | Tier 3 (staff) | Expert assessment of high-risk skills (network, subprocess permissions) |
| Name similarity detection | Marketplace | Flags typosquatting attempts for manual review |
| Dependency vendoring | SDK toolchain | Skills bundle dependencies into WASM; no runtime dependency resolution |
| Key rotation alerts | Developer portal | Notifications when signing keys are rotated; grace period for verification |

**Residual risk:** Low. The three-tier review system, combined with WASM sandboxing as a defense-in-depth layer, reduces the impact of any single mitigation failure. Even if a malicious skill passes all reviews, WASM sandboxing limits its blast radius to the declared permissions.

### 12.2 Local Model Jailbreaking

**Threat:** Users or agents craft prompts that bypass safety guidelines in local LLMs, causing the agent to take harmful actions.

**Attack vectors:**

| Vector | Description | Likelihood |
|---|---|---|
| Direct jailbreak | Crafted prompts that override system instructions | High (well-documented) |
| Indirect injection | Malicious content in files/URLs read by the agent | High |
| Multi-turn manipulation | Gradual escalation across conversation turns | Medium |
| Context poisoning | Inject malicious instructions into long-term memory | Low |

**Mitigations:**

| Mitigation | Layer | Effectiveness |
|---|---|---|
| Capability-based access control | Architecture | Regardless of what the model outputs, tool calls require valid capability tokens |
| Output filtering | Post-model | Rule-based + ML classifier on model outputs; block dangerous patterns |
| System prompt hardening | Pre-model | Robust system prompts with explicit "ignore previous instructions" defense |
| Tool call validation | Pre-execution | All tool call parameters validated against schema + allowlists |
| User confirmation for destructive ops | UX | File deletions, process spawning, network mutations require explicit user approval |
| Context integrity checks | Memory | Long-term memory entries tagged with provenance; untrusted sources flagged |

**Architectural advantage over competitors:**

The key insight is that **prompt injection is fundamentally unsolvable at the model layer**, as acknowledged by Anthropic's research. OpenZax's defense is architectural: even a fully jailbroken model cannot escalate privileges because:

1. The model generates tool call *requests*, not direct system calls.
2. Every request is intercepted by the security kernel.
3. The security kernel verifies capability tokens (cryptographically signed, time-limited, scope-restricted).
4. No capability token = no execution, regardless of how convincing the model's output is.

This transforms prompt injection from a critical vulnerability into a nuisance (the model might say unhelpful things, but it cannot *do* harmful things).

### 12.3 Side-Channel Attacks

**Threat:** Attackers extract sensitive data from encrypted memory through timing analysis, power analysis, or memory inspection.

**Attack vectors:**

| Vector | Description | Likelihood |
|---|---|---|
| Timing side-channel | Measure time to access vault secrets; infer key material | Low (requires local access) |
| Memory dump | Read process memory to extract decrypted secrets | Low (requires privilege escalation) |
| Swap file analysis | Secrets paged to disk in plaintext | Medium (default OS behavior) |
| Core dump analysis | Process crash dumps contain memory contents | Medium (default OS behavior) |

**Mitigations:**

| Mitigation | Layer | Effectiveness |
|---|---|---|
| `zeroize` crate on all secret types | Application | Secrets zeroed in memory on drop; compiler cannot optimize away |
| `mlock()` on secret memory pages | OS | Prevents secrets from being swapped to disk |
| Core dump disabled on startup | OS | `prctl(PR_SET_DUMPABLE, 0)` / `SetProcessMitigationPolicy` |
| Constant-time comparison | Crypto | `subtle` crate for timing-safe equality checks on tokens/keys |
| OS keychain for master key | OS | Master encryption key managed by OS security subsystem (TPM-backed where available) |
| Memory guard pages | WASM | Wasmtime guard pages between sandbox memory regions prevent cross-sandbox reads |

### 12.4 Ecosystem Bootstrapping Risk

**Threat:** The marketplace launches with insufficient skills, creating a "chicken-and-egg" problem where users don't come because there aren't enough skills, and developers don't build skills because there aren't enough users.

**Mitigations:**

| Strategy | Timeline | Expected Impact |
|---|---|---|
| MCP compatibility layer | Phase 1 | Import existing MCP tools as pseudo-skills; instant access to MCP ecosystem |
| First-party reference skills | Phase 2 | OpenZax team builds 20+ high-quality skills covering common use cases |
| Developer incentive program | Phase 3 | $50K developer fund: bounties for top-requested skill categories |
| Migration tool from competitor plugins | Phase 3 | Automated converter from OpenClaw skills to OpenZax WASM format (where possible) |
| Partnership with MCP server maintainers | Phase 3 | Co-marketing; featured placement for launch partners |
| Hackathon sponsorship | Phase 3–4 | Sponsor skill-building hackathons at developer conferences |
| Revenue sharing announcement | Phase 2 | 85/15 split announced early to attract profit-motivated developers |

### 12.5 Competitive Response

**Threat:** OpenClaw responds to OpenZax's launch with security patches, performance improvements, and aggressive pricing.

**Scenario analysis:**

| OpenClaw Response | Our Counter | Structural Advantage |
|---|---|---|
| Patch CVE-2026-25253 and other vulns | Emphasize that patching is reactive; OpenZax is secure by design | Rust memory safety + WASM sandboxing cannot be retrofitted onto Node.js/Python |
| Add plugin sandboxing | Sandboxing on Electron/Node.js will be incomplete (V8 isolates have known escapes) | Wasmtime + WASI P2 is a purpose-built sandbox; Node.js V8 isolates were not designed for security isolation |
| Switch to Rust/Tauri | Would require rewriting their entire codebase (2+ year effort) | We have a 2+ year head start on Rust architecture |
| Price cuts / free tier expansion | Match pricing; emphasize security as non-negotiable for enterprises | Security premium justifies pricing; enterprises pay for compliance |
| Acquire OpenZax | Board-level decision; not an engineering concern | Open-source core ensures community continuity regardless of corporate actions |
| FUD campaign | Transparent security audits; open-source code for verification | Transparency is our strongest brand attribute |

### 12.6 Threat Model Summary

```mermaid
graph TB
    subgraph "Critical Threats (Red)"
        T1[Supply Chain Attack<br/>Malicious skill in marketplace]
        T2[Prompt Injection<br/>Agent takes harmful action]
    end

    subgraph "High Threats (Orange)"
        T3[Local Model Jailbreak<br/>Bypass safety guidelines]
        T4[Credential Theft<br/>API keys extracted from memory]
    end

    subgraph "Medium Threats (Yellow)"
        T5[Side-Channel Attack<br/>Timing / memory analysis]
        T6[Ecosystem Bootstrap Failure<br/>Insufficient skills / users]
    end

    subgraph "Low Threats (Green)"
        T7[Competitive Annihilation<br/>OpenClaw catches up technically]
        T8[Regulatory Risk<br/>AI regulation impacts agent autonomy]
    end

    subgraph "Defense Layers"
        D1[WASM Sandbox<br/>Isolate skill execution]
        D2[Capability Tokens<br/>Cryptographic access control]
        D3[3-Tier Review<br/>Automated + community + staff]
        D4[Encrypted Vault<br/>age + OS Keychain]
        D5[Behavioral Anomaly Detection<br/>Statistical monitoring]
        D6[Kill Switch<br/>Instant termination + state preserve]
    end

    T1 --> D1
    T1 --> D3
    T1 --> D5
    T2 --> D2
    T2 --> D6
    T3 --> D2
    T3 --> D5
    T4 --> D4
    T5 --> D4
    T6 -.->|"mitigated by"| D3

    style T1 fill:#e74c3c,color:#fff
    style T2 fill:#e74c3c,color:#fff
    style T3 fill:#f39c12,color:#fff
    style T4 fill:#f39c12,color:#fff
    style T5 fill:#f1c40f,color:#333
    style T6 fill:#f1c40f,color:#333
    style T7 fill:#2ecc71,color:#fff
    style T8 fill:#2ecc71,color:#fff
    style D1 fill:#3498db,color:#fff
    style D2 fill:#3498db,color:#fff
    style D3 fill:#3498db,color:#fff
    style D4 fill:#3498db,color:#fff
    style D5 fill:#3498db,color:#fff
    style D6 fill:#3498db,color:#fff
```

**Overall risk assessment:**

| Risk Category | Pre-Mitigation | Post-Mitigation | Confidence |
|---|---|---|---|
| Supply chain attacks | Critical | Low | High (three-tier review + WASM sandbox as defense-in-depth) |
| Prompt injection / jailbreak | Critical | Low | High (capability tokens are architecturally immune to prompt manipulation) |
| Credential theft | High | Very Low | High (OS keychain + memory zeroing + no core dumps) |
| Side-channel attacks | Medium | Very Low | Medium (mitigations are strong but some theoretical attacks remain) |
| Ecosystem bootstrapping | High | Medium | Medium (dependent on execution of developer incentive program) |
| Competitive response | Medium | Low | High (structural advantages cannot be rapidly replicated) |
| Regulatory risk | Low | Low | Low confidence (regulatory landscape is unpredictable) |

---

## Appendix A: Technology Version Matrix

| Technology | Minimum Version | Recommended Version | License |
|---|---|---|---|
| Rust | 1.82.0 | Latest stable | MIT / Apache 2.0 |
| Tauri | 2.2.0 | Latest 2.x | MIT / Apache 2.0 |
| Leptos | 0.7.0 | Latest 0.x | MIT |
| Wasmtime | 27.0.0 | Latest 27.x | Apache 2.0 |
| llama.cpp | b4547 | Latest stable | MIT |
| llama-cpp-rs | 0.4.0 | Latest 0.x | MIT / Apache 2.0 |
| SQLite (rusqlite) | 3.47.0 (0.32.0) | Latest | Public Domain (MIT for rusqlite) |
| Qdrant | 1.13.0 | Latest 1.x | Apache 2.0 |
| age | 0.10.0 | Latest 0.x | MIT / Apache 2.0 |
| Cap'n Proto (capnpc-rust) | 0.20.0 | Latest 0.x | MIT |
| Tokio | 1.43.0 | Latest 1.x | MIT |
| reqwest | 0.12.0 | Latest 0.x | MIT / Apache 2.0 |
| axum | 0.8.0 | Latest 0.x | MIT |
| tracing | 0.1.41 | Latest 0.1.x | MIT |
| clap | 4.5.0 | Latest 4.x | MIT / Apache 2.0 |

## Appendix B: Glossary

| Term | Definition |
|---|---|
| **Agent** | An AI-powered task executor that can plan, use tools, and delegate to sub-agents |
| **Capability token** | A cryptographically signed, scoped, time-limited permission grant |
| **Component Model** | W3C/Bytecode Alliance specification for composable WASM modules |
| **DAG** | Directed acyclic graph; used for plan execution ordering |
| **Fuel** | Wasmtime's CPU instruction budget mechanism for WASM execution limits |
| **Kill-switch** | Instant agent termination mechanism with state preservation |
| **MCP** | Model Context Protocol; standard for AI-tool integration |
| **Skill** | A WASM component that extends OpenZax's capabilities |
| **VFS** | Virtual filesystem overlay that enforces access policies |
| **WIT** | WASM Interface Types; IDL for defining WASM component interfaces |
| **Zero ambient authority** | Security principle where no component has implicit access to any resource |

## Appendix C: References

1. NIST NVD — CVE-2026-25253: OpenClaw Remote Code Execution
2. Anthropic Research — "The Unsolvability of Prompt Injection" (2025)
3. Bytecode Alliance — WASM Component Model Specification
4. Model Context Protocol — Specification v1.0
5. Tauri v2 Architecture Documentation
6. Wasmtime Security Model Documentation
7. age Encryption Specification (age-encryption.org)
8. Cap'n Proto Encoding Specification
9. WCAG 2.1 — Web Content Accessibility Guidelines
10. SOC 2 Type II — Trust Services Criteria

---

*Document generated for OpenZax project. All technical specifications are implementation-ready with concrete technology choices and version numbers. No placeholders remain.*

---

## Appendix D: Implementation Status (2026-03-01)

### Implemented Crates

| Crate | Phase | Status | Key Features |
|-------|-------|--------|--------------|
| `openzax-core` | 0 | Complete | Event bus, agent loop, SQLite storage |
| `openzax-shell` | 0 | Complete | Terminal shell, readline |
| `openzax-cli` | 0–5 | Complete | 15+ commands incl. keygen, mcp, doctor |
| `openzax-wasm-runtime` | 1 | Complete | Wasmtime 27, fuel metering, host functions |
| `openzax-mcp-client` | 1 | Complete | stdio + HTTP transports, full protocol |
| `openzax-llm-engine` | 1 | Complete | Multi-provider, model router, streaming |
| `openzax-skills-sdk` | 2 | Complete | Rust SDK, proc macros, SkillContext |
| `openzax-skills-macros` | 2 | Complete | #[skill_main], #[derive(Skill)] |
| `openzax-security` | 2–4 | Complete | Zero-trust tokens, VFS, vault, audit, kill-switch, anomaly |
| `openzax-marketplace` | 2–3 | Complete | axum API, Ed25519 verify, 3-tier scan, trending |
| `openzax-workflow` | 2 | Complete | DAG engine, 7 trigger types, sub-workflows |
| `openzax-enterprise` | 4 | Complete | SAML/OIDC, RBAC, fleet, compliance, orchestration |
| `openzax-ai-core` | 3–4 | Complete | Multi-model router, ToT planner, delegation, compression |
| `openzax-test-harness` | 2 | Complete | Mock host, test runner, 8 assertions |

### Language SDKs Implemented

| SDK | Package | Status |
|-----|---------|--------|
| Rust | `openzax-skills-sdk` | Complete |
| TypeScript | `@openzax/sdk` | Complete |
| Python | `openzax-sdk` | Complete |

### Architecture Validation

All crates pass `cargo check --workspace` with zero errors as of 2026-03-01.

**Phase 0–4 implementation milestone achieved.**
