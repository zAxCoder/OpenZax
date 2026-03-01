use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HealingError {
    #[error("Max retries exceeded after {0} attempts")]
    MaxRetriesExceeded(u32),
    #[error("Unrecoverable error: {0}")]
    Unrecoverable(String),
    #[error("Checkpoint not found: {0}")]
    CheckpointNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f32,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.base_delay_ms as f64;
        let multiplier = self.backoff_multiplier as f64;
        let exp = base * multiplier.powi(attempt as i32);
        let clamped = exp.min(self.max_delay_ms as f64);

        let ms = if self.jitter {
            // Add up to 20% jitter
            let jitter_range = clamped * 0.2;
            let jitter = (rand_u64() as f64 / u64::MAX as f64) * jitter_range;
            (clamped + jitter) as u64
        } else {
            clamped as u64
        };

        Duration::from_millis(ms)
    }
}

fn rand_u64() -> u64 {
    // Simple fast source of pseudo-randomness without external deps
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(12345)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    Transient,
    RateLimited,
    AuthFailure,
    ResourceExhausted,
    InvalidInput,
    Unrecoverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    RetryWithDelay,
    SwitchModel,
    SkipStep,
    UseCache,
    AskUser,
}

pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn classify(error_str: &str) -> ErrorClass {
        let lower = error_str.to_lowercase();

        if lower.contains("rate limit") || lower.contains("429") || lower.contains("too many requests") {
            return ErrorClass::RateLimited;
        }
        if lower.contains("unauthorized") || lower.contains("401") || lower.contains("forbidden") || lower.contains("403") {
            return ErrorClass::AuthFailure;
        }
        if lower.contains("timeout") || lower.contains("connection") || lower.contains("network") || lower.contains("503") {
            return ErrorClass::Transient;
        }
        if lower.contains("out of memory") || lower.contains("quota exceeded") || lower.contains("context length") {
            return ErrorClass::ResourceExhausted;
        }
        if lower.contains("invalid") || lower.contains("parse error") || lower.contains("bad request") || lower.contains("400") {
            return ErrorClass::InvalidInput;
        }
        if lower.contains("fatal") || lower.contains("panic") || lower.contains("unrecoverable") {
            return ErrorClass::Unrecoverable;
        }
        // Default to transient for unknown errors
        ErrorClass::Transient
    }

    pub fn suggest_fallback(class: &ErrorClass, attempt: u32) -> FallbackStrategy {
        match class {
            ErrorClass::Transient => FallbackStrategy::RetryWithDelay,
            ErrorClass::RateLimited => FallbackStrategy::RetryWithDelay,
            ErrorClass::AuthFailure => FallbackStrategy::AskUser,
            ErrorClass::ResourceExhausted => {
                if attempt < 2 {
                    FallbackStrategy::SwitchModel
                } else {
                    FallbackStrategy::AskUser
                }
            }
            ErrorClass::InvalidInput => FallbackStrategy::SkipStep,
            ErrorClass::Unrecoverable => FallbackStrategy::AskUser,
        }
    }

    pub fn is_retryable(class: &ErrorClass) -> bool {
        matches!(class, ErrorClass::Transient | ErrorClass::RateLimited)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInfo {
    pub id: String,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub state_size_bytes: usize,
    pub tool_calls_at_checkpoint: u32,
}

pub struct HealingOrchestrator {
    conn: Arc<Mutex<Connection>>,
}

impl HealingOrchestrator {
    pub fn new(db_path: &str) -> Result<Self, HealingError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                state_json TEXT NOT NULL,
                tool_calls_count INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_checkpoints_agent ON checkpoints(agent_id, created_at DESC);",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Executes `task_fn` with automatic retry and healing on failure.
    pub async fn execute_with_healing<F, Fut, T>(
        &self,
        task_fn: F,
        policy: &RetryPolicy,
    ) -> Result<T, HealingError>
    where
        F: Fn(u32) -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        let mut attempt = 0u32;

        loop {
            match task_fn(attempt).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let class = ErrorClassifier::classify(&error);
                    let fallback = ErrorClassifier::suggest_fallback(&class, attempt);

                    tracing::warn!(
                        "Attempt {}/{}: {} | Class: {:?} | Fallback: {:?}",
                        attempt + 1,
                        policy.max_retries,
                        error,
                        class,
                        fallback,
                    );

                    if class == ErrorClass::Unrecoverable {
                        return Err(HealingError::Unrecoverable(error));
                    }

                    if attempt >= policy.max_retries {
                        return Err(HealingError::MaxRetriesExceeded(attempt + 1));
                    }

                    match fallback {
                        FallbackStrategy::RetryWithDelay => {
                            let delay = policy.delay_for_attempt(attempt);
                            tracing::debug!("Retrying in {:?}", delay);
                            tokio::time::sleep(delay).await;
                        }
                        FallbackStrategy::AskUser => {
                            return Err(HealingError::Unrecoverable(format!(
                                "Requires user intervention: {}",
                                error
                            )));
                        }
                        FallbackStrategy::SkipStep => {
                            return Err(HealingError::Unrecoverable(format!(
                                "Step skipped due to: {}",
                                error
                            )));
                        }
                        FallbackStrategy::SwitchModel | FallbackStrategy::UseCache => {
                            // These require higher-level coordination; retry for now
                            let delay = policy.delay_for_attempt(attempt);
                            tokio::time::sleep(delay).await;
                        }
                    }

                    attempt += 1;
                }
            }
        }
    }

    pub fn save_checkpoint(
        &self,
        agent_id: &str,
        state: &serde_json::Value,
        tool_calls_count: u32,
    ) -> Result<CheckpointInfo, HealingError> {
        let state_json = serde_json::to_string(state)?;
        let state_size = state_json.len();
        let now = Utc::now();
        let id = format!("ckpt_{}", Uuid::new_v4().simple());

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO checkpoints (id, agent_id, created_at, state_json, tool_calls_count)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                agent_id,
                now.to_rfc3339(),
                state_json,
                tool_calls_count,
            ],
        )?;

        tracing::debug!(
            "Saved checkpoint {} for agent {} ({} bytes)",
            id,
            agent_id,
            state_size
        );

        Ok(CheckpointInfo {
            id,
            agent_id: agent_id.to_string(),
            created_at: now,
            state_size_bytes: state_size,
            tool_calls_at_checkpoint: tool_calls_count,
        })
    }

    pub fn restore_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<serde_json::Value, HealingError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT state_json FROM checkpoints WHERE id = ?1")?;
        let mut rows = stmt.query(params![checkpoint_id])?;
        let row = rows
            .next()?
            .ok_or_else(|| HealingError::CheckpointNotFound(checkpoint_id.to_string()))?;
        let state_json: String = row.get(0)?;
        let state: serde_json::Value = serde_json::from_str(&state_json)?;
        Ok(state)
    }

    pub fn list_checkpoints(&self, agent_id: &str) -> Result<Vec<CheckpointInfo>, HealingError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, created_at, LENGTH(state_json), tool_calls_count
             FROM checkpoints WHERE agent_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![agent_id], |row| {
            Ok(CheckpointInfo {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                state_size_bytes: row.get::<_, i64>(3)? as usize,
                tool_calls_at_checkpoint: row.get::<_, i64>(4)? as u32,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(HealingError::from)
    }

    pub fn prune_old_checkpoints(
        &self,
        agent_id: &str,
        keep_latest_n: usize,
    ) -> Result<usize, HealingError> {
        let all = self.list_checkpoints(agent_id)?;
        if all.len() <= keep_latest_n {
            return Ok(0);
        }
        let to_delete = &all[keep_latest_n..];
        let conn = self.conn.lock().unwrap();
        let mut deleted = 0;
        for cp in to_delete {
            deleted += conn.execute("DELETE FROM checkpoints WHERE id = ?1", params![cp.id])?;
        }
        Ok(deleted)
    }
}
