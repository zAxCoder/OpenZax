# OpenZax MCP Client Guide

## Overview

The Model Context Protocol (MCP) is an open standard for connecting AI assistants to external tools and data sources. OpenZax implements a full-featured MCP client in Rust with support for multiple transports.

## Quick Start

### Installing an MCP Server

```bash
# Install filesystem MCP server (Node.js required)
npm install -g @modelcontextprotocol/server-filesystem

# Install GitHub MCP server
npm install -g @modelcontextprotocol/server-github
```

### Basic Usage

```rust
use openzax_mcp_client::{McpClient, McpClientConfig};
use openzax_mcp_client::transport::StdioTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create stdio transport
    let transport = StdioTransport::new("npx", &[
        "-y".to_string(),
        "@modelcontextprotocol/server-filesystem".to_string(),
        "/allowed/path".to_string(),
    ]).await?;

    // Create client
    let config = McpClientConfig::default();
    let client = McpClient::new(Box::new(transport), config);

    // Initialize
    let init = client.initialize().await?;
    println!("Connected to: {}", init.server_info.name);

    // List tools
    let tools = client.list_tools().await?;
    for tool in tools {
        println!("- {}", tool.name);
    }

    Ok(())
}
```

## Transports

### Stdio Transport

For local MCP servers that communicate via stdin/stdout:

```rust
let transport = StdioTransport::new("command", &["arg1", "arg2"]).await?;
```

**Use cases:**
- Local filesystem access
- Git operations
- Database queries
- System commands

### HTTP Transport

For remote MCP servers over HTTP:

```rust
let transport = HttpTransport::new("https://api.example.com/mcp")
    .with_auth("your-api-token");
```

**Use cases:**
- Cloud APIs
- Remote databases
- Third-party services
- Hosted MCP servers

### WebSocket Transport (Planned)

For persistent bidirectional connections:

```rust
let transport = WebSocketTransport::new("wss://api.example.com/mcp").await?;
```

## MCP Capabilities

### Tools

Tools are functions that the AI can call to perform actions.

```rust
// List available tools
let tools = client.list_tools().await?;

for tool in tools {
    println!("Tool: {}", tool.name);
    println!("  Description: {}", tool.description.unwrap_or_default());
    println!("  Schema: {}", tool.input_schema);
}

// Call a tool
let result = client.call_tool("read_file", Some(serde_json::json!({
    "path": "README.md"
}))).await?;

// Process result
for content in result.content {
    match content {
        ToolContent::Text { text } => println!("{}", text),
        ToolContent::Image { data, mime_type } => {
            println!("Image: {} bytes, type: {}", data.len(), mime_type);
        }
        _ => {}
    }
}
```

### Resources

Resources are data sources that can be read by the AI.

```rust
// List available resources
let resources = client.list_resources().await?;

for resource in resources {
    println!("Resource: {}", resource.uri);
    println!("  Name: {}", resource.name);
    println!("  Type: {}", resource.mime_type.unwrap_or_default());
}

// Read a resource
let content = client.read_resource("file:///path/to/file.txt").await?;

for item in content.contents {
    if let Some(text) = item.text {
        println!("{}", text);
    }
}
```

### Prompts

Prompts are pre-defined message templates with arguments.

```rust
// List available prompts
let prompts = client.list_prompts().await?;

for prompt in prompts {
    println!("Prompt: {}", prompt.name);
    if let Some(args) = prompt.arguments {
        for arg in args {
            println!("  - {}: {}", arg.name, arg.description.unwrap_or_default());
        }
    }
}

// Get a prompt
let prompt = client.get_prompt("code_review", Some(serde_json::json!({
    "language": "rust",
    "file": "src/main.rs"
}))).await?;

for message in prompt.messages {
    println!("{}: {:?}", message.role, message.content);
}
```

## Configuration

```rust
let config = McpClientConfig {
    client_name: "MyApp".to_string(),
    client_version: "1.0.0".to_string(),
    protocol_version: "2024-11-05".to_string(),
    timeout_ms: 30000,
};
```

## Error Handling

```rust
match client.call_tool("some_tool", None).await {
    Ok(result) => {
        // Handle success
    }
    Err(McpError::Transport(msg)) => {
        eprintln!("Transport error: {}", msg);
    }
    Err(McpError::JsonRpc { code, message }) => {
        eprintln!("Server error {}: {}", code, message);
    }
    Err(McpError::ConnectionClosed) => {
        eprintln!("Connection closed");
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Common MCP Servers

### Filesystem

```bash
npx -y @modelcontextprotocol/server-filesystem /path/to/directory
```

**Tools:**
- `read_file` - Read file contents
- `write_file` - Write to file
- `list_directory` - List directory contents
- `create_directory` - Create directory
- `move_file` - Move/rename file
- `search_files` - Search for files

### GitHub

```bash
export GITHUB_TOKEN=your_token
npx -y @modelcontextprotocol/server-github
```

**Tools:**
- `create_issue` - Create GitHub issue
- `create_pull_request` - Create PR
- `search_repositories` - Search repos
- `get_file_contents` - Read file from repo

### PostgreSQL

```bash
npx -y @modelcontextprotocol/server-postgres postgresql://user:pass@host/db
```

**Tools:**
- `query` - Execute SQL query
- `list_tables` - List database tables
- `describe_table` - Get table schema

## Best Practices

### 1. Connection Management

```rust
// Reuse client instances
let client = Arc::new(McpClient::new(transport, config));

// Clone for concurrent use
let client_clone = client.clone();
tokio::spawn(async move {
    client_clone.list_tools().await
});
```

### 2. Error Recovery

```rust
async fn call_with_retry(client: &McpClient, tool: &str, max_retries: u32) -> McpResult<ToolCallResponse> {
    let mut attempts = 0;
    loop {
        match client.call_tool(tool, None).await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 3. Timeout Handling

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(10),
    client.call_tool("slow_tool", None)
).await??;
```

### 4. Logging

```rust
use tracing::{info, debug, error};

debug!("Calling tool: {}", tool_name);
match client.call_tool(tool_name, args).await {
    Ok(result) => {
        info!("Tool call succeeded");
        Ok(result)
    }
    Err(e) => {
        error!("Tool call failed: {}", e);
        Err(e)
    }
}
```

## Security Considerations

1. **Validate Tool Arguments**: Always validate user input before passing to tools
2. **Limit Filesystem Access**: Use specific paths, not root directory
3. **Secure Credentials**: Store API tokens in environment variables or secure vaults
4. **Timeout All Requests**: Prevent hanging on unresponsive servers
5. **Audit Tool Calls**: Log all tool invocations for security review

## Troubleshooting

### "Failed to spawn process"

- Ensure the MCP server command is in PATH
- Check that Node.js is installed (for npm-based servers)
- Verify file permissions

### "Connection closed"

- Server may have crashed - check stderr output
- Timeout may be too short - increase `timeout_ms`
- Server may not support the protocol version

### "Invalid JSON-RPC response"

- Server may not be MCP-compliant
- Check server logs for errors
- Verify protocol version compatibility

## Next Steps

- See [MCP Specification](https://modelcontextprotocol.io/specification)
- Explore [Example Servers](https://github.com/modelcontextprotocol/servers)
- Read [OpenZax Architecture](./master-architecture-blueprint.md)
