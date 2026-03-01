use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("skill '{0}' is not quarantined")]
    NotQuarantined(String),
    #[error("skill '{0}' is already quarantined")]
    AlreadyQuarantined(String),
    #[error("skill '{0}' is whitelisted and cannot be quarantined without removing the whitelist first")]
    Whitelisted(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

impl ReviewStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ReviewStatus::Pending => "Pending",
            ReviewStatus::Approved => "Approved",
            ReviewStatus::Rejected => "Rejected",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "Approved" => ReviewStatus::Approved,
            "Rejected" => ReviewStatus::Rejected,
            _ => ReviewStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuarantineState {
    Active,
    Quarantined {
        reason: String,
        timestamp: DateTime<Utc>,
    },
    Whitelisted,
}

impl QuarantineState {
    #[allow(dead_code)]
    fn kind_str(&self) -> &'static str {
        match self {
            QuarantineState::Active => "Active",
            QuarantineState::Quarantined { .. } => "Quarantined",
            QuarantineState::Whitelisted => "Whitelisted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRecord {
    pub skill_id: String,
    pub reason: String,
    pub quarantined_at: DateTime<Utc>,
    pub review_status: ReviewStatus,
    pub reviewer_notes: Option<String>,
}

pub struct QuarantineManager {
    conn: Arc<Mutex<Connection>>,
}

impl QuarantineManager {
    /// Opens (or creates) the quarantine store at `db_path`.
    pub fn open(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        Self::init_db(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Creates an in-memory store (useful for tests).
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
             CREATE TABLE IF NOT EXISTS quarantine (
                 skill_id        TEXT PRIMARY KEY NOT NULL,
                 state_kind      TEXT NOT NULL,
                 reason          TEXT,
                 quarantined_at  TEXT,
                 review_status   TEXT NOT NULL DEFAULT 'Pending',
                 reviewer_notes  TEXT
             );",
        )?;
        Ok(())
    }

    /// Quarantines a skill with the given reason.
    ///
    /// Returns `Err(AlreadyQuarantined)` if already quarantined.
    /// Returns `Err(Whitelisted)` if the skill is on the whitelist.
    pub fn quarantine(&self, skill_id: &str, reason: impl Into<String>) -> Result<()> {
        let current = self.get_status(skill_id)?;
        match current {
            QuarantineState::Quarantined { .. } => {
                return Err(Error::AlreadyQuarantined(skill_id.to_owned()))
            }
            QuarantineState::Whitelisted => {
                return Err(Error::Whitelisted(skill_id.to_owned()))
            }
            QuarantineState::Active => {}
        }

        let reason = reason.into();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO quarantine (skill_id, state_kind, reason, quarantined_at, review_status)
             VALUES (?1, 'Quarantined', ?2, ?3, 'Pending')
             ON CONFLICT(skill_id) DO UPDATE SET
                 state_kind     = 'Quarantined',
                 reason         = excluded.reason,
                 quarantined_at = excluded.quarantined_at,
                 review_status  = 'Pending',
                 reviewer_notes = NULL",
            params![skill_id, reason, now],
        )?;
        tracing::warn!(skill_id = %skill_id, reason = %reason, "Skill quarantined");
        Ok(())
    }

    /// Lifts the quarantine on a skill, returning it to `Active` state.
    pub fn lift_quarantine(
        &self,
        skill_id: &str,
        reviewer_notes: Option<String>,
    ) -> Result<()> {
        let current = self.get_status(skill_id)?;
        if !matches!(current, QuarantineState::Quarantined { .. }) {
            return Err(Error::NotQuarantined(skill_id.to_owned()));
        }

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE quarantine SET
                 state_kind     = 'Active',
                 review_status  = 'Approved',
                 reviewer_notes = ?2
             WHERE skill_id = ?1",
            params![skill_id, reviewer_notes],
        )?;
        tracing::info!(skill_id = %skill_id, "Quarantine lifted");
        Ok(())
    }

    /// Whitelists a skill so it cannot be quarantined in the future without
    /// explicit removal from the whitelist.
    pub fn whitelist(&self, skill_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO quarantine (skill_id, state_kind, review_status)
             VALUES (?1, 'Whitelisted', 'Approved')
             ON CONFLICT(skill_id) DO UPDATE SET
                 state_kind    = 'Whitelisted',
                 review_status = 'Approved'",
            params![skill_id],
        )?;
        tracing::info!(skill_id = %skill_id, "Skill whitelisted");
        Ok(())
    }

    /// Removes a skill from the whitelist, returning it to `Active` state.
    pub fn remove_whitelist(&self, skill_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE quarantine SET state_kind = 'Active'
             WHERE skill_id = ?1 AND state_kind = 'Whitelisted'",
            params![skill_id],
        )?;
        Ok(())
    }

    /// Returns the current `QuarantineState` of a skill.
    /// If the skill has no record, it is considered `Active`.
    pub fn get_status(&self, skill_id: &str) -> Result<QuarantineState> {
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<(String, Option<String>, Option<String>)> =
            conn.query_row(
                "SELECT state_kind, reason, quarantined_at FROM quarantine WHERE skill_id = ?1",
                params![skill_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            );

        match result {
            Ok((kind, reason, quarantined_at)) => match kind.as_str() {
                "Quarantined" => {
                    let timestamp = quarantined_at
                        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                        .unwrap_or_else(Utc::now);
                    Ok(QuarantineState::Quarantined {
                        reason: reason.unwrap_or_default(),
                        timestamp,
                    })
                }
                "Whitelisted" => Ok(QuarantineState::Whitelisted),
                _ => Ok(QuarantineState::Active),
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(QuarantineState::Active),
            Err(e) => Err(Error::Database(e)),
        }
    }

    /// Returns all `QuarantineRecord`s that are currently quarantined and
    /// awaiting review (status = `Pending`).
    pub fn pending_reviews(&self) -> Result<Vec<QuarantineRecord>> {
        self.list_by_state("Quarantined", Some(ReviewStatus::Pending))
    }

    /// Returns all currently quarantined skills regardless of review status.
    pub fn quarantined_skills(&self) -> Result<Vec<QuarantineRecord>> {
        self.list_by_state("Quarantined", None)
    }

    fn list_by_state(
        &self,
        state_kind: &str,
        review_filter: Option<ReviewStatus>,
    ) -> Result<Vec<QuarantineRecord>> {
        let conn = self.conn.lock().unwrap();
        let sql = if review_filter.is_some() {
            "SELECT skill_id, reason, quarantined_at, review_status, reviewer_notes
             FROM quarantine WHERE state_kind = ?1 AND review_status = ?2"
        } else {
            "SELECT skill_id, reason, quarantined_at, review_status, reviewer_notes
             FROM quarantine WHERE state_kind = ?1"
        };

        let review_str = review_filter
            .as_ref()
            .map(|r| r.as_str())
            .unwrap_or("Pending");

        let mut stmt = conn.prepare(sql)?;
        let params_slice: &[&dyn rusqlite::ToSql] = if review_filter.is_some() {
            &[&state_kind, &review_str]
        } else {
            &[&state_kind]
        };

        let records = stmt
            .query_map(params_slice, |row| {
                let ts_str: Option<String> = row.get(2)?;
                let quarantined_at = ts_str
                    .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                    .unwrap_or_else(Utc::now);
                let status_str: String = row.get(3)?;
                Ok(QuarantineRecord {
                    skill_id: row.get(0)?,
                    reason: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    quarantined_at,
                    review_status: ReviewStatus::from_str(&status_str),
                    reviewer_notes: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(records)
    }

    /// Updates the review status and notes for a quarantined skill.
    pub fn update_review(
        &self,
        skill_id: &str,
        status: ReviewStatus,
        notes: Option<String>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE quarantine SET review_status = ?2, reviewer_notes = ?3
             WHERE skill_id = ?1",
            params![skill_id, status.as_str(), notes],
        )?;
        Ok(())
    }
}
