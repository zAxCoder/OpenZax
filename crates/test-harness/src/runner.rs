use std::path::Path;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Instance, Linker, Module, Store};

use crate::mock_host::{register_host_functions, MockHost, MockHostConfig};

// ── Test case definitions ─────────────────────────────────────────────────────

/// A single test case for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestCase {
    /// Human-readable name for the test
    pub name: String,
    /// JSON input passed to the skill's entry point
    pub input: serde_json::Value,
    /// Expected JSON output from the skill
    pub expected_output: serde_json::Value,
    /// Host functions that must have been called (in any order)
    pub expected_calls: Vec<String>,
    /// Whether the skill is expected to return an error
    pub should_fail: bool,
}

impl SkillTestCase {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            input: serde_json::Value::Null,
            expected_output: serde_json::Value::Null,
            expected_calls: Vec::new(),
            should_fail: false,
        }
    }

    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    pub fn expecting_output(mut self, output: serde_json::Value) -> Self {
        self.expected_output = output;
        self
    }

    pub fn expecting_call(mut self, function_name: impl Into<String>) -> Self {
        self.expected_calls.push(function_name.into());
        self
    }

    pub fn expecting_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

// ── Test result types ─────────────────────────────────────────────────────────

/// Result of running a single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub actual_output: serde_json::Value,
    pub expected_output: serde_json::Value,
    /// Host function calls that were made during the test
    pub actual_calls: Vec<String>,
    /// Elapsed milliseconds
    pub duration_ms: u64,
    /// Error message if the test failed
    pub error_message: Option<String>,
}

/// Aggregate result of running an entire test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    /// Total elapsed milliseconds
    pub duration_ms: u64,
    pub results: Vec<TestResult>,
}

impl TestSuiteResult {
    pub fn print_summary(&self) {
        println!(
            "Tests: {} passed, {} failed, {} skipped of {} total  ({} ms)",
            self.passed, self.failed, self.skipped, self.total, self.duration_ms
        );
        for r in &self.results {
            let icon = if r.passed { "✓" } else { "✗" };
            print!("  {} {}  ({} ms)", icon, r.name, r.duration_ms);
            if let Some(err) = &r.error_message {
                print!("  — {}", err);
            }
            println!();
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// ── Runner ────────────────────────────────────────────────────────────────────

/// Test runner for OpenZax WASM skills.
///
/// ## ABI contract
///
/// The skill WASM module must export the following functions:
///
/// ```c
/// // Allocate len bytes in WASM linear memory; returns pointer or 0 on failure.
/// i32 __openzax_alloc(i32 len);
///
/// // Invoke the skill with JSON input.
/// // input_ptr / input_len: pointer+length of the UTF-8 JSON input string.
/// // out_ptr / out_cap:     caller-allocated output buffer.
/// // Returns: number of bytes written to out_ptr, or negative error code.
/// i32 __openzax_skill_call(i32 input_ptr, i32 input_len, i32 out_ptr, i32 out_cap);
/// ```
pub struct TestRunner {
    engine: Engine,
    module: Option<Module>,
}

impl TestRunner {
    pub fn new() -> anyhow::Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine, module: None })
    }

    /// Load a WASM skill module from a file path.
    pub fn load_skill(&mut self, wasm_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let module = Module::from_file(&self.engine, wasm_path.as_ref())?;
        self.module = Some(module);
        Ok(())
    }

    /// Load a WASM skill from raw bytes.
    pub fn load_skill_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let module = Module::new(&self.engine, bytes)?;
        self.module = Some(module);
        Ok(())
    }

    /// Run a single test case against the loaded skill module.
    pub fn run_test(
        &self,
        test_case: &SkillTestCase,
        host_config: MockHostConfig,
    ) -> TestResult {
        let start = Instant::now();

        let run_result = self.execute_test(test_case, host_config);

        let duration_ms = start.elapsed().as_millis() as u64;

        match run_result {
            Ok((actual_output, actual_calls)) => {
                let output_matches = json_deep_eq(&actual_output, &test_case.expected_output);
                let mut missing_calls: Vec<String> = Vec::new();
                for expected_call in &test_case.expected_calls {
                    if !actual_calls.contains(expected_call) {
                        missing_calls.push(expected_call.clone());
                    }
                }

                let passed = if test_case.should_fail {
                    // A successful run when failure was expected = test fail
                    false
                } else {
                    output_matches && missing_calls.is_empty()
                };

                let error_message = if !passed {
                    let mut msgs = Vec::new();
                    if !output_matches {
                        msgs.push(format!(
                            "output mismatch: expected {} got {}",
                            test_case.expected_output, actual_output
                        ));
                    }
                    if !missing_calls.is_empty() {
                        msgs.push(format!("missing calls: {}", missing_calls.join(", ")));
                    }
                    if test_case.should_fail {
                        msgs.push("expected skill to fail but it succeeded".to_string());
                    }
                    Some(msgs.join("; "))
                } else {
                    None
                };

                TestResult {
                    name: test_case.name.clone(),
                    passed,
                    actual_output,
                    expected_output: test_case.expected_output.clone(),
                    actual_calls,
                    duration_ms,
                    error_message,
                }
            }
            Err(e) => {
                let passed = test_case.should_fail;
                TestResult {
                    name: test_case.name.clone(),
                    passed,
                    actual_output: serde_json::Value::Null,
                    expected_output: test_case.expected_output.clone(),
                    actual_calls: Vec::new(),
                    duration_ms,
                    error_message: if passed {
                        None
                    } else {
                        Some(format!("skill error: {}", e))
                    },
                }
            }
        }
    }

    /// Execute a test case and return (actual_output, called_functions).
    fn execute_test(
        &self,
        test_case: &SkillTestCase,
        host_config: MockHostConfig,
    ) -> anyhow::Result<(serde_json::Value, Vec<String>)> {
        let module = self
            .module
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No skill module loaded. Call load_skill() first."))?;

        let mock_host = MockHost::new(host_config);
        let mut store = Store::new(&self.engine, mock_host);

        let mut linker: Linker<MockHost> = Linker::new(&self.engine);
        register_host_functions(&mut linker)?;

        let instance: Instance = linker.instantiate(&mut store, module)?;

        // Allocate input buffer in WASM memory
        let input_json = serde_json::to_string(&test_case.input)?;
        let input_bytes = input_json.as_bytes();

        let alloc_fn = instance
            .get_typed_func::<i32, i32>(&mut store, "__openzax_alloc")
            .ok();

        let (input_ptr, output_ptr, output_cap) = if let Some(alloc) = alloc_fn {
            let input_ptr = alloc.call(&mut store, input_bytes.len() as i32)?;
            let output_cap = 1024 * 1024; // 1 MiB output buffer
            let output_ptr = alloc.call(&mut store, output_cap)?;
            (input_ptr, output_ptr, output_cap)
        } else {
            // Fallback: write directly to the first 64 KiB (for simple test modules)
            let output_cap = 32768i32;
            (0i32, input_bytes.len() as i32, output_cap)
        };

        // Write input into WASM memory
        {
            let mem = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| anyhow::anyhow!("WASM module has no 'memory' export"))?;
            let data = mem.data_mut(&mut store);
            let end = input_ptr as usize + input_bytes.len();
            if end > data.len() {
                anyhow::bail!(
                    "Input buffer overflow: need {} bytes but WASM memory is {} bytes",
                    end,
                    data.len()
                );
            }
            data[input_ptr as usize..end].copy_from_slice(input_bytes);
        }

        // Call the skill entry point
        let skill_fn = instance.get_typed_func::<(i32, i32, i32, i32), i32>(
            &mut store,
            "__openzax_skill_call",
        )?;

        let written = skill_fn.call(
            &mut store,
            (
                input_ptr,
                input_bytes.len() as i32,
                output_ptr,
                output_cap,
            ),
        )?;

        if written < 0 {
            anyhow::bail!("Skill returned error code: {}", written);
        }

        // Read output from WASM memory
        let actual_output = {
            let mem = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| anyhow::anyhow!("WASM module has no 'memory' export"))?;
            let data = mem.data(&store);
            let start = output_ptr as usize;
            let end = start + written as usize;
            let output_bytes = data.get(start..end).unwrap_or(&[]);
            serde_json::from_slice(output_bytes)?
        };

        let actual_calls: Vec<String> = store
            .data()
            .call_log
            .iter()
            .map(|r| r.function_name.clone())
            .collect();

        Ok((actual_output, actual_calls))
    }

    /// Run all test cases and return an aggregate result.
    pub fn run_suite(
        &self,
        test_cases: &[SkillTestCase],
        host_config_factory: impl Fn() -> MockHostConfig,
    ) -> TestSuiteResult {
        let suite_start = Instant::now();
        let mut results = Vec::with_capacity(test_cases.len());

        for tc in test_cases {
            let config = host_config_factory();
            let result = self.run_test(tc, config);
            results.push(result);
        }

        let duration_ms = suite_start.elapsed().as_millis() as u64;
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        TestSuiteResult {
            total: results.len(),
            passed,
            failed,
            skipped: 0,
            duration_ms,
            results,
        }
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new().expect("Failed to create TestRunner")
    }
}

// ── JSON comparison ───────────────────────────────────────────────────────────

/// Deep equality comparison for JSON values (order-insensitive for objects).
pub fn json_deep_eq(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value::*;
    match (a, b) {
        (Null, Null) => true,
        (Bool(x), Bool(y)) => x == y,
        (Number(x), Number(y)) => x == y,
        (String(x), String(y)) => x == y,
        (Array(xs), Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| json_deep_eq(x, y))
        }
        (Object(a_map), Object(b_map)) => {
            a_map.len() == b_map.len()
                && a_map
                    .iter()
                    .all(|(k, v)| b_map.get(k).map_or(false, |bv| json_deep_eq(v, bv)))
        }
        _ => false,
    }
}
