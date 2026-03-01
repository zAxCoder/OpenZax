#[no_mangle]
pub extern "C" fn hello() -> i32 {
    log(2, "Hello from WASM skill!");
    42
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Host function imports
#[link(wasm_import_module = "openzax:host/logging")]
extern "C" {
    fn log(level: i32, ptr: i32, len: i32);
}

fn log(level: i32, message: &str) {
    unsafe {
        log(level, message.as_ptr() as i32, message.len() as i32);
    }
}
