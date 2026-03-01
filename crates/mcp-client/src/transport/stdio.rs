use crate::{McpError, McpResult, protocol::{JsonRpcRequest, JsonRpcResponse}};
use super::Transport;
use async_trait::async_trait;
use tokio::process::{Child, Command, ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

/// Stdio transport for local MCP servers
pub struct StdioTransport {
    process: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    connected: Arc<Mutex<bool>>,
}

impl StdioTransport {
    /// Create a new stdio transport by spawning a command
    pub async fn new(command: &str, args: &[String]) -> McpResult<Self> {
        info!("Spawning MCP server: {} {:?}", command, args);
        
        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| McpError::Transport(format!("Failed to spawn process: {}", e)))?;
        
        let stdin = child.stdin.take()
            .ok_or_else(|| McpError::Transport("Failed to get stdin".to_string()))?;
        
        let stdout = child.stdout.take()
            .ok_or_else(|| McpError::Transport("Failed to get stdout".to_string()))?;
        
        Ok(Self {
            process: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            connected: Arc::new(Mutex::new(true)),
        })
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if !self.is_connected() {
            return Err(McpError::ConnectionClosed);
        }
        
        let request_json = serde_json::to_string(&request)?;
        debug!("Sending request: {}", request_json);
        
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        drop(stdin);
        
        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();
        let bytes_read = stdout.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            *self.connected.lock().await = false;
            return Err(McpError::ConnectionClosed);
        }
        
        debug!("Received response: {}", line.trim());
        
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| McpError::Protocol(format!("Invalid JSON-RPC response: {}", e)))?;
        
        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
            });
        }
        
        Ok(response)
    }
    
    async fn notify(&mut self, request: JsonRpcRequest) -> McpResult<()> {
        if !self.is_connected() {
            return Err(McpError::ConnectionClosed);
        }
        
        let request_json = serde_json::to_string(&request)?;
        debug!("Sending notification: {}", request_json);
        
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        
        Ok(())
    }
    
    async fn close(&mut self) -> McpResult<()> {
        info!("Closing stdio transport");
        *self.connected.lock().await = false;
        
        let mut process = self.process.lock().await;
        process.kill().await
            .map_err(|e| McpError::Transport(format!("Failed to kill process: {}", e)))?;
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        // Non-blocking check
        match self.connected.try_lock() {
            Ok(guard) => *guard,
            Err(_) => true, // Assume connected if lock is held
        }
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Ok(mut process) = self.process.try_lock() {
            let _ = process.start_kill();
        }
    }
}
