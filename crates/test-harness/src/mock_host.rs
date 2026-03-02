use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a MockHost instance
#[derive(Debug, Clone, Default)]
pub struct MockHostConfig {
    /// Minimum log level to record ("trace"|"debug"|"info"|"warn"|"error")
    pub log_level: String,
    /// Virtual filesystem: path → content
    pub mock_fs: HashMap<String, Vec<u8>>,
    /// Virtual key-value store
    pub mock_kv: HashMap<String, String>,
    /// Canned HTTP responses: URL → MockHttpResponse
    pub mock_http_responses: HashMap<String, MockHttpResponse>,
    /// Configuration values available to the skill
    pub mock_config: HashMap<String, String>,
}

impl MockHostConfig {
    pub fn new() -> Self {
        Self {
            log_level: "info".to_string(),
            ..Default::default()
        }
    }

    pub fn with_file(mut self, path: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        self.mock_fs.insert(path.into(), content.into());
        self
    }

    pub fn with_kv(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.mock_kv.insert(key.into(), value.into());
        self
    }

    pub fn with_config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.mock_config.insert(key.into(), value.into());
        self
    }

    pub fn with_http_response(
        mut self,
        url: impl Into<String>,
        response: MockHttpResponse,
    ) -> Self {
        self.mock_http_responses.insert(url.into(), response);
        self
    }
}

/// A canned HTTP response for the mock host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockHttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl MockHttpResponse {
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: body.into(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }

    pub fn error(status: u16, msg: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: msg.into(),
        }
    }
}

/// A recorded invocation of a host function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCallRecord {
    /// Name of the host function that was called
    pub function_name: String,
    /// Arguments passed (JSON-serialized)
    pub args: serde_json::Value,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Return value (JSON-serialized)
    pub result: serde_json::Value,
}

/// The mock host environment used during skill testing.
///
/// This is the runtime state stored inside the wasmtime `Store`.  
/// All host ABI functions operate on this struct via `Caller<MockHost>`.
#[derive(Debug, Clone)]
pub struct MockHost {
    pub config: MockHostConfig,
    /// Chronological record of every host function that was called
    pub call_log: Vec<HostCallRecord>,
}

impl MockHost {
    pub fn new(config: MockHostConfig) -> Self {
        Self {
            config,
            call_log: Vec::new(),
        }
    }

    // ── Logging ──────────────────────────────────────────────────────────────

    pub fn host_log(&mut self, level: i32, message: String) {
        let level_name = match level {
            0 => "trace",
            1 => "debug",
            2 => "info",
            3 => "warn",
            4 => "error",
            _ => "unknown",
        };
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_log".to_string(),
            args: serde_json::json!({ "level": level_name, "message": message }),
            timestamp: now_rfc3339(),
            result: serde_json::Value::Null,
        });
    }

    // ── Config ───────────────────────────────────────────────────────────────

    pub fn host_config_get(&mut self, key: &str) -> Option<String> {
        let value = self.config.mock_config.get(key).cloned();
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_config_get".to_string(),
            args: serde_json::json!({ "key": key }),
            timestamp: now_rfc3339(),
            result: match &value {
                Some(v) => serde_json::Value::String(v.clone()),
                None => serde_json::Value::Null,
            },
        });
        value
    }

    pub fn host_config_set(&mut self, key: String, value: String) {
        self.config.mock_config.insert(key.clone(), value.clone());
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_config_set".to_string(),
            args: serde_json::json!({ "key": key, "value": value }),
            timestamp: now_rfc3339(),
            result: serde_json::Value::Null,
        });
    }

    // ── Filesystem ───────────────────────────────────────────────────────────

    pub fn host_read_file(&mut self, path: &str) -> Option<Vec<u8>> {
        let data = self.config.mock_fs.get(path).cloned();
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_read_file".to_string(),
            args: serde_json::json!({ "path": path }),
            timestamp: now_rfc3339(),
            result: match &data {
                Some(b) => serde_json::json!({ "bytes": b.len() }),
                None => serde_json::Value::Null,
            },
        });
        data
    }

    pub fn host_write_file(&mut self, path: String, data: Vec<u8>) -> bool {
        let len = data.len();
        self.config.mock_fs.insert(path.clone(), data);
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_write_file".to_string(),
            args: serde_json::json!({ "path": path, "bytes": len }),
            timestamp: now_rfc3339(),
            result: serde_json::json!(true),
        });
        true
    }

    // ── HTTP ─────────────────────────────────────────────────────────────────

    pub fn host_http_fetch(
        &mut self,
        url: &str,
        method: &str,
        _headers: &str,
        _body: &[u8],
    ) -> MockHttpResponse {
        let response = self
            .config
            .mock_http_responses
            .get(url)
            .cloned()
            .unwrap_or_else(MockHttpResponse::not_found);

        self.call_log.push(HostCallRecord {
            function_name: "__openzax_http_fetch".to_string(),
            args: serde_json::json!({ "url": url, "method": method }),
            timestamp: now_rfc3339(),
            result: serde_json::json!({ "status": response.status }),
        });

        response
    }

    // ── KV store ─────────────────────────────────────────────────────────────

    pub fn host_kv_get(&mut self, key: &str) -> Option<String> {
        let value = self.config.mock_kv.get(key).cloned();
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_kv_get".to_string(),
            args: serde_json::json!({ "key": key }),
            timestamp: now_rfc3339(),
            result: match &value {
                Some(v) => serde_json::Value::String(v.clone()),
                None => serde_json::Value::Null,
            },
        });
        value
    }

    pub fn host_kv_put(&mut self, key: String, value: String) {
        self.config.mock_kv.insert(key.clone(), value.clone());
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_kv_put".to_string(),
            args: serde_json::json!({ "key": key, "value": value }),
            timestamp: now_rfc3339(),
            result: serde_json::Value::Null,
        });
    }

    pub fn host_kv_delete(&mut self, key: &str) {
        self.config.mock_kv.remove(key);
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_kv_delete".to_string(),
            args: serde_json::json!({ "key": key }),
            timestamp: now_rfc3339(),
            result: serde_json::Value::Null,
        });
    }

    // ── Events ───────────────────────────────────────────────────────────────

    pub fn host_emit_event(&mut self, name: &str, data: &str) {
        self.call_log.push(HostCallRecord {
            function_name: "__openzax_emit_event".to_string(),
            args: serde_json::json!({ "name": name, "data": data }),
            timestamp: now_rfc3339(),
            result: serde_json::Value::Null,
        });
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Return all calls to a specific host function
    pub fn calls_to(&self, function_name: &str) -> Vec<&HostCallRecord> {
        self.call_log
            .iter()
            .filter(|r| r.function_name == function_name)
            .collect()
    }

    /// Count calls to a specific host function
    pub fn call_count(&self, function_name: &str) -> usize {
        self.calls_to(function_name).len()
    }

    /// Check whether any call was made to a specific host function
    pub fn was_called(&self, function_name: &str) -> bool {
        self.call_count(function_name) > 0
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ── wasmtime host function registration ──────────────────────────────────────

/// Register all OpenZax host ABI functions on `linker` so that a skill WASM
/// module can be instantiated against the `MockHost` stored in the `Store`.
///
/// The host ABI convention:
///   - Strings are passed as (ptr: i32, len: i32) referencing WASM linear memory
///   - Output strings are written to a caller-allocated buffer at (out_ptr: i32)
///   - Return value is the written byte count, or -1 on miss / error
pub fn register_host_functions(linker: &mut wasmtime::Linker<MockHost>) -> anyhow::Result<()> {
    // __openzax_log(level: i32, msg_ptr: i32, msg_len: i32)
    linker.func_wrap(
        "env",
        "__openzax_log",
        |mut caller: wasmtime::Caller<MockHost>, level: i32, msg_ptr: i32, msg_len: i32| {
            let message = read_wasm_string(&mut caller, msg_ptr, msg_len).unwrap_or_default();
            caller.data_mut().host_log(level, message);
        },
    )?;

    // __openzax_config_get(key_ptr, key_len, out_ptr, out_cap) -> i32
    linker.func_wrap(
        "env",
        "__openzax_config_get",
        |mut caller: wasmtime::Caller<MockHost>,
         key_ptr: i32,
         key_len: i32,
         out_ptr: i32,
         out_cap: i32|
         -> i32 {
            let key = read_wasm_string(&mut caller, key_ptr, key_len).unwrap_or_default();
            let value = caller.data_mut().host_config_get(&key);
            match value {
                Some(v) => {
                    let bytes = v.into_bytes();
                    let len = bytes.len().min(out_cap as usize);
                    write_wasm_bytes(&mut caller, out_ptr, &bytes[..len]).unwrap_or(-1)
                }
                None => -1,
            }
        },
    )?;

    // __openzax_config_set(key_ptr, key_len, val_ptr, val_len)
    linker.func_wrap(
        "env",
        "__openzax_config_set",
        |mut caller: wasmtime::Caller<MockHost>,
         key_ptr: i32,
         key_len: i32,
         val_ptr: i32,
         val_len: i32| {
            let key = read_wasm_string(&mut caller, key_ptr, key_len).unwrap_or_default();
            let value = read_wasm_string(&mut caller, val_ptr, val_len).unwrap_or_default();
            caller.data_mut().host_config_set(key, value);
        },
    )?;

    // __openzax_read_file(path_ptr, path_len, out_ptr, out_cap) -> i32
    linker.func_wrap(
        "env",
        "__openzax_read_file",
        |mut caller: wasmtime::Caller<MockHost>,
         path_ptr: i32,
         path_len: i32,
         out_ptr: i32,
         out_cap: i32|
         -> i32 {
            let path = read_wasm_string(&mut caller, path_ptr, path_len).unwrap_or_default();
            let data_opt = caller.data_mut().host_read_file(&path);
            match data_opt {
                Some(data) => {
                    let len = data.len().min(out_cap as usize);
                    write_wasm_bytes(&mut caller, out_ptr, &data[..len]).unwrap_or(-1)
                }
                None => -1,
            }
        },
    )?;

    // __openzax_write_file(path_ptr, path_len, data_ptr, data_len) -> i32
    linker.func_wrap(
        "env",
        "__openzax_write_file",
        |mut caller: wasmtime::Caller<MockHost>,
         path_ptr: i32,
         path_len: i32,
         data_ptr: i32,
         data_len: i32|
         -> i32 {
            let path = read_wasm_string(&mut caller, path_ptr, path_len).unwrap_or_default();
            let data = read_wasm_bytes(&mut caller, data_ptr, data_len).unwrap_or_default();
            if caller.data_mut().host_write_file(path, data) {
                0
            } else {
                -1
            }
        },
    )?;

    // __openzax_http_fetch(url_ptr, url_len, method_ptr, method_len,
    //   headers_ptr, headers_len, body_ptr, body_len, out_ptr, out_cap) -> i32
    linker.func_wrap(
        "env",
        "__openzax_http_fetch",
        |mut caller: wasmtime::Caller<MockHost>,
         url_ptr: i32,
         url_len: i32,
         method_ptr: i32,
         method_len: i32,
         headers_ptr: i32,
         headers_len: i32,
         body_ptr: i32,
         body_len: i32,
         out_ptr: i32,
         out_cap: i32|
         -> i32 {
            let url = read_wasm_string(&mut caller, url_ptr, url_len).unwrap_or_default();
            let method = read_wasm_string(&mut caller, method_ptr, method_len).unwrap_or_default();
            let headers =
                read_wasm_string(&mut caller, headers_ptr, headers_len).unwrap_or_default();
            let body = read_wasm_bytes(&mut caller, body_ptr, body_len).unwrap_or_default();
            let response = caller
                .data_mut()
                .host_http_fetch(&url, &method, &headers, &body);

            let response_json = serde_json::json!({
                "status": response.status,
                "headers": response.headers,
                "body": response.body,
            });
            let bytes = serde_json::to_vec(&response_json).unwrap_or_default();
            let len = bytes.len().min(out_cap as usize);
            write_wasm_bytes(&mut caller, out_ptr, &bytes[..len]).unwrap_or(-1)
        },
    )?;

    // __openzax_kv_get(key_ptr, key_len, out_ptr, out_cap) -> i32
    linker.func_wrap(
        "env",
        "__openzax_kv_get",
        |mut caller: wasmtime::Caller<MockHost>,
         key_ptr: i32,
         key_len: i32,
         out_ptr: i32,
         out_cap: i32|
         -> i32 {
            let key = read_wasm_string(&mut caller, key_ptr, key_len).unwrap_or_default();
            let value = caller.data_mut().host_kv_get(&key);
            match value {
                Some(v) => {
                    let bytes = v.into_bytes();
                    let len = bytes.len().min(out_cap as usize);
                    write_wasm_bytes(&mut caller, out_ptr, &bytes[..len]).unwrap_or(-1)
                }
                None => -1,
            }
        },
    )?;

    // __openzax_kv_put(key_ptr, key_len, val_ptr, val_len)
    linker.func_wrap(
        "env",
        "__openzax_kv_put",
        |mut caller: wasmtime::Caller<MockHost>,
         key_ptr: i32,
         key_len: i32,
         val_ptr: i32,
         val_len: i32| {
            let key = read_wasm_string(&mut caller, key_ptr, key_len).unwrap_or_default();
            let value = read_wasm_string(&mut caller, val_ptr, val_len).unwrap_or_default();
            caller.data_mut().host_kv_put(key, value);
        },
    )?;

    // __openzax_kv_delete(key_ptr, key_len)
    linker.func_wrap(
        "env",
        "__openzax_kv_delete",
        |mut caller: wasmtime::Caller<MockHost>, key_ptr: i32, key_len: i32| {
            let key = read_wasm_string(&mut caller, key_ptr, key_len).unwrap_or_default();
            caller.data_mut().host_kv_delete(&key);
        },
    )?;

    // __openzax_emit_event(name_ptr, name_len, data_ptr, data_len)
    linker.func_wrap(
        "env",
        "__openzax_emit_event",
        |mut caller: wasmtime::Caller<MockHost>,
         name_ptr: i32,
         name_len: i32,
         data_ptr: i32,
         data_len: i32| {
            let name = read_wasm_string(&mut caller, name_ptr, name_len).unwrap_or_default();
            let data = read_wasm_string(&mut caller, data_ptr, data_len).unwrap_or_default();
            caller.data_mut().host_emit_event(&name, &data);
        },
    )?;

    Ok(())
}

// ── WASM memory helpers ───────────────────────────────────────────────────────

fn read_wasm_string(caller: &mut wasmtime::Caller<MockHost>, ptr: i32, len: i32) -> Option<String> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    // Read into owned Vec immediately to release the borrow
    let owned: Vec<u8> = mem
        .data(&*caller)
        .get(ptr as usize..(ptr + len) as usize)?
        .to_vec();
    String::from_utf8(owned).ok()
}

fn read_wasm_bytes(caller: &mut wasmtime::Caller<MockHost>, ptr: i32, len: i32) -> Option<Vec<u8>> {
    let mem = caller.get_export("memory").and_then(|e| e.into_memory())?;
    mem.data(&*caller)
        .get(ptr as usize..(ptr + len) as usize)
        .map(|s| s.to_vec())
}

#[allow(dead_code)]
fn write_wasm_string(
    caller: &mut wasmtime::Caller<MockHost>,
    out_ptr: i32,
    value: &str,
) -> anyhow::Result<i32> {
    write_wasm_bytes(caller, out_ptr, value.as_bytes())
}

fn write_wasm_bytes(
    caller: &mut wasmtime::Caller<MockHost>,
    out_ptr: i32,
    bytes: &[u8],
) -> anyhow::Result<i32> {
    let mem = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| anyhow::anyhow!("No memory export"))?;
    let start = out_ptr as usize;
    let end = start
        .checked_add(bytes.len())
        .ok_or_else(|| anyhow::anyhow!("Overflow"))?;
    mem.data_mut(caller)
        .get_mut(start..end)
        .ok_or_else(|| anyhow::anyhow!("Out of bounds write"))?
        .copy_from_slice(bytes);
    Ok(bytes.len() as i32)
}
