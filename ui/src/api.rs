use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message: ChatMessage,
    pub conversation_id: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn send_message(content: String) -> Result<SendMessageResponse, String> {
    let request = SendMessageRequest { content };
    let args = serde_wasm_bindgen::to_value(&request)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    let result = invoke("send_message", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Deserialization error: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
}

pub async fn list_models() -> Result<Vec<ModelInfo>, String> {
    let result = invoke("list_models", JsValue::NULL).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Deserialization error: {}", e))
}

pub async fn get_model_info(model_id: String) -> Result<ModelInfo, String> {
    let args = serde_wasm_bindgen::to_value(&model_id)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    let result = invoke("get_model_info", args).await;
    
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Deserialization error: {}", e))
}
