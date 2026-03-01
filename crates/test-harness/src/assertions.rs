use crate::mock_host::MockHost;
use crate::runner::json_deep_eq;

/// Assert that `actual` deeply equals `expected`.
///
/// Panics with a diff-style message on failure.
pub fn assert_output_eq(actual: &serde_json::Value, expected: &serde_json::Value) {
    if !json_deep_eq(actual, expected) {
        panic!(
            "Output mismatch\n  expected: {}\n  actual  : {}",
            serde_json::to_string_pretty(expected).unwrap_or_else(|_| expected.to_string()),
            serde_json::to_string_pretty(actual).unwrap_or_else(|_| actual.to_string()),
        );
    }
}

/// Assert that a specific host function was called at least once.
///
/// `function_name` must match the ABI name (e.g. `"__openzax_kv_put"`).
pub fn assert_call_made(host: &MockHost, function_name: &str) {
    if !host.was_called(function_name) {
        let all_calls: Vec<&str> = host
            .call_log
            .iter()
            .map(|r| r.function_name.as_str())
            .collect();
        panic!(
            "Expected host function '{}' to be called, but it was not.\n  Calls made: {:?}",
            function_name, all_calls
        );
    }
}

/// Assert that a host function was called exactly `expected_count` times.
pub fn assert_call_count(host: &MockHost, function_name: &str, expected_count: usize) {
    let actual = host.call_count(function_name);
    if actual != expected_count {
        panic!(
            "Expected '{}' to be called {} time(s), but it was called {} time(s).",
            function_name, expected_count, actual
        );
    }
}

/// Assert that no HTTP fetch calls were made during the test.
///
/// Useful to verify that a skill respects an offline-only constraint.
pub fn assert_no_network(host: &MockHost) {
    let http_calls: Vec<_> = host
        .call_log
        .iter()
        .filter(|r| r.function_name.contains("http"))
        .collect();
    if !http_calls.is_empty() {
        let urls: Vec<_> = http_calls
            .iter()
            .filter_map(|r| r.args["url"].as_str())
            .collect();
        panic!(
            "Expected no network calls, but {} HTTP request(s) were made: {:?}",
            http_calls.len(),
            urls
        );
    }
}

/// Assert that no filesystem writes were performed during the test.
pub fn assert_no_fs_writes(host: &MockHost) {
    let write_calls: Vec<_> = host
        .call_log
        .iter()
        .filter(|r| r.function_name == "__openzax_write_file")
        .collect();
    if !write_calls.is_empty() {
        let paths: Vec<_> = write_calls
            .iter()
            .filter_map(|r| r.args["path"].as_str())
            .collect();
        panic!(
            "Expected no filesystem writes, but {} write(s) were made to: {:?}",
            write_calls.len(),
            paths
        );
    }
}

/// Assert that the skill emitted a specific named event.
pub fn assert_event_emitted(host: &MockHost, event_name: &str) {
    let emitted = host.call_log.iter().any(|r| {
        r.function_name == "__openzax_emit_event"
            && r.args["name"].as_str() == Some(event_name)
    });
    if !emitted {
        panic!(
            "Expected event '{}' to be emitted, but it was not.",
            event_name
        );
    }
}

/// Assert that a specific key was written to the KV store.
pub fn assert_kv_written(host: &MockHost, key: &str) {
    let written = host.call_log.iter().any(|r| {
        r.function_name == "__openzax_kv_put" && r.args["key"].as_str() == Some(key)
    });
    if !written {
        panic!("Expected KV key '{}' to be written, but it was not.", key);
    }
}

/// Assert that a specific key was written to the KV store with a specific value.
pub fn assert_kv_written_with(host: &MockHost, key: &str, expected_value: &str) {
    let record = host.call_log.iter().find(|r| {
        r.function_name == "__openzax_kv_put" && r.args["key"].as_str() == Some(key)
    });
    match record {
        None => panic!("Expected KV key '{}' to be written, but it was not.", key),
        Some(r) => {
            let actual_value = r.args["value"].as_str().unwrap_or("");
            if actual_value != expected_value {
                panic!(
                    "KV key '{}' was written with value '{}', expected '{}'",
                    key, actual_value, expected_value
                );
            }
        }
    }
}

/// Assert that a log message at `level` was produced containing `text`.
pub fn assert_logged(host: &MockHost, level: &str, text: &str) {
    let found = host.call_log.iter().any(|r| {
        r.function_name == "__openzax_log"
            && r.args["level"].as_str() == Some(level)
            && r.args["message"]
                .as_str()
                .map(|m| m.contains(text))
                .unwrap_or(false)
    });
    if !found {
        panic!(
            "Expected a log message at level '{}' containing '{}', but none was found.",
            level, text
        );
    }
}
