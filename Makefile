.PHONY: help build test clean fmt clippy run install-deps example-skill docs

help:
	@echo "OpenZax Development Commands"
	@echo ""
	@echo "  make build         - Build all crates"
	@echo "  make test          - Run all tests"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make fmt           - Format code"
	@echo "  make clippy        - Run clippy lints"
	@echo "  make run           - Run terminal shell"
	@echo "  make install-deps  - Install development dependencies"
	@echo "  make example-skill - Build example WASM skill"
	@echo "  make docs          - Generate documentation"

build:
	cargo build --release

test:
	cargo test --all-features

clean:
	cargo clean
	rm -rf target/
	rm -rf examples/hello-skill/target/

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

run:
	cargo run --release --bin openzax shell

install-deps:
	rustup target add wasm32-wasi
	cargo install wasm-opt || true

example-skill:
	cd examples/hello-skill && cargo build --target wasm32-wasi --release
	@echo "WASM module: examples/hello-skill/target/wasm32-wasi/release/hello_skill.wasm"

docs:
	cargo doc --no-deps --open
