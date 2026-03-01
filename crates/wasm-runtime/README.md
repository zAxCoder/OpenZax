# OpenZax WASM Runtime

Secure WebAssembly sandbox runtime for OpenZax skills with capability-based security.

## Features

- **Fuel Metering**: CPU instruction budgets to prevent infinite loops
- **Memory Limits**: Configurable memory caps per skill instance
- **WASI Preview 2**: Modern WASI support for filesystem and networking
- **Host Functions**: Rich set of host APIs for skills
- **Isolation**: Each skill runs in its own sandboxed environment

## Architecture

```
┌─────────────────────────────────────────┐
│         OpenZax Core Engine             │
├─────────────────────────────────────────┤
│         WASM Runtime (Wasmtime)         │
├─────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │ Skill 1  │  │ Skill 2  │  │Skill N ││
│  │ Sandbox  │  │ Sandbox  │  │Sandbox ││
│  └──────────┘  └──────────┘  └────────┘│
└─────────────────────────────────────────┘
```

## Usage

```rust
use openzax_wasm_runtime::{Sandbox, SandboxConfig};

// Create sandbox with default config
let config = SandboxConfig::default();
let sandbox = Sandbox::new(config)?;

// Load WASM module
let module = sandbox.load_module("skill.wasm")?;

// Create instance
let mut instance = sandbox.create_instance(&module)?;

// Call exported function
let results = instance.call_function("hello", &[])?;

// Check resource usage
let fuel_remaining = instance.get_remaining_fuel()?;
let memory_used = instance.get_memory_usage()?;
```

## Configuration

```rust
let config = SandboxConfig {
    max_memory_bytes: 16 * 1024 * 1024, // 16 MB
    max_fuel: 2_000_000_000,             // 2 billion instructions
    wasi_preview2: true,
    fs_read_paths: vec!["src/**/*.rs".to_string()],
    fs_write_paths: vec!["output/**".to_string()],
    network_allow: vec!["api.github.com".to_string()],
    allow_subprocess: false,
};
```

## Host Interfaces

Skills can import these host interfaces:

- **openzax:host/logging** - Structured logging
- **openzax:host/config** - Key-value configuration
- **openzax:host/fs** - Virtual filesystem access
- **openzax:host/kv-store** - Persistent storage
- **openzax:host/http-client** - HTTP requests
- **openzax:host/events** - Pub/sub events

See `wit/` directory for complete interface definitions.

## Security

- **Zero Ambient Authority**: Skills have no implicit permissions
- **Capability Tokens**: All operations require explicit grants
- **Virtual Filesystem**: Skills cannot access host filesystem directly
- **Network Allowlist**: Only declared hosts are accessible
- **Resource Limits**: CPU and memory budgets enforced at runtime

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_test

# Build example skill
cd examples/hello-skill
./build.sh
```

## Performance

Typical overhead:
- Module load: ~5-10ms
- Instance creation: ~1-2ms
- Function call: ~1-5μs
- Memory access: Near-native speed

## Roadmap

- [x] Basic Wasmtime integration
- [x] Fuel metering
- [x] Memory limits
- [x] Host function framework
- [ ] Complete WIT interface implementations
- [ ] Virtual filesystem overlay
- [ ] Network request filtering
- [ ] Component Model support
- [ ] Multi-threading support
