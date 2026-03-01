# Hello Skill Example

A minimal OpenZax skill demonstrating WASM compilation and host function usage.

## Building

```bash
# Install Rust WASM target
rustup target add wasm32-wasi

# Build the skill
./build.sh

# Or manually
cargo build --target wasm32-wasi --release
```

## Output

The compiled WASM module will be at:
```
target/wasm32-wasi/release/hello_skill.wasm
```

## Functions

- `hello() -> i32`: Logs a message and returns 42
- `add(a: i32, b: i32) -> i32`: Adds two numbers

## Host Functions Used

- `openzax:host/logging::log`: Structured logging

## Testing

```rust
use openzax_wasm_runtime::{Sandbox, SandboxConfig};
use wasmtime::Val;

let sandbox = Sandbox::new(SandboxConfig::default())?;
let module = sandbox.load_module("target/wasm32-wasi/release/hello_skill.wasm")?;
let mut instance = sandbox.create_instance(&module)?;

// Call hello function
let results = instance.call_function("hello", &[])?;
assert_eq!(results[0].unwrap_i32(), 42);

// Call add function
let results = instance.call_function("add", &[Val::I32(5), Val::I32(7)])?;
assert_eq!(results[0].unwrap_i32(), 12);
```

## Size

Optimized WASM binary: ~100-200 KB (with `opt-level = "z"` and `strip = true`)
