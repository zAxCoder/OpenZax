use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DeterministicError {
    #[error("Recording not found for tool call: {0}")]
    NotFound(String),
    #[error("Replay mismatch detected")]
    Mismatch,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicConfig {
    pub seed: u64,
    pub record_to_file: Option<PathBuf>,
    pub replay_from_file: Option<PathBuf>,
}

impl DeterministicConfig {
    pub fn recording(seed: u64, path: impl Into<PathBuf>) -> Self {
        Self {
            seed,
            record_to_file: Some(path.into()),
            replay_from_file: None,
        }
    }

    pub fn replaying(seed: u64, path: impl Into<PathBuf>) -> Self {
        Self {
            seed,
            record_to_file: None,
            replay_from_file: Some(path.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub call_id: Uuid,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallRecord {
    pub call_id: Uuid,
    pub prompt_hash: String,
    pub seed: u64,
    pub response: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RecordEntry {
    ToolCall(ToolCallRecord),
    LlmCall(LlmCallRecord),
}

pub struct ExecutionRecorder {
    file: Arc<Mutex<File>>,
}

impl ExecutionRecorder {
    pub fn new(path: &Path) -> Result<Self, DeterministicError> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    pub fn record_tool_call(&self, record: &ToolCallRecord) -> Result<(), DeterministicError> {
        let entry = RecordEntry::ToolCall(record.clone());
        let line = serde_json::to_string(&entry)?;
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn record_llm_call(
        &self,
        prompt_hash: &str,
        seed: u64,
        response: &str,
    ) -> Result<(), DeterministicError> {
        let record = LlmCallRecord {
            call_id: Uuid::new_v4(),
            prompt_hash: prompt_hash.to_string(),
            seed,
            response: response.to_string(),
            timestamp: Utc::now(),
        };
        let entry = RecordEntry::LlmCall(record);
        let line = serde_json::to_string(&entry)?;
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", line)?;
        Ok(())
    }

    pub fn flush(&self) -> Result<(), DeterministicError> {
        let mut file = self.file.lock().unwrap();
        file.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub tool_name: String,
    pub field: String,
    pub recorded: serde_json::Value,
    pub actual: serde_json::Value,
}

pub struct ExecutionReplayer {
    /// Index: (tool_name, param_hash) -> ToolCallRecord
    tool_index: HashMap<(String, String), Vec<ToolCallRecord>>,
    /// Sequential cursor per tool for ordered replay
    cursors: Mutex<HashMap<String, usize>>,
    llm_index: HashMap<String, LlmCallRecord>,
}

impl ExecutionReplayer {
    pub fn load_recording(path: &Path) -> Result<Self, DeterministicError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut tool_index: HashMap<(String, String), Vec<ToolCallRecord>> = HashMap::new();
        let mut llm_index: HashMap<String, LlmCallRecord> = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: RecordEntry = serde_json::from_str(&line)?;
            match entry {
                RecordEntry::ToolCall(record) => {
                    let param_hash = Self::hash_params(&record.parameters);
                    tool_index
                        .entry((record.tool_name.clone(), param_hash))
                        .or_default()
                        .push(record);
                }
                RecordEntry::LlmCall(record) => {
                    llm_index.insert(record.prompt_hash.clone(), record);
                }
            }
        }

        Ok(Self {
            tool_index,
            cursors: Mutex::new(HashMap::new()),
            llm_index,
        })
    }

    pub fn get_all_tool_calls(&self) -> Vec<ToolCallRecord> {
        self.tool_index.values().flatten().cloned().collect()
    }

    pub fn replay_tool_call(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, DeterministicError> {
        let param_hash = Self::hash_params(params);
        let key = (tool_name.to_string(), param_hash);
        let records = self
            .tool_index
            .get(&key)
            .ok_or_else(|| DeterministicError::NotFound(format!("{}({:?})", tool_name, params)))?;

        let mut cursors = self.cursors.lock().unwrap();
        let idx = cursors
            .entry(format!("{}-{}", tool_name, key.1))
            .or_insert(0);
        let record = records.get(*idx).unwrap_or_else(|| records.last().unwrap());
        *idx = (*idx + 1).min(records.len().saturating_sub(1));

        Ok(record.result.clone())
    }

    pub fn replay_llm_call(&self, prompt_hash: &str) -> Option<&str> {
        self.llm_index.get(prompt_hash).map(|r| r.response.as_str())
    }

    pub fn verify_replay(
        &self,
        recorded: &[ToolCallRecord],
        actual: &[ToolCallRecord],
    ) -> Vec<Discrepancy> {
        let mut discrepancies = vec![];

        for (rec, act) in recorded.iter().zip(actual.iter()) {
            if rec.tool_name != act.tool_name {
                discrepancies.push(Discrepancy {
                    tool_name: rec.tool_name.clone(),
                    field: "tool_name".to_string(),
                    recorded: serde_json::Value::String(rec.tool_name.clone()),
                    actual: serde_json::Value::String(act.tool_name.clone()),
                });
                continue;
            }
            if rec.result != act.result {
                discrepancies.push(Discrepancy {
                    tool_name: rec.tool_name.clone(),
                    field: "result".to_string(),
                    recorded: rec.result.clone(),
                    actual: act.result.clone(),
                });
            }
            if Self::hash_params(&rec.parameters) != Self::hash_params(&act.parameters) {
                discrepancies.push(Discrepancy {
                    tool_name: rec.tool_name.clone(),
                    field: "parameters".to_string(),
                    recorded: rec.parameters.clone(),
                    actual: act.parameters.clone(),
                });
            }
        }

        if recorded.len() != actual.len() {
            discrepancies.push(Discrepancy {
                tool_name: "N/A".to_string(),
                field: "call_count".to_string(),
                recorded: serde_json::Value::Number(recorded.len().into()),
                actual: serde_json::Value::Number(actual.len().into()),
            });
        }

        discrepancies
    }

    fn hash_params(params: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let s = serde_json::to_string(params).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// A simple Linear Congruential Generator seeded from a deterministic seed.
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        // LCG parameters from Knuth
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }

    pub fn next_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min))
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let n = slice.len();
        for i in (1..n).rev() {
            let j = self.next_range(0, (i + 1) as u64) as usize;
            slice.swap(i, j);
        }
    }
}
