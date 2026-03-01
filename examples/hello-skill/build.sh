#!/bin/bash
set -e

echo "Building hello-skill WASM module..."

cargo build --target wasm32-wasi --release

echo "WASM module built successfully!"
echo "Output: target/wasm32-wasi/release/hello_skill.wasm"
