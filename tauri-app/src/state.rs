use std::sync::Arc;
use tokio::sync::Mutex;
use openzax_core::{agent::AgentConfig, storage::Storage};

pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
    pub config: Arc<Mutex<AgentConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        let storage_path = dirs::home_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join(".openzax")
            .join("openzax.db");

        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let storage = Storage::new(&storage_path)
            .expect("Failed to initialize storage");

        let api_key = std::env::var("OPENZAX_API_KEY")
            .ok()
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok());

        let config = AgentConfig {
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            api_key,
            model: "deepseek/deepseek-r1-0528:free".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: None,
        };

        Self {
            storage: Arc::new(Mutex::new(storage)),
            config: Arc::new(Mutex::new(config)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
