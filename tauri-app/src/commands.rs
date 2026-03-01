use tauri::State;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::ChatMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message: ChatMessage,
    pub conversation_id: String,
}

#[tauri::command]
pub async fn send_message(
    request: SendMessageRequest,
    state: State<'_, AppState>,
) -> Result<SendMessageResponse, String> {
    tracing::info!("Received message: {}", request.content);
    
    // TODO: Implement actual agent processing
    // For now, return a simple echo response
    
    let response_message = ChatMessage {
        role: "assistant".to_string(),
        content: format!("Echo: {}", request.content),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    Ok(SendMessageResponse {
        message: response_message,
        conversation_id: "default".to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub messages: Vec<ChatMessage>,
}

#[tauri::command]
pub async fn get_conversation_history(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<ConversationHistory, String> {
    tracing::info!("Getting conversation history for: {}", conversation_id);
    
    // TODO: Implement actual storage retrieval
    Ok(ConversationHistory {
        messages: Vec::new(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
}

#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    tracing::info!("Listing available models");
    
    let local_models = state.local_models.lock().await;
    let models = local_models.discover_models()
        .map_err(|e| e.to_string())?;
    
    let model_infos: Vec<ModelInfo> = models.into_iter().map(|m| ModelInfo {
        id: m.id,
        name: m.name,
        provider: format!("{:?}", m.provider),
        context_window: m.context_window,
    }).collect();
    
    Ok(model_infos)
}

#[tauri::command]
pub async fn get_model_info(
    model_id: String,
    state: State<'_, AppState>,
) -> Result<ModelInfo, String> {
    tracing::info!("Getting model info for: {}", model_id);
    
    let local_models = state.local_models.lock().await;
    let models = local_models.discover_models()
        .map_err(|e| e.to_string())?;
    
    models.into_iter()
        .find(|m| m.id == model_id)
        .map(|m| ModelInfo {
            id: m.id,
            name: m.name,
            provider: format!("{:?}", m.provider),
            context_window: m.context_window,
        })
        .ok_or_else(|| format!("Model not found: {}", model_id))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub status: String,
    pub tools_count: usize,
}

#[tauri::command]
pub async fn list_mcp_servers(
    state: State<'_, AppState>,
) -> Result<Vec<McpServerInfo>, String> {
    tracing::info!("Listing MCP servers");
    
    // TODO: Implement actual MCP server listing
    Ok(Vec::new())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub output: String,
}

#[tauri::command]
pub async fn execute_command(
    request: CommandRequest,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    tracing::info!("Executing command: {} {:?}", request.command, request.args);
    
    // TODO: Implement command execution through command palette
    Ok(CommandResponse {
        success: true,
        output: format!("Command '{}' executed", request.command),
    })
}
