use tauri::State;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::ChatMessage;
use openzax_core::agent::Agent;
use openzax_core::event::EventBus;

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

    let cfg = state.config.lock().await.clone();
    let eb = EventBus::default();
    let agent = Agent::new(cfg, eb);

    let response_content = agent
        .process(&request.content)
        .await
        .map_err(|e| e.to_string())?;

    let response_message = ChatMessage {
        role: "assistant".to_string(),
        content: response_content,
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
    _conversation_id: String,
    _state: State<'_, AppState>,
) -> Result<ConversationHistory, String> {
    Ok(ConversationHistory {
        messages: Vec::new(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}

#[tauri::command]
pub async fn list_models(
    _state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    Ok(vec![
        ModelInfo { id: "deepseek/deepseek-r1-0528:free".into(), name: "DeepSeek R1".into(), provider: "OpenRouter".into() },
        ModelInfo { id: "meta-llama/llama-3.3-70b-instruct:free".into(), name: "Llama 3.3 70B".into(), provider: "OpenRouter".into() },
        ModelInfo { id: "qwen/qwen3-235b-a22b:free".into(), name: "Qwen3 235B".into(), provider: "OpenRouter".into() },
        ModelInfo { id: "google/gemma-3-27b-it:free".into(), name: "Gemma 3 27B".into(), provider: "OpenRouter".into() },
    ])
}

#[tauri::command]
pub async fn get_model_info(
    model_id: String,
    _state: State<'_, AppState>,
) -> Result<ModelInfo, String> {
    Ok(ModelInfo {
        id: model_id.clone(),
        name: model_id.split('/').last().unwrap_or(&model_id).to_string(),
        provider: "OpenRouter".into(),
    })
}

#[tauri::command]
pub async fn list_mcp_servers(
    _state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
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
    _state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    tracing::info!("Executing command: {} {:?}", request.command, request.args);

    Ok(CommandResponse {
        success: true,
        output: format!("Command '{}' executed", request.command),
    })
}
