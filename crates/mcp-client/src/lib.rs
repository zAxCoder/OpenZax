pub mod protocol;
pub mod transport;
pub mod client;
pub mod error;

pub use client::{McpClient, McpClientConfig};
pub use error::{McpError, McpResult};
pub use protocol::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    Tool, Resource, Prompt, SamplingRequest,
};
pub use transport::{Transport, StdioTransport, HttpTransport};
