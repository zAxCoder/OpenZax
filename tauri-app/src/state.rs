use std::sync::Arc;
use tokio::sync::Mutex;
use openzax_core::{agent::AgentConfig, storage::Storage};
use openzax_llm_engine::{ModelRouter, LocalModelManager};
use openzax_mcp_client::McpClient;

pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
    pub model_router: Arc<Mutex<Option<ModelRouter>>>,
    pub local_models: Arc<Mutex<LocalModelManager>>,
    pub mcp_clients: Arc<Mutex<Vec<McpClient>>>,
    pub config: Arc<Mutex<AgentConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        // Initialize storage
        let storage_path = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("openzax")
            .join("openzax.db");
        
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let storage = Storage::new(&storage_path)
            .expect("Failed to initialize storage");
        
        // Initialize local model manager
        let models_dir = dirs::home_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join(".openzax")
            .join("models");
        
        std::fs::create_dir_all(&models_dir).ok();
        let local_models = LocalModelManager::new(models_dir);
        
        // Initialize default config
        let config = AgentConfig {
            api_key: std::env::var("OPENZAX_API_KEY").ok(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            stream: true,
        };
        
        Self {
            storage: Arc::new(Mutex::new(storage)),
            model_router: Arc::new(Mutex::new(None)),
            local_models: Arc::new(Mutex::new(local_models)),
            mcp_clients: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(config)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
