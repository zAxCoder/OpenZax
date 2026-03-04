use crate::auth::AuthProvider;
use crate::fleet::FleetPolicy;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OrgError {
    #[error("Organization not found: {0}")]
    OrgNotFound(String),
    #[error("Team not found: {0}")]
    TeamNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invite not found or expired")]
    InviteNotFound,
    #[error("Seat limit reached: {0}/{1}")]
    SeatLimitReached(u32, u32),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OrgPlan {
    Free,
    Pro,
    Enterprise {
        custom_limits: std::collections::HashMap<String, i64>,
    },
}

impl OrgPlan {
    pub fn max_seats(&self) -> u32 {
        match self {
            OrgPlan::Free => 5,
            OrgPlan::Pro => 50,
            OrgPlan::Enterprise { .. } => u32::MAX,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OrgPlan::Free => "free",
            OrgPlan::Pro => "pro",
            OrgPlan::Enterprise { .. } => "enterprise",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub plan: OrgPlan,
    pub max_seats: u32,
    pub used_seats: u32,
    pub created_at: DateTime<Utc>,
    pub sso_config: Option<AuthProvider>,
    pub fleet_policy: Option<FleetPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    Active,
    Invited,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMember {
    pub user_id: String,
    pub org_id: Uuid,
    pub team_ids: Vec<Uuid>,
    pub role: String,
    pub invited_by: String,
    pub joined_at: DateTime<Utc>,
    pub status: MemberStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgUsage {
    pub org_id: Uuid,
    pub used_seats: u32,
    pub max_seats: u32,
    pub skill_install_count: u32,
    pub task_count: u64,
    pub period_start: DateTime<Utc>,
}

pub struct OrgManager {
    conn: Arc<Mutex<Connection>>,
}

impl OrgManager {
    pub fn new(db_path: &str) -> Result<Self, OrgError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                plan TEXT NOT NULL DEFAULT 'free',
                plan_data TEXT,
                max_seats INTEGER NOT NULL DEFAULT 5,
                used_seats INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                sso_config TEXT,
                fleet_policy TEXT
            );
            CREATE TABLE IF NOT EXISTS teams (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                member_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (org_id) REFERENCES organizations(id)
            );
            CREATE TABLE IF NOT EXISTS org_members (
                user_id TEXT NOT NULL,
                org_id TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'developer',
                invited_by TEXT NOT NULL,
                joined_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'invited',
                PRIMARY KEY (user_id, org_id)
            );
            CREATE TABLE IF NOT EXISTS team_members (
                team_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                PRIMARY KEY (team_id, user_id)
            );
            CREATE TABLE IF NOT EXISTS invitations (
                token TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                email TEXT NOT NULL,
                role TEXT NOT NULL,
                invited_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS org_usage (
                org_id TEXT NOT NULL,
                metric TEXT NOT NULL,
                value INTEGER NOT NULL DEFAULT 0,
                period TEXT NOT NULL,
                PRIMARY KEY (org_id, metric, period)
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn create_org(
        &self,
        name: &str,
        slug: &str,
        plan: OrgPlan,
    ) -> Result<Organization, OrgError> {
        let org = Organization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            slug: slug.to_string(),
            max_seats: plan.max_seats(),
            used_seats: 0,
            created_at: Utc::now(),
            sso_config: None,
            fleet_policy: None,
            plan,
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO organizations (id, name, slug, plan, max_seats, used_seats, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                org.id.to_string(),
                org.name,
                org.slug,
                org.plan.as_str(),
                org.max_seats,
                org.used_seats,
                org.created_at.to_rfc3339(),
            ],
        )?;
        Ok(org)
    }

    pub fn get_org(&self, org_id: &Uuid) -> Result<Organization, OrgError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, slug, plan, max_seats, used_seats, created_at, sso_config, fleet_policy
             FROM organizations WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![org_id.to_string()])?;
        let row = rows
            .next()?
            .ok_or_else(|| OrgError::OrgNotFound(org_id.to_string()))?;

        let plan_str: String = row.get(3)?;
        let plan = match plan_str.as_str() {
            "pro" => OrgPlan::Pro,
            "enterprise" => OrgPlan::Enterprise {
                custom_limits: std::collections::HashMap::new(),
            },
            _ => OrgPlan::Free,
        };

        let sso_config: Option<AuthProvider> = row
            .get::<_, Option<String>>(7)?
            .and_then(|s| serde_json::from_str(&s).ok());
        let fleet_policy: Option<FleetPolicy> = row
            .get::<_, Option<String>>(8)?
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(Organization {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            name: row.get(1)?,
            slug: row.get(2)?,
            plan,
            max_seats: row.get(4)?,
            used_seats: row.get(5)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .unwrap_or_default()
                .with_timezone(&Utc),
            sso_config,
            fleet_policy,
        })
    }

    pub fn invite_user(
        &self,
        org_id: &Uuid,
        email: &str,
        role: &str,
        invited_by: &str,
    ) -> Result<String, OrgError> {
        let org = self.get_org(org_id)?;
        if org.used_seats >= org.max_seats {
            return Err(OrgError::SeatLimitReached(org.used_seats, org.max_seats));
        }

        let token = format!("inv_{}", Uuid::new_v4().simple());
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO invitations (token, org_id, email, role, invited_by, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                token,
                org_id.to_string(),
                email,
                role,
                invited_by,
                now.to_rfc3339(),
                expires_at.to_rfc3339(),
            ],
        )?;

        tracing::info!("Invitation created for {} to org {}", email, org_id);

        Ok(token)
    }

    pub fn accept_invite(&self, token: &str, user_id: &str) -> Result<OrgMember, OrgError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT token, org_id, email, role, invited_by, created_at, expires_at
             FROM invitations WHERE token = ?1",
        )?;
        let mut rows = stmt.query(params![token])?;
        let row = rows.next()?.ok_or(OrgError::InviteNotFound)?;

        let expires_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
            .unwrap_or_default()
            .with_timezone(&Utc);
        if Utc::now() > expires_at {
            return Err(OrgError::InviteNotFound);
        }

        let org_id_str: String = row.get(1)?;
        let role: String = row.get(3)?;
        let invited_by: String = row.get(4)?;
        let now = Utc::now();

        conn.execute(
            "INSERT OR REPLACE INTO org_members (user_id, org_id, role, invited_by, joined_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active')",
            params![user_id, org_id_str, role, invited_by, now.to_rfc3339()],
        )?;
        conn.execute(
            "UPDATE organizations SET used_seats = used_seats + 1 WHERE id = ?1",
            params![org_id_str],
        )?;
        conn.execute("DELETE FROM invitations WHERE token = ?1", params![token])?;

        Ok(OrgMember {
            user_id: user_id.to_string(),
            org_id: Uuid::parse_str(&org_id_str).unwrap_or_default(),
            team_ids: vec![],
            role,
            invited_by,
            joined_at: now,
            status: MemberStatus::Active,
        })
    }

    pub fn remove_user(&self, org_id: &Uuid, user_id: &str) -> Result<(), OrgError> {
        let conn = self.conn.lock().unwrap();
        let removed = conn.execute(
            "DELETE FROM org_members WHERE user_id = ?1 AND org_id = ?2",
            params![user_id, org_id.to_string()],
        )?;
        if removed > 0 {
            conn.execute(
                "UPDATE organizations SET used_seats = MAX(0, used_seats - 1) WHERE id = ?1",
                params![org_id.to_string()],
            )?;
            conn.execute(
                "DELETE FROM team_members WHERE user_id = ?1",
                params![user_id],
            )?;
        }
        Ok(())
    }

    pub fn create_team(
        &self,
        org_id: &Uuid,
        name: &str,
        description: &str,
    ) -> Result<Team, OrgError> {
        let team = Team {
            id: Uuid::new_v4(),
            org_id: *org_id,
            name: name.to_string(),
            description: description.to_string(),
            member_count: 0,
            created_at: Utc::now(),
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO teams (id, org_id, name, description, member_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                team.id.to_string(),
                team.org_id.to_string(),
                team.name,
                team.description,
                team.member_count,
                team.created_at.to_rfc3339(),
            ],
        )?;
        Ok(team)
    }

    pub fn add_to_team(&self, team_id: &Uuid, user_id: &str) -> Result<(), OrgError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO team_members (team_id, user_id) VALUES (?1, ?2)",
            params![team_id.to_string(), user_id],
        )?;
        conn.execute(
            "UPDATE teams SET member_count = member_count + 1 WHERE id = ?1",
            params![team_id.to_string()],
        )?;
        Ok(())
    }

    pub fn update_plan(&self, org_id: &Uuid, plan: OrgPlan) -> Result<(), OrgError> {
        let max_seats = plan.max_seats();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE organizations SET plan = ?1, max_seats = ?2 WHERE id = ?3",
            params![plan.as_str(), max_seats, org_id.to_string()],
        )?;
        Ok(())
    }

    pub fn get_usage(&self, org_id: &Uuid) -> Result<OrgUsage, OrgError> {
        let org = self.get_org(org_id)?;
        let conn = self.conn.lock().unwrap();

        let period = Utc::now().format("%Y-%m").to_string();
        let mut stmt =
            conn.prepare("SELECT metric, value FROM org_usage WHERE org_id = ?1 AND period = ?2")?;
        let rows = stmt.query_map(params![org_id.to_string(), period], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut skill_install_count = 0u32;
        let mut task_count = 0u64;
        for row in rows.flatten() {
            match row.0.as_str() {
                "skill_installs" => skill_install_count = row.1 as u32,
                "tasks" => task_count = row.1 as u64,
                _ => {}
            }
        }

        Ok(OrgUsage {
            org_id: *org_id,
            used_seats: org.used_seats,
            max_seats: org.max_seats,
            skill_install_count,
            task_count,
            period_start: Utc::now(),
        })
    }

    pub fn increment_usage(&self, org_id: &Uuid, metric: &str, by: i64) -> Result<(), OrgError> {
        let conn = self.conn.lock().unwrap();
        let period = Utc::now().format("%Y-%m").to_string();
        conn.execute(
            "INSERT INTO org_usage (org_id, metric, value, period) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(org_id, metric, period) DO UPDATE SET value = value + ?3",
            params![org_id.to_string(), metric, by, period],
        )?;
        Ok(())
    }
}
