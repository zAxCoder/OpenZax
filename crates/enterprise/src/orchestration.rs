use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Task already completed")]
    TaskAlreadyCompleted,
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceClass {
    Micro,
    Standard,
    Performance,
}

impl ResourceClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceClass::Micro => "micro",
            ResourceClass::Standard => "standard",
            ResourceClass::Performance => "performance",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "micro" => ResourceClass::Micro,
            "performance" => ResourceClass::Performance,
            _ => ResourceClass::Standard,
        }
    }

    /// Cost multiplier relative to Standard
    pub fn cost_multiplier(&self) -> f32 {
        match self {
            ResourceClass::Micro => 0.5,
            ResourceClass::Standard => 1.0,
            ResourceClass::Performance => 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: Uuid,
    pub org_id: Uuid,
    pub workflow_id: Option<String>,
    pub skill_ids: Vec<String>,
    pub input: serde_json::Value,
    pub priority: u8,
    pub max_duration_secs: u64,
    pub resource_class: ResourceClass,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum TaskStatus {
    Queued,
    Running {
        started_at: DateTime<Utc>,
        worker_id: String,
    },
    Completed {
        result: serde_json::Value,
        duration_ms: u64,
    },
    Failed {
        error: String,
        duration_ms: u64,
    },
    Cancelled,
}

impl TaskStatus {
    pub fn as_state_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Running { .. } => "running",
            TaskStatus::Completed { .. } => "completed",
            TaskStatus::Failed { .. } => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub spec: TaskSpec,
    pub status: TaskStatus,
    pub logs: Vec<LogLine>,
    pub metered_minutes: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    pub total_workers: u32,
    pub busy_workers: u32,
    pub queue_depth: u32,
    pub avg_task_duration_secs: f32,
    pub tasks_completed_24h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceClassUsage {
    pub resource_class: ResourceClass,
    pub task_count: u32,
    pub total_task_minutes: f32,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub org_id: Uuid,
    pub month: String,
    pub task_count: u32,
    pub total_task_minutes: f32,
    pub cost_cents: i64,
    pub breakdown_by_resource_class: Vec<ResourceClassUsage>,
}

pub struct OrchestrationManager {
    conn: Arc<Mutex<Connection>>,
    /// Base cost in cents per task-minute for Standard resource class
    base_cost_cents_per_minute: f32,
}

impl OrchestrationManager {
    pub fn new(db_path: &str) -> Result<Self, OrchestrationError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                workflow_id TEXT,
                skill_ids TEXT NOT NULL DEFAULT '[]',
                input TEXT NOT NULL DEFAULT '{}',
                priority INTEGER NOT NULL DEFAULT 5,
                max_duration_secs INTEGER NOT NULL DEFAULT 3600,
                resource_class TEXT NOT NULL DEFAULT 'standard',
                submitted_at TEXT NOT NULL,
                status_state TEXT NOT NULL DEFAULT 'queued',
                status_json TEXT NOT NULL DEFAULT '{\"state\":\"queued\"}',
                logs TEXT NOT NULL DEFAULT '[]',
                metered_minutes REAL NOT NULL DEFAULT 0.0
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_org ON tasks(org_id, status_state);
            CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority DESC, submitted_at ASC);",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            base_cost_cents_per_minute: 2.0,
        })
    }

    pub fn submit_task(&self, spec: TaskSpec) -> Result<Uuid, OrchestrationError> {
        let id = spec.id;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tasks
             (id, org_id, workflow_id, skill_ids, input, priority, max_duration_secs, resource_class, submitted_at, status_state, status_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'queued', '{\"state\":\"queued\"}')",
            params![
                id.to_string(),
                spec.org_id.to_string(),
                spec.workflow_id,
                serde_json::to_string(&spec.skill_ids)?,
                serde_json::to_string(&spec.input)?,
                spec.priority as i64,
                spec.max_duration_secs as i64,
                spec.resource_class.as_str(),
                spec.submitted_at.to_rfc3339(),
            ],
        )?;
        tracing::info!(
            "Task {} submitted for org {} with priority {}",
            id,
            spec.org_id,
            spec.priority
        );
        Ok(id)
    }

    pub fn get_task(&self, id: &Uuid) -> Result<TaskRecord, OrchestrationError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, workflow_id, skill_ids, input, priority, max_duration_secs,
                    resource_class, submitted_at, status_json, logs, metered_minutes
             FROM tasks WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id.to_string()])?;
        let row = rows
            .next()?
            .ok_or_else(|| OrchestrationError::TaskNotFound(id.to_string()))?;

        let status: TaskStatus =
            serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or(TaskStatus::Queued);
        let logs: Vec<LogLine> =
            serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default();

        let spec = TaskSpec {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            org_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
            workflow_id: row.get(2)?,
            skill_ids: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            input: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            priority: row.get::<_, i64>(5)? as u8,
            max_duration_secs: row.get::<_, i64>(6)? as u64,
            resource_class: ResourceClass::from_str(&row.get::<_, String>(7)?),
            submitted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap_or_default()
                .with_timezone(&Utc),
        };

        Ok(TaskRecord {
            spec,
            status,
            logs,
            metered_minutes: row.get(11)?,
        })
    }

    pub fn cancel_task(&self, id: &Uuid) -> Result<(), OrchestrationError> {
        let conn = self.conn.lock().unwrap();
        let cancelled_json = serde_json::to_string(&TaskStatus::Cancelled)?;
        let updated = conn.execute(
            "UPDATE tasks SET status_state = 'cancelled', status_json = ?1
             WHERE id = ?2 AND status_state IN ('queued', 'running')",
            params![cancelled_json, id.to_string()],
        )?;
        if updated == 0 {
            return Err(OrchestrationError::TaskAlreadyCompleted);
        }
        Ok(())
    }

    pub fn list_tasks(
        &self,
        org_id: &Uuid,
        status_filter: Option<&str>,
    ) -> Result<Vec<TaskRecord>, OrchestrationError> {
        let conn = self.conn.lock().unwrap();
        let sql = if status_filter.is_some() {
            "SELECT id, org_id, workflow_id, skill_ids, input, priority, max_duration_secs,
                    resource_class, submitted_at, status_json, logs, metered_minutes
             FROM tasks WHERE org_id = ?1 AND status_state = ?2 ORDER BY priority DESC, submitted_at ASC"
        } else {
            "SELECT id, org_id, workflow_id, skill_ids, input, priority, max_duration_secs,
                    resource_class, submitted_at, status_json, logs, metered_minutes
             FROM tasks WHERE org_id = ?1 ORDER BY priority DESC, submitted_at ASC"
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(filter) = status_filter {
            stmt.query_map(params![org_id.to_string(), filter], Self::row_to_record)?
                .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![org_id.to_string()], Self::row_to_record)?
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    pub fn update_task_status(
        &self,
        id: &Uuid,
        status: TaskStatus,
        metered_minutes: f32,
    ) -> Result<(), OrchestrationError> {
        let conn = self.conn.lock().unwrap();
        let status_json = serde_json::to_string(&status)?;
        conn.execute(
            "UPDATE tasks SET status_state = ?1, status_json = ?2, metered_minutes = ?3 WHERE id = ?4",
            params![
                status.as_state_str(),
                status_json,
                metered_minutes,
                id.to_string(),
            ],
        )?;
        Ok(())
    }

    pub fn append_log(
        &self,
        id: &Uuid,
        level: &str,
        message: &str,
    ) -> Result<(), OrchestrationError> {
        let record = self.get_task(id)?;
        let mut logs = record.logs;
        logs.push(LogLine {
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
        });
        let logs_json = serde_json::to_string(&logs)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET logs = ?1 WHERE id = ?2",
            params![logs_json, id.to_string()],
        )?;
        Ok(())
    }

    pub fn get_worker_stats(&self) -> Result<WorkerStats, OrchestrationError> {
        let conn = self.conn.lock().unwrap();

        let queue_depth: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status_state = 'queued'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let busy_workers: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status_state = 'running'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let tasks_24h: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status_state = 'completed' AND submitted_at > datetime('now', '-24 hours')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let avg_duration: f32 = conn
            .query_row(
                "SELECT AVG(metered_minutes * 60.0) FROM tasks WHERE status_state = 'completed' AND metered_minutes > 0",
                [],
                |r| r.get::<_, Option<f64>>(0),
            )
            .unwrap_or(None)
            .unwrap_or(0.0) as f32;

        Ok(WorkerStats {
            total_workers: 10,
            busy_workers,
            queue_depth,
            avg_task_duration_secs: avg_duration,
            tasks_completed_24h: tasks_24h,
        })
    }

    pub fn meter_usage(&self, org_id: &Uuid, month: &str) -> Result<UsageReport, OrchestrationError> {
        let conn = self.conn.lock().unwrap();
        let period_start = format!("{}-01T00:00:00Z", month);
        let period_end = format!("{}-31T23:59:59Z", month);

        let mut stmt = conn.prepare(
            "SELECT resource_class, COUNT(*), SUM(metered_minutes)
             FROM tasks
             WHERE org_id = ?1 AND status_state = 'completed'
               AND submitted_at >= ?2 AND submitted_at <= ?3
             GROUP BY resource_class",
        )?;

        let rows = stmt
            .query_map(
                params![org_id.to_string(), period_start, period_end],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)? as u32,
                        row.get::<_, f64>(2)? as f32,
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        let mut breakdown = vec![];
        let mut total_tasks = 0u32;
        let mut total_minutes = 0.0f32;
        let mut total_cost = 0i64;

        for (class_str, count, minutes) in rows {
            let rc = ResourceClass::from_str(&class_str);
            let cost = (minutes * self.base_cost_cents_per_minute * rc.cost_multiplier()) as i64;
            total_tasks += count;
            total_minutes += minutes;
            total_cost += cost;
            breakdown.push(ResourceClassUsage {
                resource_class: rc,
                task_count: count,
                total_task_minutes: minutes,
                cost_cents: cost,
            });
        }

        Ok(UsageReport {
            org_id: *org_id,
            month: month.to_string(),
            task_count: total_tasks,
            total_task_minutes: total_minutes,
            cost_cents: total_cost,
            breakdown_by_resource_class: breakdown,
        })
    }

    fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<TaskRecord> {
        let status: TaskStatus =
            serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or(TaskStatus::Queued);
        let logs: Vec<LogLine> =
            serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default();
        let spec = TaskSpec {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            org_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
            workflow_id: row.get(2)?,
            skill_ids: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            input: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            priority: row.get::<_, i64>(5)? as u8,
            max_duration_secs: row.get::<_, i64>(6)? as u64,
            resource_class: ResourceClass::from_str(&row.get::<_, String>(7)?),
            submitted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap_or_default()
                .with_timezone(&Utc),
        };
        Ok(TaskRecord {
            spec,
            status,
            logs,
            metered_minutes: row.get(11)?,
        })
    }
}
