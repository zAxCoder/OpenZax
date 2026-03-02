use crate::{protocol::*, transport::Transport, McpError, McpResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct McpClientConfig {
    pub client_name: String,
    pub client_version: String,
    pub protocol_version: String,
    pub timeout_ms: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            client_name: "OpenZax".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: "2024-11-05".to_string(),
            timeout_ms: 30000,
        }
    }
}

pub struct McpClient {
    transport: Arc<Mutex<Box<dyn Transport>>>,
    config: McpClientConfig,
    request_id: AtomicU64,
    server_info: Arc<Mutex<Option<ServerInfo>>>,
    server_capabilities: Arc<Mutex<Option<ServerCapabilities>>>,
}

impl McpClient {
    pub fn new(transport: Box<dyn Transport>, config: McpClientConfig) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
            config,
            request_id: AtomicU64::new(1),
            server_info: Arc::new(Mutex::new(None)),
            server_capabilities: Arc::new(Mutex::new(None)),
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn initialize(&self) -> McpResult<InitializeResponse> {
        info!("Initializing MCP client");

        let request = InitializeRequest {
            protocol_version: self.config.protocol_version.clone(),
            capabilities: ClientCapabilities {
                roots: Some(RootsCapability { list_changed: true }),
                sampling: None,
            },
            client_info: ClientInfo {
                name: self.config.client_name.clone(),
                version: self.config.client_version.clone(),
            },
        };

        let params = serde_json::to_value(request)?;
        let rpc_request = JsonRpcRequest::new(self.next_id(), "initialize", Some(params));

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let init_response: InitializeResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result in response".to_string()))?,
        )?;

        *self.server_info.lock().await = Some(init_response.server_info.clone());
        *self.server_capabilities.lock().await = Some(init_response.capabilities.clone());

        info!(
            "MCP client initialized: server={} v{}",
            init_response.server_info.name, init_response.server_info.version
        );

        Ok(init_response)
    }

    pub async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        debug!("Listing tools");

        let rpc_request = JsonRpcRequest::new(self.next_id(), "tools/list", None);

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let tools_response: ToolsListResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        debug!("Found {} tools", tools_response.tools.len());
        Ok(tools_response.tools)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<ToolCallResponse> {
        debug!("Calling tool: {}", name);

        let request = ToolCallRequest {
            name: name.to_string(),
            arguments,
        };

        let params = serde_json::to_value(request)?;
        let rpc_request = JsonRpcRequest::new(self.next_id(), "tools/call", Some(params));

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let tool_response: ToolCallResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        Ok(tool_response)
    }

    pub async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        debug!("Listing resources");

        let rpc_request = JsonRpcRequest::new(self.next_id(), "resources/list", None);

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let resources_response: ResourcesListResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        debug!("Found {} resources", resources_response.resources.len());
        Ok(resources_response.resources)
    }

    pub async fn read_resource(&self, uri: &str) -> McpResult<ResourceReadResponse> {
        debug!("Reading resource: {}", uri);

        let request = ResourceReadRequest {
            uri: uri.to_string(),
        };

        let params = serde_json::to_value(request)?;
        let rpc_request = JsonRpcRequest::new(self.next_id(), "resources/read", Some(params));

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let resource_response: ResourceReadResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        Ok(resource_response)
    }

    pub async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        debug!("Listing prompts");

        let rpc_request = JsonRpcRequest::new(self.next_id(), "prompts/list", None);

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let prompts_response: PromptsListResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        debug!("Found {} prompts", prompts_response.prompts.len());
        Ok(prompts_response.prompts)
    }

    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> McpResult<PromptGetResponse> {
        debug!("Getting prompt: {}", name);

        let request = PromptGetRequest {
            name: name.to_string(),
            arguments,
        };

        let params = serde_json::to_value(request)?;
        let rpc_request = JsonRpcRequest::new(self.next_id(), "prompts/get", Some(params));

        let mut transport = self.transport.lock().await;
        let response = transport.send(rpc_request).await?;

        let prompt_response: PromptGetResponse = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("No result".to_string()))?,
        )?;

        Ok(prompt_response)
    }

    pub async fn server_info(&self) -> Option<ServerInfo> {
        self.server_info.lock().await.clone()
    }

    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.lock().await.clone()
    }

    pub async fn close(&self) -> McpResult<()> {
        info!("Closing MCP client");
        let mut transport = self.transport.lock().await;
        transport.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = McpClientConfig::default();
        assert_eq!(config.client_name, "OpenZax");
        assert_eq!(config.protocol_version, "2024-11-05");
        assert_eq!(config.timeout_ms, 30000);
    }
}
