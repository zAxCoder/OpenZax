use crate::sandbox::SandboxConfig;
use std::collections::HashMap;
use wasmtime_wasi::WasiCtx;

/// Host context that holds state for WASM instance
pub struct HostContext {
    config: SandboxConfig,
    wasi: Option<WasiCtx>,
    kv_store: HashMap<String, Vec<u8>>,
}

impl HostContext {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            wasi: None,
            kv_store: HashMap::new(),
        }
    }

    pub fn set_wasi(&mut self, wasi: WasiCtx) {
        self.wasi = Some(wasi);
    }

    pub fn wasi(&self) -> Option<&WasiCtx> {
        self.wasi.as_ref()
    }

    pub fn wasi_mut(&mut self) -> Option<&mut WasiCtx> {
        self.wasi.as_mut()
    }

    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    pub fn kv_get(&self, key: &str) -> Option<&[u8]> {
        self.kv_store.get(key).map(|v| v.as_slice())
    }

    pub fn kv_set(&mut self, key: String, value: Vec<u8>) {
        self.kv_store.insert(key, value);
    }

    pub fn kv_delete(&mut self, key: &str) -> bool {
        self.kv_store.remove(key).is_some()
    }
}
