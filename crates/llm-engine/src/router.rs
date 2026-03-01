use crate::{LlmError, LlmResult, Model, ModelCapability};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub weight_latency: f64,
    pub weight_cost: f64,
    pub weight_capability: f64,
    pub weight_quality: f64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            weight_latency: 0.25,
            weight_cost: 0.20,
            weight_capability: 0.30,
            weight_quality: 0.25,
        }
    }
}

pub struct ModelRouter {
    models: Arc<RwLock<HashMap<String, Model>>>,
    config: RouterConfig,
}

impl ModelRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn register_model(&self, model: Model) {
        let model_id = model.info.id.clone();
        info!("Registering model: {}", model_id);
        self.models.write().await.insert(model_id, model);
    }

    pub async fn unregister_model(&self, model_id: &str) -> LlmResult<()> {
        self.models.write().await.remove(model_id)
            .ok_or_else(|| LlmError::ModelNotFound(model_id.to_string()))?;
        info!("Unregistered model: {}", model_id);
        Ok(())
    }

    pub async fn list_models(&self) -> Vec<Model> {
        self.models.read().await.values().cloned().collect()
    }

    pub async fn get_model(&self, model_id: &str) -> LlmResult<Model> {
        self.models.read().await.get(model_id)
            .cloned()
            .ok_or_else(|| LlmError::ModelNotFound(model_id.to_string()))
    }

    pub async fn select_best_model(
        &self,
        required_capabilities: &[ModelCapability],
        context_size: usize,
    ) -> LlmResult<Model> {
        let models = self.models.read().await;
        
        if models.is_empty() {
            return Err(LlmError::ModelNotFound("No models registered".to_string()));
        }

        let mut candidates: Vec<_> = models.values()
            .filter(|m| {
                // Check context window
                if context_size > m.info.context_window {
                    return false;
                }
                
                // Check capabilities
                required_capabilities.iter().all(|cap| m.supports_capability(cap))
            })
            .collect();

        if candidates.is_empty() {
            return Err(LlmError::ModelNotFound(
                format!("No model found matching requirements: {:?}", required_capabilities)
            ));
        }

        // Score and sort candidates
        candidates.sort_by(|a, b| {
            let score_a = self.score_model(a, context_size);
            let score_b = self.score_model(b, context_size);
            score_b.partial_cmp(&score_a).unwrap()
        });

        debug!("Selected model: {} (score: {:.3})", 
               candidates[0].info.id, 
               self.score_model(candidates[0], context_size));

        Ok(candidates[0].clone())
    }

    fn score_model(&self, model: &Model, context_size: usize) -> f64 {
        // Normalize latency (lower is better)
        let max_latency = 10000.0; // 10 seconds
        let normalized_latency = 1.0 - (model.avg_latency_ms / max_latency).min(1.0);

        // Normalize cost (lower is better)
        let normalized_cost = if let Some(cost) = model.estimate_cost(context_size, context_size) {
            let max_cost = 1.0; // $1 per request
            1.0 - (cost / max_cost).min(1.0)
        } else {
            1.0 // Free (local models)
        };

        // Capability match (already filtered, so 1.0)
        let capability_match = 1.0;

        // Quality score (0.0 - 1.0)
        let quality = model.quality_score;

        // Weighted sum
        self.config.weight_latency * normalized_latency
            + self.config.weight_cost * normalized_cost
            + self.config.weight_capability * capability_match
            + self.config.weight_quality * quality
    }

    pub async fn update_model_stats(&self, model_id: &str, latency_ms: f64) -> LlmResult<()> {
        let mut models = self.models.write().await;
        let model = models.get_mut(model_id)
            .ok_or_else(|| LlmError::ModelNotFound(model_id.to_string()))?;

        // Update rolling average (exponential moving average)
        let alpha = 0.1;
        model.avg_latency_ms = alpha * latency_ms + (1.0 - alpha) * model.avg_latency_ms;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelInfo, ModelProvider};

    #[tokio::test]
    async fn test_model_registration() {
        let router = ModelRouter::new(RouterConfig::default());
        
        let model = Model::new(ModelInfo {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            provider: ModelProvider::Local,
            context_window: 4096,
            capabilities: vec![ModelCapability::Chat],
            size_bytes: None,
            quantization: None,
            is_local: true,
            path: None,
        });

        router.register_model(model.clone()).await;
        
        let retrieved = router.get_model("test-model").await.unwrap();
        assert_eq!(retrieved.info.id, "test-model");
    }

    #[tokio::test]
    async fn test_model_selection() {
        let router = ModelRouter::new(RouterConfig::default());
        
        let model1 = Model::new(ModelInfo {
            id: "fast-model".to_string(),
            name: "Fast Model".to_string(),
            provider: ModelProvider::Local,
            context_window: 2048,
            capabilities: vec![ModelCapability::Chat],
            size_bytes: None,
            quantization: None,
            is_local: true,
            path: None,
        });

        let model2 = Model::new(ModelInfo {
            id: "large-model".to_string(),
            name: "Large Model".to_string(),
            provider: ModelProvider::Local,
            context_window: 8192,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Code],
            size_bytes: None,
            quantization: None,
            is_local: true,
            path: None,
        });

        router.register_model(model1).await;
        router.register_model(model2).await;

        let selected = router.select_best_model(
            &[ModelCapability::Chat],
            1000
        ).await.unwrap();

        assert!(selected.info.context_window >= 1000);
    }
}
