# OpenZax WASM Runtime Guide

## Overview

The OpenZax WASM Runtime provides a secure, isolated execution environment for skills (plugins/extensions). Built on Wasmtime, it enforces strict resource limits and capability-based security.

## Key Concepts

### Sandbox

A sandbox is an isolated execution environment with:
- CPU instruction budget (fuel)
- Memory limits
- Filesystem access restrictions
- Network access allowlist
- No subprocess spawning by default

### Host Functions

Host functions are Rust functions exposed to WASM skills. They provide controlled access to system resources:

```rust
// Example: Logging host function
linker.func_wrap(
    "openzax:host/logging",
    "log",
    |caller: Caller<'_, HostContext>, level: i32, ptr: i32, len: i32| {
        // Read message from WASM memory
        // Log using tracing
        Ok(())
    },
)?;
```

### Capability Tokens

Every privileged operation requires a capability token:
- Cryptographically signed
- Time-limited (max 1 hour)
- Scope-restricted (specific paths, hosts, operations)
- Revocable

## Creating a Skill

### 1. Project Setup

```bash
cargo new --lib my-skill
cd my-skill
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-skill"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "z"  # Optimize for size
lto = true       # Link-time optimization
strip = true     # Strip debug symbols
```

### 3. Implement Skill

```rust
// src/lib.rs

#[no_mangle]
pub extern "C" fn process(input_ptr: i32, input_len: i32) -> i32 {
    // Read input from WASM memory
    let input = unsafe {
        std::slice::from_raw_parts(input_ptr as *const u8, input_len as usize)
    };
    
    // Process data
    let result = do_work(input);
    
    // Return result pointer
    result.as_ptr() as i32
}

fn do_work(input: &[u8]) -> Vec<u8> {
    // Your skill logic here
    input.to_vec()
}
```

### 4. Build

```bash
cargo build --target wasm32-wasi --release
```

Output: `target/wasm32-wasi/release/my_skill.wasm`

## Using Host Functions

### Logging

```rust
#[link(wasm_import_module = "openzax:host/logging")]
extern "C" {
    fn log(level: i32, ptr: i32, len: i32);
}

fn log_info(message: &str) {
    unsafe {
        log(2, message.as_ptr() as i32, message.len() as i32);
    }
}
```

### Filesystem

```rust
#[link(wasm_import_module = "openzax:host/fs")]
extern "C" {
    fn read(path_ptr: i32, path_len: i32, out_ptr: i32, out_len: i32) -> i32;
}

fn read_file(path: &str) -> Result<Vec<u8>, String> {
    // Implementation using host function
    todo!()
}
```

### HTTP Client

```rust
#[link(wasm_import_module = "openzax:host/http-client")]
extern "C" {
    fn fetch(req_ptr: i32, req_len: i32, resp_ptr: i32, resp_len: i32) -> i32;
}

fn http_get(url: &str) -> Result<Vec<u8>, String> {
    // Implementation using host function
    todo!()
}
```

## Resource Limits

### CPU (Fuel)

```rust
let config = SandboxConfig {
    max_fuel: 1_000_000_000, // 1 billion instructions
    ..Default::default()
};
```

Typical fuel consumption:
- Simple arithmetic: 1-10 fuel
- Memory access: 1 fuel
- Function call: 10-100 fuel
- Host function call: 100-1000 fuel

### Memory

```rust
let config = SandboxConfig {
    max_memory_bytes: 8 * 1024 * 1024, // 8 MB
    ..Default::default()
};
```

Memory is allocated in 64KB pages. Skills start with 1 page and can grow up to the limit.

### Filesystem

```rust
let config = SandboxConfig {
    fs_read_paths: vec!["src/**/*.rs".to_string()],
    fs_write_paths: vec!["output/**".to_string()],
    ..Default::default()
};
```

Paths use glob patterns. Skills can only access declared paths.

### Network

```rust
let config = SandboxConfig {
    network_allow: vec![
        "api.github.com".to_string(),
        "*.example.com".to_string(),
    ],
    ..Default::default()
};
```

Only declared hosts are accessible. Wildcards supported.

## Error Handling

```rust
match instance.call_function("process", &[Val::I32(42)]) {
    Ok(results) => println!("Success: {:?}", results),
    Err(WasmError::FuelExhausted(msg)) => {
        eprintln!("CPU budget exhausted: {}", msg);
    }
    Err(WasmError::MemoryLimitExceeded(msg)) => {
        eprintln!("Memory limit exceeded: {}", msg);
    }
    Err(WasmError::PermissionDenied(msg)) => {
        eprintln!("Permission denied: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Best Practices

### 1. Minimize Memory Allocations

```rust
// Bad: Allocates on every call
fn process(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    // ...
    result
}

// Good: Reuse buffer
static mut BUFFER: [u8; 4096] = [0; 4096];

fn process(input: &[u8]) -> &[u8] {
    unsafe {
        // Use BUFFER
        &BUFFER[..]
    }
}
```

### 2. Batch Operations

```rust
// Bad: Multiple host calls
for item in items {
    log_info(&format!("Processing {}", item));
}

// Good: Single host call
let message = items.iter()
    .map(|i| format!("Processing {}", i))
    .collect::<Vec<_>>()
    .join("\n");
log_info(&message);
```

### 3. Check Limits Proactively

```rust
fn process(input: &[u8]) -> Result<Vec<u8>, String> {
    // Check if we have enough fuel
    let fuel = get_remaining_fuel()?;
    if fuel < 1000 {
        return Err("Insufficient fuel".to_string());
    }
    
    // Process
    Ok(vec![])
}
```

## Debugging

### 1. Enable Logging

```rust
RUST_LOG=openzax_wasm_runtime=debug cargo test
```

### 2. Inspect WASM Module

```bash
wasm-objdump -x skill.wasm
wasm-objdump -d skill.wasm  # Disassemble
```

### 3. Profile Fuel Usage

```rust
let fuel_before = instance.get_remaining_fuel()?;
instance.call_function("process", &[])?;
let fuel_after = instance.get_remaining_fuel()?;
println!("Fuel consumed: {}", fuel_before - fuel_after);
```

## Security Considerations

1. **Never trust skill input**: Validate all data from skills
2. **Limit resource usage**: Set conservative fuel and memory limits
3. **Restrict filesystem access**: Use minimal glob patterns
4. **Allowlist network hosts**: Only permit necessary domains
5. **Disable subprocess spawning**: Unless absolutely required
6. **Audit host functions**: Review all exposed functionality
7. **Monitor anomalies**: Track unusual resource consumption patterns

## Performance Tips

1. **Optimize WASM size**: Use `opt-level = "z"` and `wasm-opt`
2. **Minimize host calls**: Batch operations when possible
3. **Reuse instances**: Instance creation has overhead
4. **Profile hot paths**: Use fuel consumption as a proxy
5. **Consider caching**: Cache compiled modules

## Troubleshooting

### "Fuel exhausted" error

Increase `max_fuel` or optimize skill code.

### "Memory limit exceeded" error

Increase `max_memory_bytes` or reduce allocations.

### "Permission denied" error

Check `fs_read_paths`, `fs_write_paths`, and `network_allow` configuration.

### "Function not found" error

Ensure function is exported with `#[no_mangle]` and `pub extern "C"`.

## Next Steps

- Read [WIT Interface Reference](../wit/)
- See [Example Skills](../../examples/)
- Review [Security Model](./security-model.md)
- Explore [Marketplace Guide](./marketplace-guide.md)
