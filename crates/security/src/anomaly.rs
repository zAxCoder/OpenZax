use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("insufficient baseline samples: need at least {0}")]
    InsufficientSamples(usize),
    #[error("metric '{0}' not found in baseline")]
    MetricNotFound(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Minimum number of samples required before a baseline is considered stable.
const MIN_BASELINE_SAMPLES: usize = 5;

/// Z-score threshold above which an anomaly alert is raised.
const ANOMALY_ZSCORE_THRESHOLD: f64 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    ExcessiveFileReads,
    ExcessiveNetworkRequests,
    CpuAnomalous,
    MemorySpike,
    UnusualToolPattern,
    SuspiciousDataFlow,
}

/// A snapshot of behavioural metrics for a running skill.
#[derive(Debug, Clone)]
pub struct BehaviorMetrics {
    pub file_reads: u64,
    pub file_writes: u64,
    pub network_requests: u64,
    pub cpu_fuel_consumed: u64,
    pub memory_bytes_used: u64,
    pub tool_calls: HashMap<String, u64>,
    pub start_time: Instant,
    pub last_update: Instant,
}

impl BehaviorMetrics {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            file_reads: 0,
            file_writes: 0,
            network_requests: 0,
            cpu_fuel_consumed: 0,
            memory_bytes_used: 0,
            tool_calls: HashMap::new(),
            start_time: now,
            last_update: now,
        }
    }

    /// Returns the elapsed time since the metrics were last updated.
    pub fn idle_duration(&self) -> Duration {
        self.last_update.elapsed()
    }

    /// Returns the total number of tool calls across all tool names.
    pub fn total_tool_calls(&self) -> u64 {
        self.tool_calls.values().sum()
    }

    /// Records a file read.
    pub fn record_file_read(&mut self) {
        self.file_reads += 1;
        self.last_update = Instant::now();
    }

    /// Records a file write.
    pub fn record_file_write(&mut self) {
        self.file_writes += 1;
        self.last_update = Instant::now();
    }

    /// Records a network request.
    pub fn record_network_request(&mut self) {
        self.network_requests += 1;
        self.last_update = Instant::now();
    }

    /// Records CPU fuel consumption.
    pub fn record_cpu_fuel(&mut self, units: u64) {
        self.cpu_fuel_consumed += units;
        self.last_update = Instant::now();
    }

    /// Records memory usage (absolute, not additive).
    pub fn record_memory(&mut self, bytes: u64) {
        self.memory_bytes_used = bytes;
        self.last_update = Instant::now();
    }

    /// Records a single call to a named tool.
    pub fn record_tool_call(&mut self, tool_name: impl Into<String>) {
        *self.tool_calls.entry(tool_name.into()).or_insert(0) += 1;
        self.last_update = Instant::now();
    }
}

impl Default for BehaviorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Welford's online algorithm state for computing running mean and variance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WelfordState {
    count: u64,
    mean: f64,
    /// Running sum of squared deviations (M2 in Welford's notation).
    m2: f64,
}

impl WelfordState {
    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    fn mean(&self) -> f64 {
        self.mean
    }

    fn variance(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        self.m2 / (self.count - 1) as f64
    }

    fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    fn zscore(&self, value: f64) -> f64 {
        let sd = self.std_dev();
        if sd == 0.0 {
            return 0.0;
        }
        (value - self.mean()) / sd
    }
}

/// Statistical baseline profile for a skill built from historic observations.
#[derive(Debug, Default, Clone)]
pub struct BaselineProfile {
    file_reads: WelfordState,
    file_writes: WelfordState,
    network_requests: WelfordState,
    cpu_fuel: WelfordState,
    memory_bytes: WelfordState,
    total_tool_calls: WelfordState,
    sample_count: usize,
}

impl BaselineProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a metrics snapshot into the baseline.
    pub fn observe(&mut self, m: &BehaviorMetrics) {
        self.file_reads.update(m.file_reads as f64);
        self.file_writes.update(m.file_writes as f64);
        self.network_requests.update(m.network_requests as f64);
        self.cpu_fuel.update(m.cpu_fuel_consumed as f64);
        self.memory_bytes.update(m.memory_bytes_used as f64);
        self.total_tool_calls.update(m.total_tool_calls() as f64);
        self.sample_count += 1;
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn is_ready(&self) -> bool {
        self.sample_count >= MIN_BASELINE_SAMPLES
    }
}

/// Serializable snapshot of metrics used in anomaly alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub file_reads: u64,
    pub file_writes: u64,
    pub network_requests: u64,
    pub cpu_fuel_consumed: u64,
    pub memory_bytes_used: u64,
    pub total_tool_calls: u64,
}

impl From<&BehaviorMetrics> for MetricsSnapshot {
    fn from(m: &BehaviorMetrics) -> Self {
        Self {
            file_reads: m.file_reads,
            file_writes: m.file_writes,
            network_requests: m.network_requests,
            cpu_fuel_consumed: m.cpu_fuel_consumed,
            memory_bytes_used: m.memory_bytes_used,
            total_tool_calls: m.total_tool_calls(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub skill_id: String,
    pub alert_type: AnomalyType,
    pub zscore: f64,
    pub metrics_snapshot: MetricsSnapshot,
    pub suggested_action: String,
}

/// Per-dimension z-scores produced by `AnomalyDetector::compute_zscore`.
#[derive(Debug, Clone)]
pub struct ZScores {
    pub file_reads: f64,
    pub file_writes: f64,
    pub network_requests: f64,
    pub cpu_fuel: f64,
    pub memory_bytes: f64,
    pub total_tool_calls: f64,
}

/// Weights for the compound anomaly score.
const W_FILE_READS: f64 = 0.15;
const W_FILE_WRITES: f64 = 0.15;
const W_NETWORK: f64 = 0.25;
const W_CPU: f64 = 0.20;
const W_MEMORY: f64 = 0.15;
const W_TOOLS: f64 = 0.10;

pub struct AnomalyDetector {
    skill_id: String,
    baseline: BaselineProfile,
}

impl AnomalyDetector {
    pub fn new(skill_id: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.into(),
            baseline: BaselineProfile::new(),
        }
    }

    /// Feeds a metrics snapshot into the baseline model.
    pub fn update_metrics(&mut self, metrics: &BehaviorMetrics) {
        self.baseline.observe(metrics);
    }

    /// Computes per-dimension z-scores for the given metrics against the
    /// current baseline.
    pub fn compute_zscore(&self, metrics: &BehaviorMetrics) -> Result<ZScores> {
        if !self.baseline.is_ready() {
            return Err(Error::InsufficientSamples(MIN_BASELINE_SAMPLES));
        }
        Ok(ZScores {
            file_reads: self.baseline.file_reads.zscore(metrics.file_reads as f64),
            file_writes: self.baseline.file_writes.zscore(metrics.file_writes as f64),
            network_requests: self
                .baseline
                .network_requests
                .zscore(metrics.network_requests as f64),
            cpu_fuel: self
                .baseline
                .cpu_fuel
                .zscore(metrics.cpu_fuel_consumed as f64),
            memory_bytes: self
                .baseline
                .memory_bytes
                .zscore(metrics.memory_bytes_used as f64),
            total_tool_calls: self
                .baseline
                .total_tool_calls
                .zscore(metrics.total_tool_calls() as f64),
        })
    }

    /// Computes a weighted compound anomaly score from individual z-scores.
    pub fn compound_score(&self, z: &ZScores) -> f64 {
        W_FILE_READS * z.file_reads.abs()
            + W_FILE_WRITES * z.file_writes.abs()
            + W_NETWORK * z.network_requests.abs()
            + W_CPU * z.cpu_fuel.abs()
            + W_MEMORY * z.memory_bytes.abs()
            + W_TOOLS * z.total_tool_calls.abs()
    }

    /// Checks the given metrics against the baseline. Returns `Some(alert)` if
    /// any dimension exceeds the threshold, or `None` if everything is normal.
    ///
    /// If the baseline has fewer than `MIN_BASELINE_SAMPLES` observations, no
    /// alert is raised (returns `None`).
    pub fn check_anomaly(&self, metrics: &BehaviorMetrics) -> Option<AnomalyAlert> {
        let z = match self.compute_zscore(metrics) {
            Ok(z) => z,
            Err(_) => return None,
        };

        let compound = self.compound_score(&z);
        if compound < ANOMALY_ZSCORE_THRESHOLD {
            return None;
        }

        // Identify the dominant anomaly dimension.
        let (alert_type, dim_zscore) = self.dominant_anomaly(&z);
        let suggested_action = self.suggested_action(&alert_type);

        Some(AnomalyAlert {
            skill_id: self.skill_id.clone(),
            alert_type,
            zscore: dim_zscore,
            metrics_snapshot: MetricsSnapshot::from(metrics),
            suggested_action,
        })
    }

    /// Returns the anomaly type and z-score for the highest-scoring dimension.
    fn dominant_anomaly(&self, z: &ZScores) -> (AnomalyType, f64) {
        let candidates: &[(f64, AnomalyType)] = &[
            (z.file_reads.abs(), AnomalyType::ExcessiveFileReads),
            (
                z.network_requests.abs(),
                AnomalyType::ExcessiveNetworkRequests,
            ),
            (z.cpu_fuel.abs(), AnomalyType::CpuAnomalous),
            (z.memory_bytes.abs(), AnomalyType::MemorySpike),
            (z.total_tool_calls.abs(), AnomalyType::UnusualToolPattern),
            (z.file_writes.abs(), AnomalyType::SuspiciousDataFlow),
        ];

        candidates
            .iter()
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(score, t)| (t.clone(), *score))
            .unwrap_or((AnomalyType::UnusualToolPattern, 0.0))
    }

    fn suggested_action(&self, t: &AnomalyType) -> String {
        match t {
            AnomalyType::ExcessiveFileReads => {
                "Throttle file-read operations and audit accessed paths.".to_owned()
            }
            AnomalyType::ExcessiveNetworkRequests => {
                "Rate-limit outbound network calls and review egress policy.".to_owned()
            }
            AnomalyType::CpuAnomalous => {
                "Profile CPU usage and consider capping compute fuel budget.".to_owned()
            }
            AnomalyType::MemorySpike => {
                "Inspect heap allocations; consider adding a memory limit.".to_owned()
            }
            AnomalyType::UnusualToolPattern => {
                "Review tool-call sequence for unexpected capability usage.".to_owned()
            }
            AnomalyType::SuspiciousDataFlow => {
                "Audit file-write targets for potential data exfiltration.".to_owned()
            }
        }
    }

    pub fn baseline(&self) -> &BaselineProfile {
        &self.baseline
    }

    pub fn skill_id(&self) -> &str {
        &self.skill_id
    }
}
