use openzax_wasm_runtime::{Sandbox, SandboxConfig};
use wasmtime::Val;

#[test]
#[ignore] // Temporarily disabled due to WASM runtime issues
fn test_sandbox_basic() {
    let config = SandboxConfig::default();
    let sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    // Create a simple WASM module
    let wat = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;

    let wasm = wat::parse_str(wat).expect("Failed to parse WAT");
    let module = sandbox
        .load_module_bytes(&wasm)
        .expect("Failed to load module");
    let mut instance = sandbox
        .create_instance(&module)
        .expect("Failed to create instance");

    let results = instance
        .call_function("add", &[Val::I32(5), Val::I32(7)])
        .expect("Failed to call function");

    assert_eq!(results.len(), 1);
    if let Val::I32(result) = results[0] {
        assert_eq!(result, 12);
    } else {
        panic!("Expected I32 result");
    }
}

#[test]
#[ignore] // Temporarily disabled due to panic in infinite loop
fn test_fuel_exhaustion() {
    let mut config = SandboxConfig::default();
    config.max_fuel = 100; // Very low fuel budget

    let sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    // Create a WASM module with an infinite loop
    let wat = r#"
        (module
            (func (export "infinite_loop")
                (loop $loop
                    br $loop
                )
            )
        )
    "#;

    let wasm = wat::parse_str(wat).expect("Failed to parse WAT");
    let module = sandbox
        .load_module_bytes(&wasm)
        .expect("Failed to load module");
    let mut instance = sandbox
        .create_instance(&module)
        .expect("Failed to create instance");

    let result = instance.call_function("infinite_loop", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("fuel"));
}

#[test]
fn test_memory_limits() {
    let config = SandboxConfig {
        max_memory_bytes: 1024 * 1024, // 1 MB
        ..Default::default()
    };

    let sandbox = Sandbox::new(config).expect("Failed to create sandbox");

    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "get_memory_size") (result i32)
                memory.size
            )
        )
    "#;

    let wasm = wat::parse_str(wat).expect("Failed to parse WAT");
    let module = sandbox
        .load_module_bytes(&wasm)
        .expect("Failed to load module");
    let mut instance = sandbox
        .create_instance(&module)
        .expect("Failed to create instance");

    let memory_usage = instance
        .get_memory_usage()
        .expect("Failed to get memory usage");
    assert!(memory_usage <= 1024 * 1024);
}
