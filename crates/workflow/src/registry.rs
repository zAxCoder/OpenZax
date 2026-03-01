use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::Mutex;
use thiserror::Error;
use tracing::{debug, info};
use uuid::Uuid;

use crate::graph::Workflow;

const MAX_RUNS_PER_WORKFLOW: usize = 1000;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Workflow not found: {0}")]
    NotFound(Uuid),

    #[error("Version not found: workflow {workflow_id} version {version}")]
    VersionNotFound { workflow_id: Uuid, version: u32 },
}

pub type RegistryResult<T> = std::result::Result<T, RegistryError>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionHistory {
    pub run_id: Uuid,
    pub workflow_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub nodes_executed: u32,
    pub trigger_payload: serde_json::Value,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowVersion {
    pub workflow_id: Uuid,
    pub version: u32,
    pub snapshot: serde_json::Value,
    pub saved_at: DateTime<Utc>,
    pub change_summary: Option<String>,
}

/// SQLite-backed workflow registry
pub struct WorkflowRegistry {
    conn: Mutex<Connection>,
}

impl WorkflowRegistry {
    pub fn open(path: &str) -> RegistryResult<Self> {
        let conn = Connection::open(path)?;
        let reg = Self { conn: Mutex::new(conn) };
        reg.initialize()?;
        Ok(reg)
    }

    pub fn open_in_memory() -> RegistryResult<Self> {
        let conn = Connection::open_in_memory()?;
        let reg = Self { conn: Mutex::new(conn) };
        reg.initialize()?;
        Ok(reg)
    }

    fn with_conn<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Connection) -> T,
    {
        let conn = self.conn.lock().expect("WorkflowRegistry mutex poisoned");
        f(&conn)
    }

    pub fn initialize(&self) -> RegistryResult<()> {
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

            conn.execute_batch(r#"
                CREATE TABLE IF NOT EXISTS workflows (
                    id          TEXT PRIMARY KEY,
                    name        TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    version     INTEGER NOT NULL DEFAULT 1,
                    body        TEXT NOT NULL,
                    is_active   INTEGER NOT NULL DEFAULT 0,
                    created_at  TEXT NOT NULL,
                    updated_at  TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS workflow_versions (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    workflow_id TEXT NOT NULL REFERENCES workflows(id),
                    version     INTEGER NOT NULL,
                    snapshot    TEXT NOT NULL,
                    change_summary TEXT,
                    saved_at    TEXT NOT NULL,
                    UNIQUE(workflow_id, version)
                );

                CREATE TABLE IF NOT EXISTS execution_history (
                    run_id          TEXT PRIMARY KEY,
                    workflow_id     TEXT NOT NULL REFERENCES workflows(id),
                    success         INTEGER NOT NULL,
                    output          TEXT NOT NULL DEFAULT 'null',
                    error_message   TEXT,
                    duration_ms     INTEGER NOT NULL DEFAULT 0,
                    nodes_executed  INTEGER NOT NULL DEFAULT 0,
                    trigger_payload TEXT NOT NULL DEFAULT 'null',
                    started_at      TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_exec_workflow ON execution_history(workflow_id, started_at DESC);
            "#)?;

            info!("Workflow registry initialized");
            Ok(())
        })
    }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    pub fn create(&self, workflow: &Workflow) -> RegistryResult<()> {
        let body = serde_json::to_string(workflow)?;
        self.with_conn(|conn| -> RegistryResult<()> {
            conn.execute(
                r#"INSERT INTO workflows (id, name, description, version, body, is_active, created_at, updated_at)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8)"#,
                params![
                    workflow.id.to_string(),
                    workflow.name,
                    workflow.description,
                    workflow.version as i64,
                    body,
                    workflow.is_active as i64,
                    workflow.created_at.to_rfc3339(),
                    workflow.updated_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;
        self.save_version_internal(workflow, None)?;
        debug!("Created workflow {} ({})", workflow.id, workflow.name);
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> RegistryResult<Option<Workflow>> {
        let body: Option<String> = self.with_conn(|conn| {
            conn.query_row(
                "SELECT body FROM workflows WHERE id = ?1",
                params![id.to_string()],
                |row| row.get::<_, String>(0),
            ).optional()
        })?;

        match body {
            Some(b) => Ok(Some(serde_json::from_str(&b)?)),
            None => Ok(None),
        }
    }

    pub fn update(&self, workflow: &Workflow) -> RegistryResult<()> {
        let body = serde_json::to_string(workflow)?;
        let affected: usize = self.with_conn(|conn| {
            conn.execute(
                r#"UPDATE workflows SET name=?1, description=?2, version=?3, body=?4,
                   is_active=?5, updated_at=?6 WHERE id=?7"#,
                params![
                    workflow.name,
                    workflow.description,
                    workflow.version as i64,
                    body,
                    workflow.is_active as i64,
                    Utc::now().to_rfc3339(),
                    workflow.id.to_string(),
                ],
            )
        })?;

        if affected == 0 {
            return Err(RegistryError::NotFound(workflow.id));
        }

        self.save_version_internal(workflow, None)?;
        Ok(())
    }

    pub fn delete(&self, id: Uuid) -> RegistryResult<()> {
        let affected: usize = self.with_conn(|conn| {
            conn.execute("DELETE FROM workflows WHERE id = ?1", params![id.to_string()])
        })?;
        if affected == 0 {
            return Err(RegistryError::NotFound(id));
        }
        Ok(())
    }

    pub fn list(&self) -> RegistryResult<Vec<Workflow>> {
        let bodies: Vec<String> = self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT body FROM workflows ORDER BY updated_at DESC")?;
            let result: rusqlite::Result<Vec<String>> = stmt.query_map([], |row| row.get::<_, String>(0))?
                .collect();
            result
        })?;
        Ok(bodies.into_iter().filter_map(|b| serde_json::from_str(&b).ok()).collect())
    }

    pub fn list_active(&self) -> RegistryResult<Vec<Workflow>> {
        let bodies: Vec<String> = self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT body FROM workflows WHERE is_active = 1 ORDER BY updated_at DESC",
            )?;
            let result: rusqlite::Result<Vec<String>> = stmt.query_map([], |row| row.get::<_, String>(0))?
                .collect();
            result
        })?;
        Ok(bodies.into_iter().filter_map(|b| serde_json::from_str(&b).ok()).collect())
    }

    // ── Version history ───────────────────────────────────────────────────────

    pub fn save_version(&self, workflow: &Workflow, change_summary: Option<&str>) -> RegistryResult<()> {
        self.save_version_internal(workflow, change_summary)
    }

    fn save_version_internal(&self, workflow: &Workflow, change_summary: Option<&str>) -> RegistryResult<()> {
        let snapshot = serde_json::to_string(workflow)?;
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| -> RegistryResult<()> {
            conn.execute(
                r#"INSERT OR IGNORE INTO workflow_versions (workflow_id, version, snapshot, change_summary, saved_at)
                   VALUES (?1,?2,?3,?4,?5)"#,
                params![
                    workflow.id.to_string(),
                    workflow.version as i64,
                    snapshot,
                    change_summary,
                    now,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_version(&self, workflow_id: Uuid, version: u32) -> RegistryResult<Option<WorkflowVersion>> {
        let row: Option<(String, Option<String>, String)> = self.with_conn(|conn| {
            conn.query_row(
                "SELECT snapshot, change_summary, saved_at FROM workflow_versions WHERE workflow_id=?1 AND version=?2",
                params![workflow_id.to_string(), version as i64],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?)),
            ).optional()
        })?;

        match row {
            Some((snapshot_str, change_summary, saved_str)) => {
                let snapshot: serde_json::Value = serde_json::from_str(&snapshot_str)?;
                let saved_at = DateTime::parse_from_rfc3339(&saved_str)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(Some(WorkflowVersion { workflow_id, version, snapshot, saved_at, change_summary }))
            }
            None => Ok(None),
        }
    }

    pub fn list_versions(&self, workflow_id: Uuid) -> RegistryResult<Vec<WorkflowVersion>> {
        let rows: Vec<(u32, String, Option<String>, String)> = self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT version, snapshot, change_summary, saved_at FROM workflow_versions WHERE workflow_id=?1 ORDER BY version DESC",
            )?;
            let result: rusqlite::Result<Vec<_>> = stmt.query_map(params![workflow_id.to_string()], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?.collect();
            result
        })?;

        let versions = rows.into_iter().filter_map(|(version, snapshot_str, change_summary, saved_str)| {
            let snapshot = serde_json::from_str(&snapshot_str).ok()?;
            let saved_at = DateTime::parse_from_rfc3339(&saved_str)
                .map(|d| d.with_timezone(&Utc)).ok()?;
            Some(WorkflowVersion { workflow_id, version, snapshot, saved_at, change_summary })
        }).collect();

        Ok(versions)
    }

    /// Returns a JSON diff between two versions (field-level additions/removals/changes)
    pub fn diff_versions(&self, workflow_id: Uuid, v1: u32, v2: u32) -> RegistryResult<serde_json::Value> {
        let ver1 = self.get_version(workflow_id, v1)?
            .ok_or(RegistryError::VersionNotFound { workflow_id, version: v1 })?;
        let ver2 = self.get_version(workflow_id, v2)?
            .ok_or(RegistryError::VersionNotFound { workflow_id, version: v2 })?;

        Ok(json_diff(&ver1.snapshot, &ver2.snapshot))
    }

    // ── Execution history ─────────────────────────────────────────────────────

    pub fn record_execution(&self, history: &ExecutionHistory) -> RegistryResult<()> {
        let output_str = serde_json::to_string(&history.output)?;
        let trigger_str = serde_json::to_string(&history.trigger_payload)?;

        self.with_conn(|conn| -> RegistryResult<()> {
            conn.execute(
                r#"INSERT INTO execution_history
                   (run_id, workflow_id, success, output, error_message, duration_ms, nodes_executed, trigger_payload, started_at)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)"#,
                params![
                    history.run_id.to_string(),
                    history.workflow_id.to_string(),
                    history.success as i64,
                    output_str,
                    history.error_message,
                    history.duration_ms as i64,
                    history.nodes_executed as i64,
                    trigger_str,
                    history.started_at.to_rfc3339(),
                ],
            )?;

            // Enforce 1000-run limit per workflow
            conn.execute(
                r#"DELETE FROM execution_history WHERE run_id IN (
                   SELECT run_id FROM execution_history WHERE workflow_id=?1
                   ORDER BY started_at DESC LIMIT -1 OFFSET ?2)"#,
                params![history.workflow_id.to_string(), MAX_RUNS_PER_WORKFLOW as i64],
            )?;

            Ok(())
        })
    }

    pub fn get_execution_history(&self, workflow_id: Uuid, limit: u32) -> RegistryResult<Vec<ExecutionHistory>> {
        let rows: Vec<(String, i64, String, Option<String>, u64, u32, String, String)> = self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT run_id, success, output, error_message, duration_ms, nodes_executed, trigger_payload, started_at
                   FROM execution_history WHERE workflow_id=?1 ORDER BY started_at DESC LIMIT ?2"#,
            )?;
            let result: rusqlite::Result<Vec<_>> = stmt.query_map(params![workflow_id.to_string(), limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)? as u64,
                    row.get::<_, i64>(5)? as u32,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?.collect();
            result
        })?;

        let history = rows.into_iter().filter_map(|(run_id_str, success, output_str, error_message, duration_ms, nodes_executed, trigger_str, started_str)| {
            let run_id = Uuid::parse_str(&run_id_str).ok()?;
            let output = serde_json::from_str(&output_str).ok()?;
            let trigger_payload = serde_json::from_str(&trigger_str).ok()?;
            let started_at = DateTime::parse_from_rfc3339(&started_str)
                .map(|d| d.with_timezone(&Utc)).ok()?;

            Some(ExecutionHistory {
                run_id,
                workflow_id,
                success: success != 0,
                output,
                error_message,
                duration_ms,
                nodes_executed,
                trigger_payload,
                started_at,
            })
        }).collect();

        Ok(history)
    }

    pub fn execution_success_rate(&self, workflow_id: Uuid) -> RegistryResult<f64> {
        let (total, successes): (i64, i64) = self.with_conn(|conn| {
            conn.query_row(
                r#"SELECT COUNT(*), SUM(CASE WHEN success=1 THEN 1 ELSE 0 END)
                   FROM execution_history WHERE workflow_id=?1"#,
                params![workflow_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        })?;

        if total == 0 { return Ok(0.0); }
        Ok(successes as f64 / total as f64)
    }
}

fn json_diff(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
    use serde_json::{Map, Value};

    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            let mut diff = Map::new();

            for (key, a_val) in a_map {
                if let Some(b_val) = b_map.get(key) {
                    if a_val != b_val {
                        diff.insert(key.clone(), serde_json::json!({
                            "from": a_val,
                            "to": b_val,
                        }));
                    }
                } else {
                    diff.insert(key.clone(), serde_json::json!({ "removed": a_val }));
                }
            }

            for (key, b_val) in b_map {
                if !a_map.contains_key(key) {
                    diff.insert(key.clone(), serde_json::json!({ "added": b_val }));
                }
            }

            Value::Object(diff)
        }
        (a, b) if a == b => serde_json::json!({}),
        _ => serde_json::json!({ "from": a, "to": b }),
    }
}
