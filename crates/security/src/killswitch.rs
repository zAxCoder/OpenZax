use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("checkpoint not found: {0}")]
    CheckpointNotFound(Uuid),
    #[error("kill switch is not armed")]
    NotArmed,
    #[error("kill switch already armed")]
    AlreadyArmed,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("send error: kill channel closed")]
    SendError,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KillSwitchTrigger {
    UserHotkey,
    AnomalyDetected,
    BudgetExhausted,
    PolicyViolation,
    WatchdogTimeout,
    ManualCommand,
}

impl KillSwitchTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            KillSwitchTrigger::UserHotkey => "UserHotkey",
            KillSwitchTrigger::AnomalyDetected => "AnomalyDetected",
            KillSwitchTrigger::BudgetExhausted => "BudgetExhausted",
            KillSwitchTrigger::PolicyViolation => "PolicyViolation",
            KillSwitchTrigger::WatchdogTimeout => "WatchdogTimeout",
            KillSwitchTrigger::ManualCommand => "ManualCommand",
        }
    }
}

/// A kill signal that is broadcast when the kill switch fires.
#[derive(Debug, Clone)]
pub struct KillSignal {
    pub trigger: KillSwitchTrigger,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub checkpoint_id: Uuid,
    pub agent_id: String,
    pub timestamp: DateTime<Utc>,
    /// Opaque serialized JSON snapshot of the agent state.
    pub state_snapshot: Vec<u8>,
    pub tool_calls_count: u64,
    pub tokens_consumed: u64,
}

pub struct KillSwitch {
    armed: bool,
    sender: broadcast::Sender<KillSignal>,
    conn: Arc<Mutex<Connection>>,
}

impl KillSwitch {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        Self::init_db(&conn)?;
        let (sender, _) = broadcast::channel(64);
        Ok(Self {
            armed: false,
            sender,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_db(&conn)?;
        let (sender, _) = broadcast::channel(64);
        Ok(Self {
            armed: false,
            sender,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS checkpoints (
                 checkpoint_id   TEXT PRIMARY KEY NOT NULL,
                 agent_id        TEXT NOT NULL,
                 timestamp       TEXT NOT NULL,
                 state_snapshot  BLOB NOT NULL,
                 tool_calls_count INTEGER NOT NULL,
                 tokens_consumed  INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_ckpt_agent ON checkpoints(agent_id);",
        )?;
        Ok(())
    }

    /// Arms the kill switch, allowing it to fire.
    pub fn arm(&mut self) -> Result<()> {
        if self.armed {
            return Err(Error::AlreadyArmed);
        }
        self.armed = true;
        tracing::info!("KillSwitch armed");
        Ok(())
    }

    /// Fires the kill switch, broadcasting a `KillSignal` to all subscribers.
    pub fn trigger(&self, trigger: KillSwitchTrigger, reason: impl Into<String>) -> Result<()> {
        if !self.armed {
            return Err(Error::NotArmed);
        }
        let signal = KillSignal {
            trigger,
            reason: reason.into(),
            timestamp: Utc::now(),
        };
        tracing::warn!(
            trigger = signal.trigger.as_str(),
            reason = %signal.reason,
            "KillSwitch TRIGGERED"
        );
        self.sender.send(signal).map_err(|_| Error::SendError)?;
        Ok(())
    }

    /// Returns a new receiver that will receive kill signals.
    pub fn subscribe(&self) -> broadcast::Receiver<KillSignal> {
        self.sender.subscribe()
    }

    /// Persists a checkpoint for an agent.
    pub fn create_checkpoint(
        &self,
        agent_id: impl Into<String>,
        state_snapshot: Vec<u8>,
        tool_calls_count: u64,
        tokens_consumed: u64,
    ) -> Result<Checkpoint> {
        let ckpt = Checkpoint {
            checkpoint_id: Uuid::new_v4(),
            agent_id: agent_id.into(),
            timestamp: Utc::now(),
            state_snapshot,
            tool_calls_count,
            tokens_consumed,
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO checkpoints
                 (checkpoint_id, agent_id, timestamp, state_snapshot, tool_calls_count, tokens_consumed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                ckpt.checkpoint_id.to_string(),
                ckpt.agent_id,
                ckpt.timestamp.to_rfc3339(),
                ckpt.state_snapshot,
                ckpt.tool_calls_count as i64,
                ckpt.tokens_consumed as i64,
            ],
        )?;
        Ok(ckpt)
    }

    /// Loads a checkpoint by its ID.
    pub fn restore_checkpoint(&self, checkpoint_id: Uuid) -> Result<Checkpoint> {
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<Checkpoint> = conn.query_row(
            "SELECT checkpoint_id, agent_id, timestamp, state_snapshot, tool_calls_count, tokens_consumed
             FROM checkpoints WHERE checkpoint_id = ?1",
            params![checkpoint_id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let ts_str: String = row.get(2)?;
                Ok(Checkpoint {
                    checkpoint_id: id_str.parse::<Uuid>().unwrap_or(Uuid::nil()),
                    agent_id: row.get(1)?,
                    timestamp: ts_str
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                    state_snapshot: row.get(3)?,
                    tool_calls_count: row.get::<_, i64>(4)? as u64,
                    tokens_consumed: row.get::<_, i64>(5)? as u64,
                })
            },
        );
        match result {
            Ok(c) => Ok(c),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(Error::CheckpointNotFound(checkpoint_id))
            }
            Err(e) => Err(Error::Database(e)),
        }
    }

    /// Lists all checkpoints for a given agent, ordered by timestamp descending.
    pub fn list_checkpoints(&self, agent_id: &str) -> Result<Vec<Checkpoint>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT checkpoint_id, agent_id, timestamp, state_snapshot, tool_calls_count, tokens_consumed
             FROM checkpoints WHERE agent_id = ?1 ORDER BY timestamp DESC",
        )?;
        let checkpoints = stmt
            .query_map(params![agent_id], |row| {
                let id_str: String = row.get(0)?;
                let ts_str: String = row.get(2)?;
                Ok(Checkpoint {
                    checkpoint_id: id_str.parse::<Uuid>().unwrap_or(Uuid::nil()),
                    agent_id: row.get(1)?,
                    timestamp: ts_str
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                    state_snapshot: row.get(3)?,
                    tool_calls_count: row.get::<_, i64>(4)? as u64,
                    tokens_consumed: row.get::<_, i64>(5)? as u64,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(checkpoints)
    }
}

/// Runs a background tokio task that fires the kill switch if the watched
/// agent does not call `heartbeat()` within the timeout window.
pub struct Watchdog {
    last_heartbeat: Arc<Mutex<std::time::Instant>>,
    timeout: Duration,
    agent_id: String,
}

impl Watchdog {
    /// Creates a new watchdog with the given timeout (defaults recommend 30s).
    pub fn new(agent_id: impl Into<String>, timeout: Duration) -> Self {
        Self {
            last_heartbeat: Arc::new(Mutex::new(std::time::Instant::now())),
            timeout,
            agent_id: agent_id.into(),
        }
    }

    /// Resets the heartbeat timer. Call this regularly from the monitored task.
    pub fn heartbeat(&self) {
        *self.last_heartbeat.lock().unwrap() = std::time::Instant::now();
    }

    /// Returns a `JoinHandle` for the watchdog background task. The handle
    /// should be stored alongside the kill switch.
    ///
    /// When the monitored agent is silent for longer than `timeout`, the
    /// provided `kill_switch` is triggered with `WatchdogTimeout`.
    pub fn spawn(
        self,
        kill_switch: Arc<Mutex<KillSwitch>>,
    ) -> tokio::task::JoinHandle<()> {
        let last_heartbeat = Arc::clone(&self.last_heartbeat);
        let timeout = self.timeout;
        let agent_id = self.agent_id.clone();

        tokio::spawn(async move {
            let poll_interval = Duration::from_secs(1);
            loop {
                tokio::time::sleep(poll_interval).await;
                let elapsed = last_heartbeat.lock().unwrap().elapsed();
                if elapsed > timeout {
                    tracing::warn!(
                        agent_id = %agent_id,
                        elapsed_secs = elapsed.as_secs(),
                        "Watchdog timeout — triggering kill switch"
                    );
                    if let Ok(ks) = kill_switch.lock() {
                        let _ = ks.trigger(
                            KillSwitchTrigger::WatchdogTimeout,
                            format!(
                                "Agent {} silent for {}s",
                                agent_id,
                                elapsed.as_secs()
                            ),
                        );
                    }
                    break;
                }
            }
        })
    }
}
