use openzax_mcp_client::{McpClient, McpClientConfig};
use openzax_mcp_client::transport::StdioTransport;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    println!("OpenZax MCP Filesystem Example\n");

    // Spawn filesystem MCP server
    println!("Starting filesystem MCP server...");
    let transport = StdioTransport::new("npx", &[
        "-y".to_string(),
        "@modelcontextprotocol/server-filesystem".to_string(),
        ".".to_string(), // Allow current directory
    ]).await?;

    // Create MCP client
    let config = McpClientConfig::default();
    let client = McpClient::new(Box::new(transport), config);

    // Initialize connection
    println!("Initializing MCP connection...");
    let init_response = client.initialize().await?;
    println!("✓ Connected to: {} v{}\n", 
             init_response.server_info.name,
             init_response.server_info.version);

    // List available tools
    println!("Available tools:");
    let tools = client.list_tools().await?;
    for tool in &tools {
        println!("  - {}: {}", 
                 tool.name, 
                 tool.description.as_ref().unwrap_or(&"No description".to_string()));
    }
    println!();

    // Read a file
    if tools.iter().any(|t| t.name == "read_file") {
        println!("Reading README.md...");
        match client.call_tool("read_file", Some(serde_json::json!({
            "path": "README.md"
        }))).await {
            Ok(result) => {
                println!("✓ File read successfully");
                for content in result.content {
                    match content {
                        openzax_mcp_client::protocol::ToolContent::Text { text } => {
                            println!("Content preview: {}...", 
                                     text.chars().take(200).collect::<String>());
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Error reading file: {}", e);
            }
        }
    }

    println!("\nExample completed!");
    Ok(())
}
