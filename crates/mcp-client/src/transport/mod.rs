pub mod http;
pub mod stdio;

pub use http::HttpTransport;
pub use stdio::StdioTransport;

use crate::{
    protocol::{JsonRpcRequest, JsonRpcResponse},
    McpResult,
};
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
