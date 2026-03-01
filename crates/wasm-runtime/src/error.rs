use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("WASM compilation error: {0}")]
    Compilation(String),

    #[error("WASM instantiation error: {0}")]
    Instantiation(String),

    #[error("WASM execution error: {0}")]
    Execution(String),

    #[error("Fuel exhausted: {0}")]
    FuelExhausted(String),

    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Host function error: {0}")]
    HostFunction(String),

    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type WasmResult<T> = std::result::Result<T, WasmError>;
