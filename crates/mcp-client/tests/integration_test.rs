use openzax_mcp_client::protocol::*;
use serde_json::json;

#[test]
fn test_json_rpc_request_serialization() {
    let request = JsonRpcRequest::new(1, "test_method", Some(json!({"key": "value"})));

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
    assert!(serialized.contains("\"method\":\"test_method\""));
    assert!(serialized.contains("\"id\":1"));
}

#[test]
fn test_json_rpc_notification() {
    let notification = JsonRpcRequest::notification("notify", None);

    assert_eq!(notification.jsonrpc, "2.0");
    assert_eq!(notification.method, "notify");
    assert!(notification.id.is_none());
}

#[test]
fn test_tool_definition() {
    let tool = Tool {
        name: "test_tool".to_string(),
        description: Some("A test tool".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "arg1": {"type": "string"}
            }
        }),
    };

    assert_eq!(tool.name, "test_tool");
    assert!(tool.description.is_some());
}

#[test]
fn test_initialize_request() {
    let request = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            roots: Some(RootsCapability { list_changed: true }),
            sampling: None,
        },
        client_info: ClientInfo {
            name: "TestClient".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let serialized = serde_json::to_value(&request).unwrap();
    assert_eq!(serialized["protocolVersion"], "2024-11-05");
    assert_eq!(serialized["clientInfo"]["name"], "TestClient");
}
