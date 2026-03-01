use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod state;

use state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize application state
            let state = AppState::new();
            app.manage(state);
            
            tracing::info!("OpenZax Tauri application initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::send_message,
            commands::get_conversation_history,
            commands::list_models,
            commands::get_model_info,
            commands::list_mcp_servers,
            commands::execute_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
