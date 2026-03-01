use crate::{WasmError, WasmResult};
use crate::host::HostContext;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use std::path::Path;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory in bytes (default: 8 MB)
    pub max_memory_bytes: usize,
    
    /// CPU fuel budget (default: 1 billion instructions)
    pub max_fuel: u64,
    
    /// Enable WASI preview 2 support
    pub wasi_preview2: bool,
    
    /// Allowed filesystem paths (glob patterns)
    pub fs_read_paths: Vec<String>,
    pub fs_write_paths: Vec<String>,
    
    /// Allowed network hosts
    pub network_allow: Vec<String>,
    
    /// Enable subprocess spawning
    pub allow_subprocess: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 8 * 1024 * 1024, // 8 MB
            max_fuel: 1_000_000_000, // 1 billion instructions
            wasi_preview2: true,
            fs_read_paths: vec![],
            fs_write_paths: vec![],
            network_allow: vec![],
            allow_subprocess: false,
        }
    }
}

pub struct Sandbox {
    engine: Engine,
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> WasmResult<Self> {
        let mut engine_config = Config::new();
        
        // Enable fuel metering for CPU limits
        engine_config.consume_fuel(true);
        
        // Enable epoch interruption for timeout handling
        engine_config.epoch_interruption(true);
        
        // Enable WASI support
        engine_config.wasm_component_model(config.wasi_preview2);
        
        let engine = Engine::new(&engine_config)
            .map_err(|e| WasmError::Compilation(e.to_string()))?;
        
        info!("WASM sandbox initialized with max_memory={} bytes, max_fuel={}", 
              config.max_memory_bytes, config.max_fuel);
        
        Ok(Self { engine, config })
    }

    pub fn load_module<P: AsRef<Path>>(&self, wasm_path: P) -> WasmResult<Module> {
        debug!("Loading WASM module from {:?}", wasm_path.as_ref());
        
        let module = Module::from_file(&self.engine, wasm_path)
            .map_err(|e| WasmError::Compilation(format!("Failed to load module: {}", e)))?;
        
        info!("WASM module loaded successfully");
        Ok(module)
    }

    pub fn load_module_bytes(&self, wasm_bytes: &[u8]) -> WasmResult<Module> {
        debug!("Loading WASM module from bytes ({} bytes)", wasm_bytes.len());
        
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::Compilation(format!("Failed to load module: {}", e)))?;
        
        info!("WASM module loaded successfully from bytes");
        Ok(module)
    }

    pub fn create_instance(&self, module: &Module) -> WasmResult<SandboxInstance> {
        let mut store = Store::new(&self.engine, HostContext::new(self.config.clone()));
        
        // Set fuel budget
        store.set_fuel(self.config.max_fuel)
            .map_err(|e| WasmError::Execution(format!("Failed to set fuel: {}", e)))?;
        
        // Build WASI context with restricted permissions
        let wasi_ctx = self.build_wasi_context()?;
        store.data_mut().set_wasi(wasi_ctx);
        
        // Create linker with host functions
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| WasmError::Instantiation(format!("Failed to add WASI to linker: {}", e)))?;
        
        // Add custom host functions
        self.add_host_functions(&mut linker)?;
        
        // Instantiate the module
        let instance = linker.instantiate(&mut store, module)
            .map_err(|e| WasmError::Instantiation(format!("Failed to instantiate: {}", e)))?;
        
        info!("WASM instance created successfully");
        
        Ok(SandboxInstance {
            store,
            instance,
            config: self.config.clone(),
        })
    }

    fn build_wasi_context(&self) -> WasmResult<WasiCtx> {
        let mut builder = WasiCtxBuilder::new();
        
        // Inherit stdio (can be customized later)
        builder = builder.inherit_stdio();
        
        // Add allowed filesystem paths
        for path in &self.config.fs_read_paths {
            debug!("Adding read-only path: {}", path);
            // TODO: Implement virtual filesystem overlay
        }
        
        for path in &self.config.fs_write_paths {
            debug!("Adding writable path: {}", path);
            // TODO: Implement virtual filesystem overlay
        }
        
        Ok(builder.build())
    }

    fn add_host_functions(&self, linker: &mut Linker<HostContext>) -> WasmResult<()> {
        // openzax:host/logging
        linker.func_wrap(
            "openzax:host/logging",
            "log",
            |mut caller: Caller<'_, HostContext>, level: i32, ptr: i32, len: i32| {
                let memory = caller.get_export("memory")
                    .and_then(|e| e.into_memory())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get memory"))?;
                
                let data = memory.data(&caller);
                let message = std::str::from_utf8(&data[ptr as usize..(ptr + len) as usize])
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))?;
                
                match level {
                    0 => tracing::trace!("[WASM] {}", message),
                    1 => tracing::debug!("[WASM] {}", message),
                    2 => tracing::info!("[WASM] {}", message),
                    3 => tracing::warn!("[WASM] {}", message),
                    4 => tracing::error!("[WASM] {}", message),
                    _ => tracing::info!("[WASM] {}", message),
                }
                
                Ok(())
            },
        ).map_err(|e| WasmError::HostFunction(e.to_string()))?;
        
        // openzax:host/config
        linker.func_wrap(
            "openzax:host/config",
            "get",
            |_caller: Caller<'_, HostContext>, _key_ptr: i32, _key_len: i32| -> i32 {
                // TODO: Implement config get
                0
            },
        ).map_err(|e| WasmError::HostFunction(e.to_string()))?;
        
        Ok(())
    }
}

pub struct SandboxInstance {
    store: Store<HostContext>,
    instance: Instance,
    config: SandboxConfig,
}

impl SandboxInstance {
    pub fn call_function(&mut self, name: &str, args: &[Val]) -> WasmResult<Vec<Val>> {
        let func = self.instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| WasmError::Execution(format!("Function '{}' not found", name)))?;
        
        let mut results = vec![Val::I32(0); func.ty(&self.store).results().len()];
        
        func.call(&mut self.store, args, &mut results)
            .map_err(|e| {
                if e.to_string().contains("fuel") {
                    WasmError::FuelExhausted(format!("CPU budget exhausted calling '{}'", name))
                } else {
                    WasmError::Execution(format!("Function '{}' failed: {}", name, e))
                }
            })?;
        
        Ok(results)
    }

    pub fn get_remaining_fuel(&self) -> WasmResult<u64> {
        self.store.get_fuel()
            .map_err(|e| WasmError::Execution(format!("Failed to get fuel: {}", e)))
    }

    pub fn get_memory_usage(&self) -> WasmResult<usize> {
        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| WasmError::Execution("Memory export not found".to_string()))?;
        
        Ok(memory.data_size(&self.store))
    }

    pub fn check_limits(&self) -> WasmResult<()> {
        let fuel = self.get_remaining_fuel()?;
        if fuel == 0 {
            return Err(WasmError::FuelExhausted("CPU budget exhausted".to_string()));
        }
        
        let memory = self.get_memory_usage()?;
        if memory > self.config.max_memory_bytes {
            return Err(WasmError::MemoryLimitExceeded(
                format!("Memory usage {} exceeds limit {}", memory, self.config.max_memory_bytes)
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_sandbox_config_defaults() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_memory_bytes, 8 * 1024 * 1024);
        assert_eq!(config.max_fuel, 1_000_000_000);
        assert!(config.wasi_preview2);
        assert!(!config.allow_subprocess);
    }
}
