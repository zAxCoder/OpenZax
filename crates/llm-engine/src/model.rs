use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: ModelProvider,
    pub context_window: usize,
    pub capabilities: Vec<ModelCapability>,
    pub size_bytes: Option<u64>,
    pub quantization: Option<String>,
    pub is_local: bool,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelProvider {
    Local,
    OpenAI,
    Anthropic,
    Google,
    Mistral,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelCapability {
    Chat,
    Code,
    Reasoning,
    Vision,
    ToolUse,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub info: ModelInfo,
    pub avg_latency_ms: f64,
    pub quality_score: f64,
    pub input_cost_per_1m: Option<f64>,
    pub output_cost_per_1m: Option<f64>,
}

impl Model {
    pub fn new(info: ModelInfo) -> Self {
        Self {
            info,
            avg_latency_ms: 0.0,
            quality_score: 0.5,
            input_cost_per_1m: None,
            output_cost_per_1m: None,
        }
    }

    pub fn supports_capability(&self, capability: &ModelCapability) -> bool {
        self.info.capabilities.contains(capability)
    }

    pub fn estimate_cost(&self, input_tokens: usize, output_tokens: usize) -> Option<f64> {
        let input_cost = self.input_cost_per_1m? * (input_tokens as f64 / 1_000_000.0);
        let output_cost = self.output_cost_per_1m? * (output_tokens as f64 / 1_000_000.0);
        Some(input_cost + output_cost)
    }
}
