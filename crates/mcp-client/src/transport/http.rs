use super::Transport;
use crate::{
    protocol::{JsonRpcRequest, JsonRpcResponse},
    McpError, McpResult,
};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, info};

/// HTTP transport for remote MCP servers
pub struct HttpTransport {
    client: Client,
    url: String,
    headers: HashMap<String, String>,
    connected: bool,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            url: url.into(),
            headers: HashMap::new(),
            connected: true,
        }
    }

    /// Add a header to all requests
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set authorization header
    pub fn with_auth(self, token: impl Into<String>) -> Self {
        self.with_header("Authorization", format!("Bearer {}", token.into()))
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if !self.connected {
            return Err(McpError::ConnectionClosed);
        }

        debug!("Sending HTTP request to {}: {:?}", self.url, request);

        let mut req = self.client.post(&self.url).json(&request);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(McpError::Transport(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json: JsonRpcResponse = response.json().await?;

        if let Some(error) = response_json.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }

        Ok(response_json)
    }

    async fn notify(&mut self, request: JsonRpcRequest) -> McpResult<()> {
        if !self.connected {
            return Err(McpError::ConnectionClosed);
        }

        debug!("Sending HTTP notification to {}: {:?}", self.url, request);

        let mut req = self.client.post(&self.url).json(&request);

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(McpError::Transport(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn close(&mut self) -> McpResult<()> {
        info!("Closing HTTP transport");
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
