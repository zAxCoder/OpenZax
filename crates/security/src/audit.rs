use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("hash chain broken at entry {0}")]
    BrokenChain(Uuid),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("export error: {0}")]
    Export(String),
    #[error("entry not found: {0}")]
    NotFound(Uuid),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEvent {
    SkillInstalled,
    SkillRemoved,
    SkillExecuted,
    CapabilityMinted,
    CapabilityRevoked,
    VaultAccess,
    VaultWrite,
    AgentSpawned,
    AgentKilled,
    WorkflowExecuted,
    UserLogin,
    UserLogout,
    ConfigChanged,
    PolicyViolation,
    AnomalyDetected,
    QuarantineStarted,
    QuarantineLifted,
    DataExported,
    DataImported,
    SystemStartup,
    SystemShutdown,
}

impl AuditEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEvent::SkillInstalled => "SkillInstalled",
            AuditEvent::SkillRemoved => "SkillRemoved",
            AuditEvent::SkillExecuted => "SkillExecuted",
            AuditEvent::CapabilityMinted => "CapabilityMinted",
            AuditEvent::CapabilityRevoked => "CapabilityRevoked",
            AuditEvent::VaultAccess => "VaultAccess",
            AuditEvent::VaultWrite => "VaultWrite",
            AuditEvent::AgentSpawned => "AgentSpawned",
            AuditEvent::AgentKilled => "AgentKilled",
            AuditEvent::WorkflowExecuted => "WorkflowExecuted",
            AuditEvent::UserLogin => "UserLogin",
            AuditEvent::UserLogout => "UserLogout",
            AuditEvent::ConfigChanged => "ConfigChanged",
            AuditEvent::PolicyViolation => "PolicyViolation",
            AuditEvent::AnomalyDetected => "AnomalyDetected",
            AuditEvent::QuarantineStarted => "QuarantineStarted",
            AuditEvent::QuarantineLifted => "QuarantineLifted",
            AuditEvent::DataExported => "DataExported",
            AuditEvent::DataImported => "DataImported",
            AuditEvent::SystemStartup => "SystemStartup",
            AuditEvent::SystemShutdown => "SystemShutdown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEvent,
    pub actor_id: String,
    pub target_id: Option<String>,
    pub metadata: Value,
    /// SHA-256 hex of the previous entry (empty string for the genesis entry).
    pub prev_hash: String,
    /// SHA-256 hex of this entry's canonical payload.
    pub hash: String,
}

impl AuditEntry {
    /// Computes the canonical SHA-256 hash for this entry using the chain.
    pub fn compute_hash(
        prev_hash: &str,
        timestamp: &DateTime<Utc>,
        event_type: &AuditEvent,
        actor_id: &str,
        metadata: &Value,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(event_type.as_str().as_bytes());
        hasher.update(actor_id.as_bytes());
        hasher.update(
            serde_json::to_string(metadata)
                .unwrap_or_default()
                .as_bytes(),
        );
        format!("{:x}", hasher.finalize())
    }
}

/// Query filter for audit log searches.
#[derive(Debug, Default, Clone)]
pub struct AuditQuery {
    pub actor_id: Option<String>,
    pub event_type: Option<AuditEvent>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

pub struct AuditLog {
    conn: Arc<Mutex<Connection>>,
}

impl AuditLog {
    /// Opens (or creates) the audit log at `db_path`.
    pub fn open(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        Self::init_db(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Creates an in-memory audit log (useful for tests).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_db(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS audit_log (
                 entry_id    TEXT PRIMARY KEY NOT NULL,
                 timestamp   TEXT NOT NULL,
                 event_type  TEXT NOT NULL,
                 actor_id    TEXT NOT NULL,
                 target_id   TEXT,
                 metadata    TEXT NOT NULL,
                 prev_hash   TEXT NOT NULL,
                 hash        TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
             CREATE INDEX IF NOT EXISTS idx_audit_actor    ON audit_log(actor_id);
             CREATE INDEX IF NOT EXISTS idx_audit_event    ON audit_log(event_type);",
        )?;
        Ok(())
    }

    /// Retrieves the hash of the most recent entry, or an empty string for the
    /// genesis entry.
    fn last_hash(&self, conn: &Connection) -> Result<String> {
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT hash FROM audit_log ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(h) => Ok(h),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
            Err(e) => Err(Error::Database(e)),
        }
    }

    /// Appends a new entry to the log, computing its hash and linking it to
    /// the previous entry.
    pub fn append(
        &self,
        event_type: AuditEvent,
        actor_id: impl Into<String>,
        target_id: Option<String>,
        metadata: Value,
    ) -> Result<AuditEntry> {
        let conn = self.conn.lock().unwrap();
        let prev_hash = self.last_hash(&conn)?;
        let timestamp = Utc::now();
        let actor_id = actor_id.into();

        let hash =
            AuditEntry::compute_hash(&prev_hash, &timestamp, &event_type, &actor_id, &metadata);

        let entry = AuditEntry {
            entry_id: Uuid::new_v4(),
            timestamp,
            event_type,
            actor_id,
            target_id,
            metadata,
            prev_hash,
            hash,
        };

        conn.execute(
            "INSERT INTO audit_log
                 (entry_id, timestamp, event_type, actor_id, target_id, metadata, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.entry_id.to_string(),
                entry.timestamp.to_rfc3339(),
                entry.event_type.as_str(),
                entry.actor_id,
                entry.target_id,
                serde_json::to_string(&entry.metadata)?,
                entry.prev_hash,
                entry.hash,
            ],
        )?;

        Ok(entry)
    }

    /// Validates the entire hash chain. Returns `Ok(())` if all hashes are
    /// consistent, or `Err(Error::BrokenChain(entry_id))` on the first broken
    /// link.
    pub fn verify_chain(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT entry_id, timestamp, event_type, actor_id, metadata, prev_hash, hash
             FROM audit_log
             ORDER BY timestamp ASC",
        )?;

        let entries = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut expected_prev = String::new();
        for row in entries {
            let (entry_id, timestamp_str, event_type_str, actor_id, metadata_str, prev_hash, hash) =
                row?;

            let entry_uuid = entry_id.parse::<Uuid>().unwrap_or_else(|_| Uuid::nil());

            if prev_hash != expected_prev {
                return Err(Error::BrokenChain(entry_uuid));
            }

            let timestamp = timestamp_str
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now());
            let metadata: Value = serde_json::from_str(&metadata_str).unwrap_or(Value::Null);

            // Reconstruct a dummy event for hash computation — we only need the
            // string representation which we have directly.
            let mut hasher = Sha256::new();
            hasher.update(prev_hash.as_bytes());
            hasher.update(timestamp.to_rfc3339().as_bytes());
            hasher.update(event_type_str.as_bytes());
            hasher.update(actor_id.as_bytes());
            hasher.update(
                serde_json::to_string(&metadata)
                    .unwrap_or_default()
                    .as_bytes(),
            );
            let computed = format!("{:x}", hasher.finalize());

            if computed != hash {
                return Err(Error::BrokenChain(entry_uuid));
            }

            expected_prev = hash;
        }
        Ok(())
    }

    /// Queries the audit log with optional filters.
    pub fn query(&self, filter: &AuditQuery) -> Result<Vec<AuditEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT entry_id, timestamp, event_type, actor_id, target_id, metadata, prev_hash, hash
             FROM audit_log WHERE 1=1",
        );
        let mut conditions: Vec<String> = Vec::new();

        if filter.actor_id.is_some() {
            conditions.push("AND actor_id = ?".to_owned());
        }
        if filter.event_type.is_some() {
            conditions.push("AND event_type = ?".to_owned());
        }
        if filter.since.is_some() {
            conditions.push("AND timestamp >= ?".to_owned());
        }
        if filter.until.is_some() {
            conditions.push("AND timestamp <= ?".to_owned());
        }

        for c in &conditions {
            sql.push(' ');
            sql.push_str(c);
        }
        sql.push_str(" ORDER BY timestamp ASC");
        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = conn.prepare(&sql)?;

        // Build the params tuple manually (rusqlite requires positional params).
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(ref a) = filter.actor_id {
            params_vec.push(Box::new(a.clone()));
        }
        if let Some(ref e) = filter.event_type {
            params_vec.push(Box::new(e.as_str().to_owned()));
        }
        if let Some(ref s) = filter.since {
            params_vec.push(Box::new(s.to_rfc3339()));
        }
        if let Some(ref u) = filter.until {
            params_vec.push(Box::new(u.to_rfc3339()));
        }

        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let entries = stmt
            .query_map(refs.as_slice(), |row| {
                let event_str: String = row.get(2)?;
                let event_type = parse_event_type(&event_str);
                let metadata_str: String = row.get(5)?;
                let metadata: Value = serde_json::from_str(&metadata_str).unwrap_or(Value::Null);
                let ts_str: String = row.get(1)?;
                let timestamp = ts_str
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now());
                let entry_id: String = row.get(0)?;
                let entry_uuid = entry_id.parse::<Uuid>().unwrap_or_else(|_| Uuid::nil());

                Ok(AuditEntry {
                    entry_id: entry_uuid,
                    timestamp,
                    event_type,
                    actor_id: row.get(3)?,
                    target_id: row.get(4)?,
                    metadata,
                    prev_hash: row.get(6)?,
                    hash: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entries)
    }

    /// Exports all audit entries as a JSON array.
    pub fn export_json(&self) -> Result<String> {
        let entries = self.query(&AuditQuery::default())?;
        Ok(serde_json::to_string_pretty(&entries)?)
    }

    /// Exports all audit entries as CSV.
    pub fn export_csv(&self) -> Result<String> {
        let entries = self.query(&AuditQuery::default())?;
        let mut csv =
            String::from("entry_id,timestamp,event_type,actor_id,target_id,prev_hash,hash\n");
        for e in &entries {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                e.entry_id,
                e.timestamp.to_rfc3339(),
                e.event_type.as_str(),
                e.actor_id,
                e.target_id.as_deref().unwrap_or(""),
                e.prev_hash,
                e.hash,
            ));
        }
        Ok(csv)
    }
}

fn parse_event_type(s: &str) -> AuditEvent {
    match s {
        "SkillInstalled" => AuditEvent::SkillInstalled,
        "SkillRemoved" => AuditEvent::SkillRemoved,
        "SkillExecuted" => AuditEvent::SkillExecuted,
        "CapabilityMinted" => AuditEvent::CapabilityMinted,
        "CapabilityRevoked" => AuditEvent::CapabilityRevoked,
        "VaultAccess" => AuditEvent::VaultAccess,
        "VaultWrite" => AuditEvent::VaultWrite,
        "AgentSpawned" => AuditEvent::AgentSpawned,
        "AgentKilled" => AuditEvent::AgentKilled,
        "WorkflowExecuted" => AuditEvent::WorkflowExecuted,
        "UserLogin" => AuditEvent::UserLogin,
        "UserLogout" => AuditEvent::UserLogout,
        "ConfigChanged" => AuditEvent::ConfigChanged,
        "PolicyViolation" => AuditEvent::PolicyViolation,
        "AnomalyDetected" => AuditEvent::AnomalyDetected,
        "QuarantineStarted" => AuditEvent::QuarantineStarted,
        "QuarantineLifted" => AuditEvent::QuarantineLifted,
        "DataExported" => AuditEvent::DataExported,
        "DataImported" => AuditEvent::DataImported,
        "SystemStartup" => AuditEvent::SystemStartup,
        "SystemShutdown" => AuditEvent::SystemShutdown,
        _ => AuditEvent::PolicyViolation,
    }
}
