# OpenZax MCP Client

Native Rust implementation of the Model Context Protocol (MCP) client.

## Features

- **Multiple Transports**: stdio, HTTP, WebSocket (planned)
- **Full MCP Support**: Tools, Resources, Prompts, Sampling
- **Async/Await**: Built on Tokio
- **Type-Safe**: Complete protocol types
- **Connection Management**: Auto-reconnect, health checks

## Usage

### Stdio Transport (Local Servers)

```rust
use openzax_mcp_client::{McpClient, McpClientConfig};
use openzax_mcp_client::transport::StdioTransport;

// Spawn local MCP server
let transport = StdioTransport::new("npx", &[
    "-y".to_string(),
    "@modelcontextprotocol/server-filesystem".to_string(),
    "/path/to/allowed/directory".to_string(),
]).await?;

// Create client
let config = McpClientConfig::default();
let client = McpClient::new(Box::new(transport), config);

// Initialize connection
let init_response = client.initialize().await?;
println!("Connected to: {} v{}", 
         init_response.server_info.name,
         init_response.server_info.version);

// List available tools
let tools = client.list_tools().await?;
for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.description.unwrap_or_default());
}

// Call a tool
let result = client.call_tool("read_file", Some(serde_json::json!({
    "path": "README.md"
}))).await?;
```

### HTTP Transport (Remote Servers)

```rust
use openzax_mcp_client::transport::HttpTransport;

let transport = HttpTransport::new("https://api.example.com/mcp")
    .with_auth("your-api-token");

let client = McpClient::new(Box::new(transport), config);
```

## Protocol Support

### Tools
- `tools/list` - List available tools
- `tools/call` - Invoke a tool

### Resources
- `resources/list` - List available resources
- `resources/read` - Read resource content
- `resources/subscribe` - Subscribe to resource changes

### Prompts
- `prompts/list` - List available prompts
- `prompts/get` - Get prompt with arguments

### Sampling
- `sampling/createMessage` - Request model completion

## Architecture

```
┌─────────────────────────────────────────┐
│         McpClient                       │
│  - Connection management                │
│  - Request/response handling            │
│  - Protocol implementation              │
├─────────────────────────────────────────┤
│         Transport Layer                 │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │  Stdio   │  │   HTTP   │  │WebSocket││
│  │Transport │  │Transport │  │Transport││
│  └──────────┘  └──────────┘  └────────┘│
└─────────────────────────────────────────┘
```

## Examples

See `examples/` directory for complete examples:
- `mcp-filesystem.rs` - Filesystem MCP server
- `mcp-github.rs` - GitHub MCP server
- `mcp-custom.rs` - Custom MCP server

## Testing

```bash
cargo test --package openzax-mcp-client
```
