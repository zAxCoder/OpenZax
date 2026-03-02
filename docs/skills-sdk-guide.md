# OpenZax Skills SDK Guide

## Overview

The OpenZax Skills SDK enables developers to create secure, sandboxed extensions (skills) that run in a WASM environment. Skills can interact with the host system through a well-defined capability-based API.

## Supported Languages

- **Rust** [done] (Fully supported)
- **TypeScript** [wip] (Coming in Phase 2)
- **Python** [wip] (Coming in Phase 2)

## Quick Start

### 1. Create a New Skill

```bash
openzax skill init my-skill --language rust
cd my-skill
```

This creates a new skill project with the following structure:

```
my-skill/
├── .cargo/
│   └── config.toml
├── src/
│   └── lib.rs
├── Cargo.toml
├── build.sh
└── README.md
```

### 2. Implement Your Skill

Edit `src/lib.rs`:

```rust
use openzax_skills_sdk::{skill_main, SkillContext, SkillResult};

#[skill_main]
fn run() -> SkillResult<()> {
    let ctx = SkillContext::new();
    
    // Log messages
    ctx.log_info("Starting my skill...");
    
    // Read configuration
    if let Some(api_key) = ctx.get_config("api_key") {
        ctx.log_info(&format!("Using API key: {}", api_key));
    }
    
    // Your skill logic here
    ctx.log_info("Skill completed successfully!");
    
    Ok(())
}
```

### 3. Build the Skill

```bash
openzax skill build --release
```

This compiles your skill to WASM targeting `wasm32-wasip1`.

### 4. Test the Skill

```bash
openzax skill test
```

## SDK API Reference

### SkillContext

The `SkillContext` provides access to host capabilities:

#### Configuration

```rust
// Get configuration value
let value = ctx.get_config("key");

// Set configuration value
ctx.set_config("key".to_string(), "value".to_string());
```

#### Logging

```rust
// Log info message
ctx.log_info("Information message");

// Log error message
ctx.log_error("Error message");
```

#### File System (requires `fs:read` or `fs:write` permission)

```rust
// Read file
let content = ctx.read_file("/path/to/file")?;

// Write file
ctx.write_file("/path/to/file", b"content")?;
```

#### HTTP (requires `net:http` permission)

```rust
// Make HTTP GET request
let response = ctx.http_get("https://api.example.com/data")?;
```

### Error Handling

```rust
use openzax_skills_sdk::{SkillError, SkillResult};

fn my_function() -> SkillResult<String> {
    // Return errors
    Err(SkillError::InvalidInput("Invalid parameter".to_string()))
}
```

Available error types:
- `SkillError::Config` - Configuration errors
- `SkillError::Io` - I/O errors
- `SkillError::Network` - Network errors
- `SkillError::Permission` - Permission denied
- `SkillError::InvalidInput` - Invalid input
- `SkillError::Internal` - Internal errors

### Macros

#### `#[skill_main]`

Marks a function as the skill entry point:

```rust
#[skill_main]
fn run() -> SkillResult<()> {
    // Your code here
    Ok(())
}
```

#### `#[derive(Skill)]`

Derives the `Skill` trait for a struct:

```rust
#[derive(Skill)]
struct MySkill {
    name: String,
}
```

## Skill Manifest

Create a `skill.toml` file to define skill metadata:

```toml
[skill]
name = "my-skill"
version = "0.1.0"
description = "A sample skill"
author = "Your Name"
license = "MIT"

[permissions]
required = [
    "fs:read",
    "net:http",
]

[dependencies]
other-skill = "1.0.0"
```

## Permissions

Skills must declare required permissions:

- `fs:read` - Read files
- `fs:write` - Write files
- `fs:execute` - Execute files
- `net:http` - Make HTTP requests
- `net:websocket` - Use WebSockets
- `tool:call` - Call MCP tools
- `agent:spawn` - Spawn sub-agents
- `env:read` - Read environment variables

## Building for Production

### Optimize WASM Size

The default `Cargo.toml` includes optimization settings:

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Enable link-time optimization
strip = true         # Strip debug symbols
```

### Additional Optimizations

```bash
# Install wasm-opt
cargo install wasm-opt

# Build and optimize
openzax skill build --release
wasm-opt -Oz -o optimized.wasm target/wasm32-wasip1/release/my_skill.wasm
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

Run tests:

```bash
openzax skill test
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use openzax_skills_sdk::*;

#[test]
fn test_skill_execution() {
    // Test your skill
}
```

## Packaging and Distribution

### Pack a Skill

```bash
openzax skill pack
```

This creates a `.ozskill` package containing:
- WASM binary
- Manifest
- Documentation
- Signature

### Sign a Skill

```bash
# Generate signing key
openzax keygen --output skill.key

# Sign package
openzax skill sign my-skill.ozskill --key skill.key
```

### Publish to Marketplace

```bash
openzax skill publish my-skill.ozskill
```

## Advanced Topics

### Using WIT Interfaces

Skills can use WIT (WebAssembly Interface Types) for type-safe host communication:

```rust
// Import WIT bindings
wit_bindgen::generate!({
    world: "skill",
    exports: {
        world: MySkill,
    },
});

struct MySkill;

impl Guest for MySkill {
    fn run() -> Result<(), String> {
        // Implementation
        Ok(())
    }
}
```

### Resource Limits

Skills run with enforced limits:
- **CPU**: Fuel metering (configurable)
- **Memory**: 64MB default (configurable)
- **Execution Time**: 30s default (configurable)

### Debugging

Enable debug logging:

```rust
ctx.log_debug("Debug information");
```

View logs in OpenZax:
```bash
openzax logs --skill my-skill
```

## Best Practices

1. **Keep skills focused** - One skill, one purpose
2. **Minimize dependencies** - Smaller WASM = faster loading
3. **Handle errors gracefully** - Always return meaningful errors
4. **Request minimal permissions** - Only what you need
5. **Write tests** - Test before publishing
6. **Document your skill** - Help users understand what it does
7. **Version carefully** - Follow semantic versioning

## Examples

### File Processing Skill

```rust
use openzax_skills_sdk::{skill_main, SkillContext, SkillResult};

#[skill_main]
fn run() -> SkillResult<()> {
    let ctx = SkillContext::new();
    
    let input_path = ctx.get_config("input")
        .ok_or_else(|| SkillError::Config("Missing 'input' config".to_string()))?;
    
    let content = ctx.read_file(input_path)?;
    let processed = process_content(&content);
    
    let output_path = ctx.get_config("output")
        .ok_or_else(|| SkillError::Config("Missing 'output' config".to_string()))?;
    
    ctx.write_file(output_path, &processed)?;
    ctx.log_info("File processed successfully!");
    
    Ok(())
}

fn process_content(content: &[u8]) -> Vec<u8> {
    // Your processing logic
    content.to_vec()
}
```

### API Integration Skill

```rust
use openzax_skills_sdk::{skill_main, SkillContext, SkillResult, SkillError};

#[skill_main]
fn run() -> SkillResult<()> {
    let ctx = SkillContext::new();
    
    let api_key = ctx.get_config("api_key")
        .ok_or_else(|| SkillError::Config("Missing API key".to_string()))?;
    
    let url = format!("https://api.example.com/data?key={}", api_key);
    let response = ctx.http_get(&url)?;
    
    ctx.log_info(&format!("Received: {}", response));
    
    Ok(())
}
```

## Troubleshooting

### Build Errors

**Error**: `target 'wasm32-wasip1' not found`

**Solution**: Install the target:
```bash
rustup target add wasm32-wasip1
```

**Error**: `linking with rust-lld failed`

**Solution**: Update Rust toolchain:
```bash
rustup update
```

### Runtime Errors

**Error**: `Permission denied`

**Solution**: Add required permission to `skill.toml`:
```toml
[permissions]
required = ["fs:read"]
```

**Error**: `Fuel exhausted`

**Solution**: Optimize your code or request higher fuel limit in manifest.

## Resources

- [WIT Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [WASI Preview 1](https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [OpenZax Examples](https://github.com/zAxCoder/OpenZax/tree/master/examples)

## Support

- GitHub Issues: https://github.com/zAxCoder/OpenZax/issues
- Discord: Coming soon
- Documentation: https://openzax.dev/docs

---

**Last Updated**: 2026-03-01  
**SDK Version**: 0.1.0  
**Status**: Phase 2 - Month 5 Implementation
