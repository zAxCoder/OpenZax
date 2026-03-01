pub mod stdio;
pub mod http;

pub use stdio::StdioTransport;
pub use http::HttpTransport;

use crate::{McpResult, protocol::{JsonRpcRequest, JsonRpcResponse}};
use async_trait::async_trait;

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a request and wait for response
    async fn send(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse>;
    
    /// Send a notification (no response expected)
    async fn notify(&mut self, request: JsonRpcRequest) -> McpResult<()>;
    
    /// Close the transport
    async fn close(&mut self) -> McpResult<()>;
    
    /// Check if transport is connected
    fn is_connected(&self) -> bool;
}
