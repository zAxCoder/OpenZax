use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("No capable model found for request")]
    NoModelFound,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Google,
    Local,
    Cohere,
    Mistral,
    Custom { name: String, base_url: String },
}

impl ModelProvider {
    pub fn as_str(&self) -> String {
        match self {
            ModelProvider::OpenAI => "openai".to_string(),
            ModelProvider::Anthropic => "anthropic".to_string(),
            ModelProvider::Google => "google".to_string(),
            ModelProvider::Local => "local".to_string(),
            ModelProvider::Cohere => "cohere".to_string(),
            ModelProvider::Mistral => "mistral".to_string(),
            ModelProvider::Custom { name, .. } => format!("custom:{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Reasoning,
    Coding,
    Math,
    Creative,
    Summarization,
    Embeddings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: String,
    pub provider: ModelProvider,
    pub model_name: String,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub cost_per_1k_input: f32,
    pub cost_per_1k_output: f32,
    pub avg_latency_ms: u32,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub task_description: String,
    pub required_capabilities: Vec<Capability>,
    pub max_cost_per_1k: Option<f32>,
    pub max_latency_ms: Option<u32>,
    pub prefer_local: bool,
    pub context_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ModelScore {
    pub model: ModelSpec,
    pub score: f32,
    pub reasoning: String,
}

pub struct ModelRegistry {
    conn: Arc<Mutex<Connection>>,
}

impl ModelRegistry {
    pub fn new(db_path: &str) -> Result<Self, RouterError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                model_name TEXT NOT NULL,
                context_window INTEGER NOT NULL,
                max_output_tokens INTEGER NOT NULL,
                cost_per_1k_input REAL NOT NULL,
                cost_per_1k_output REAL NOT NULL,
                avg_latency_ms INTEGER NOT NULL,
                supports_tools INTEGER NOT NULL DEFAULT 0,
                supports_vision INTEGER NOT NULL DEFAULT 0,
                capabilities TEXT NOT NULL DEFAULT '[]',
                provider_json TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS latency_stats (
                model_id TEXT NOT NULL,
                ema_latency_ms REAL NOT NULL,
                sample_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (model_id)
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn store(&self, spec: &ModelSpec) -> Result<(), RouterError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO models
             (id, provider, model_name, context_window, max_output_tokens, cost_per_1k_input,
              cost_per_1k_output, avg_latency_ms, supports_tools, supports_vision, capabilities, provider_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                spec.id,
                spec.provider.as_str(),
                spec.model_name,
                spec.context_window,
                spec.max_output_tokens,
                spec.cost_per_1k_input,
                spec.cost_per_1k_output,
                spec.avg_latency_ms,
                spec.supports_tools as i32,
                spec.supports_vision as i32,
                serde_json::to_string(&spec.capabilities)?,
                serde_json::to_string(&spec.provider)?,
            ],
        )?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<(), RouterError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM models WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<ModelSpec, RouterError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, provider_json, model_name, context_window, max_output_tokens,
                    cost_per_1k_input, cost_per_1k_output, avg_latency_ms, supports_tools,
                    supports_vision, capabilities
             FROM models WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        let row = rows.next()?.ok_or_else(|| RouterError::ModelNotFound(id.to_string()))?;
        Ok(Self::row_to_spec(row)?)
    }

    pub fn list_all(&self) -> Result<Vec<ModelSpec>, RouterError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, provider_json, model_name, context_window, max_output_tokens,
                    cost_per_1k_input, cost_per_1k_output, avg_latency_ms, supports_tools,
                    supports_vision, capabilities
             FROM models",
        )?;
        let rows = stmt.query_map([], Self::row_to_spec)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| RouterError::Database(e))
    }

    pub fn update_latency_ema(&self, model_id: &str, measured_ms: u32) -> Result<(), RouterError> {
        let conn = self.conn.lock().unwrap();
        let existing: Option<(f64, i64)> = conn
            .query_row(
                "SELECT ema_latency_ms, sample_count FROM latency_stats WHERE model_id = ?1",
                params![model_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        let (new_ema, new_count) = if let Some((old_ema, count)) = existing {
            let alpha = 0.2f64;
            (
                alpha * measured_ms as f64 + (1.0 - alpha) * old_ema,
                count + 1,
            )
        } else {
            (measured_ms as f64, 1i64)
        };

        conn.execute(
            "INSERT OR REPLACE INTO latency_stats (model_id, ema_latency_ms, sample_count) VALUES (?1, ?2, ?3)",
            params![model_id, new_ema, new_count],
        )?;

        // Also update the main model table with the new EMA
        conn.execute(
            "UPDATE models SET avg_latency_ms = ?1 WHERE id = ?2",
            params![new_ema as i64, model_id],
        )?;
        Ok(())
    }

    fn row_to_spec(row: &rusqlite::Row) -> rusqlite::Result<ModelSpec> {
        let provider: ModelProvider =
            serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or(ModelProvider::Local);
        let capabilities: Vec<Capability> =
            serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default();
        Ok(ModelSpec {
            id: row.get(0)?,
            provider,
            model_name: row.get(2)?,
            context_window: row.get::<_, i64>(3)? as u32,
            max_output_tokens: row.get::<_, i64>(4)? as u32,
            cost_per_1k_input: row.get::<_, f64>(5)? as f32,
            cost_per_1k_output: row.get::<_, f64>(6)? as f32,
            avg_latency_ms: row.get::<_, i64>(7)? as u32,
            supports_tools: row.get::<_, i64>(8)? != 0,
            supports_vision: row.get::<_, i64>(9)? != 0,
            capabilities,
        })
    }
}

pub struct ModelRouter {
    registry: ModelRegistry,
    /// availability: model_id -> bool (can be toggled for circuit-breaking)
    availability: Arc<Mutex<HashMap<String, bool>>>,
}

impl ModelRouter {
    pub fn new(db_path: &str) -> Result<Self, RouterError> {
        Ok(Self {
            registry: ModelRegistry::new(db_path)?,
            availability: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn register_model(&self, spec: ModelSpec) -> Result<(), RouterError> {
        self.availability
            .lock()
            .unwrap()
            .insert(spec.id.clone(), true);
        self.registry.store(&spec)
    }

    pub fn deregister_model(&self, id: &str) -> Result<(), RouterError> {
        self.availability.lock().unwrap().remove(id);
        self.registry.remove(id)
    }

    pub fn set_availability(&self, model_id: &str, available: bool) {
        self.availability
            .lock()
            .unwrap()
            .insert(model_id.to_string(), available);
    }

    pub fn route(&self, request: &RoutingRequest) -> Result<ModelSpec, RouterError> {
        let scored = self.score_all(request)?;
        scored
            .into_iter()
            .next()
            .map(|s| s.model)
            .ok_or(RouterError::NoModelFound)
    }

    pub fn fallback_chain(&self, request: &RoutingRequest) -> Result<Vec<ModelSpec>, RouterError> {
        Ok(self
            .score_all(request)?
            .into_iter()
            .map(|s| s.model)
            .collect())
    }

    pub fn update_latency_stats(&self, model_id: &str, measured_ms: u32) -> Result<(), RouterError> {
        self.registry.update_latency_ema(model_id, measured_ms)
    }

    fn score_all(&self, request: &RoutingRequest) -> Result<Vec<ModelScore>, RouterError> {
        let models = self.registry.list_all()?;
        let availability = self.availability.lock().unwrap();

        let mut scored: Vec<ModelScore> = models
            .into_iter()
            .filter(|m| *availability.get(&m.id).unwrap_or(&true))
            .filter(|m| m.context_window >= request.context_tokens)
            .filter(|m| {
                // All required capabilities must be present
                request
                    .required_capabilities
                    .iter()
                    .all(|cap| m.capabilities.contains(cap))
            })
            .filter(|m| {
                request
                    .max_cost_per_1k
                    .map(|max| m.cost_per_1k_input <= max)
                    .unwrap_or(true)
            })
            .filter(|m| {
                request
                    .max_latency_ms
                    .map(|max| m.avg_latency_ms <= max)
                    .unwrap_or(true)
            })
            .map(|m| {
                let score = self.score_model(&m, request);
                let reasoning = self.build_reasoning(&m, request, score);
                ModelScore {
                    model: m,
                    score,
                    reasoning,
                }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored)
    }

    pub fn score_model(&self, model: &ModelSpec, request: &RoutingRequest) -> f32 {
        // 40% capability match, 30% latency, 20% cost, 10% availability/local preference
        let capability_score = {
            if request.required_capabilities.is_empty() {
                1.0
            } else {
                let matched = request
                    .required_capabilities
                    .iter()
                    .filter(|cap| model.capabilities.contains(cap))
                    .count() as f32;
                matched / request.required_capabilities.len() as f32
            }
        };

        // Normalize latency: lower is better. Use 1 - (latency / 10000ms) clamped to [0,1]
        let latency_score = {
            let max_ms = request.max_latency_ms.unwrap_or(10_000) as f32;
            (1.0 - (model.avg_latency_ms as f32 / max_ms)).max(0.0).min(1.0)
        };

        // Normalize cost: lower is better. Use 1 - (cost / max_cost) clamped to [0,1]
        let cost_score = {
            let max_cost = request.max_cost_per_1k.unwrap_or(10.0);
            (1.0 - (model.cost_per_1k_input / max_cost)).max(0.0).min(1.0)
        };

        // Availability/local preference bonus
        let local_score = if request.prefer_local && model.provider == ModelProvider::Local {
            1.0
        } else if !request.prefer_local {
            0.5
        } else {
            0.0
        };

        0.40 * capability_score + 0.30 * latency_score + 0.20 * cost_score + 0.10 * local_score
    }

    fn build_reasoning(&self, model: &ModelSpec, request: &RoutingRequest, score: f32) -> String {
        let matched_caps: Vec<String> = request
            .required_capabilities
            .iter()
            .filter(|cap| model.capabilities.contains(cap))
            .map(|c| format!("{:?}", c))
            .collect();
        format!(
            "Score {:.3}: model='{}' provider='{}' latency={}ms cost=${:.4}/1k capabilities=[{}]",
            score,
            model.model_name,
            model.provider.as_str(),
            model.avg_latency_ms,
            model.cost_per_1k_input,
            matched_caps.join(", ")
        )
    }
}
