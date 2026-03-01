use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum FleetError {
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),
    #[error("Policy validation failed: {0}")]
    PolicyViolation(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataRegion {
    Us,
    Eu,
    Ap,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetPolicy {
    pub required_skills: Vec<String>,
    pub blocked_skills: Vec<String>,
    pub permission_overrides: HashMap<String, Vec<String>>,
    pub max_skills_per_agent: u32,
    pub allowed_model_providers: Vec<String>,
    pub data_residency: DataRegion,
}

impl Default for FleetPolicy {
    fn default() -> Self {
        Self {
            required_skills: vec![],
            blocked_skills: vec![],
            permission_overrides: HashMap::new(),
            max_skills_per_agent: 100,
            allowed_model_providers: vec![],
            data_residency: DataRegion::Any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointStatus {
    Online,
    Offline,
    Degraded,
}

impl EndpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EndpointStatus::Online => "online",
            EndpointStatus::Offline => "offline",
            EndpointStatus::Degraded => "degraded",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "online" => EndpointStatus::Online,
            "degraded" => EndpointStatus::Degraded,
            _ => EndpointStatus::Offline,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEndpoint {
    pub id: Uuid,
    pub org_id: Uuid,
    pub hostname: String,
    pub ip_address: String,
    pub os: String,
    pub version: String,
    pub status: EndpointStatus,
    pub last_seen: DateTime<Utc>,
    pub installed_skills: Vec<String>,
    pub active_policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetHealthReport {
    pub total_endpoints: u32,
    pub online: u32,
    pub offline: u32,
    pub degraded: u32,
    pub outdated_version: u32,
    pub policy_violations: u32,
    pub recent_incidents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointFilter {
    pub status: Option<EndpointStatus>,
    pub version: Option<String>,
    pub skill: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version_id: Uuid,
    pub org_id: Uuid,
    pub config_json: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
}

pub struct ConfigVersioning {
    conn: Arc<Mutex<Connection>>,
}

impl ConfigVersioning {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self, FleetError> {
        {
            let c = conn.lock().unwrap();
            c.execute_batch(
                "CREATE TABLE IF NOT EXISTS config_versions (
                    version_id TEXT PRIMARY KEY,
                    org_id TEXT NOT NULL,
                    config_json TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    created_by TEXT NOT NULL,
                    description TEXT NOT NULL
                );",
            )?;
        }
        Ok(Self { conn })
    }

    pub fn save_version(
        &self,
        org_id: &Uuid,
        config: &FleetPolicy,
        created_by: &str,
        description: &str,
    ) -> Result<ConfigVersion, FleetError> {
        let version = ConfigVersion {
            version_id: Uuid::new_v4(),
            org_id: *org_id,
            config_json: serde_json::to_string(config)?,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            description: description.to_string(),
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO config_versions (version_id, org_id, config_json, created_at, created_by, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                version.version_id.to_string(),
                version.org_id.to_string(),
                version.config_json,
                version.created_at.to_rfc3339(),
                version.created_by,
                version.description,
            ],
        )?;
        Ok(version)
    }

    pub fn list_versions(&self, org_id: &Uuid) -> Result<Vec<ConfigVersion>, FleetError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT version_id, org_id, config_json, created_at, created_by, description
             FROM config_versions WHERE org_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id.to_string()], |row| {
            Ok(ConfigVersion {
                version_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                org_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                config_json: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                created_by: row.get(4)?,
                description: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(FleetError::from)
    }

    pub fn rollback(&self, org_id: &Uuid, version_id: &Uuid) -> Result<FleetPolicy, FleetError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT config_json FROM config_versions WHERE org_id = ?1 AND version_id = ?2",
        )?;
        let mut rows = stmt.query(params![org_id.to_string(), version_id.to_string()])?;
        let row = rows
            .next()?
            .ok_or_else(|| FleetError::EndpointNotFound(version_id.to_string()))?;
        let config_json: String = row.get(0)?;
        let policy: FleetPolicy = serde_json::from_str(&config_json)?;
        Ok(policy)
    }

    pub fn diff(
        &self,
        version_a: &Uuid,
        version_b: &Uuid,
    ) -> Result<Vec<String>, FleetError> {
        let conn = self.conn.lock().unwrap();
        let get_config = |vid: &Uuid| -> Result<serde_json::Value, FleetError> {
            let mut stmt = conn
                .prepare("SELECT config_json FROM config_versions WHERE version_id = ?1")
                .map_err(FleetError::from)?;
            let mut rows = stmt.query(params![vid.to_string()]).map_err(FleetError::from)?;
            let row = rows
                .next()
                .map_err(FleetError::from)?
                .ok_or_else(|| FleetError::EndpointNotFound(vid.to_string()))?;
            let json: String = row.get(0).map_err(FleetError::from)?;
            serde_json::from_str(&json).map_err(FleetError::from)
        };

        let a = get_config(version_a)?;
        let b = get_config(version_b)?;

        let mut diffs = vec![];
        if let (Some(ao), Some(bo)) = (a.as_object(), b.as_object()) {
            for key in ao.keys().chain(bo.keys()) {
                let va = ao.get(key);
                let vb = bo.get(key);
                if va != vb {
                    diffs.push(format!(
                        "Field '{}': {:?} -> {:?}",
                        key,
                        va.map(|v| v.to_string()).unwrap_or_default(),
                        vb.map(|v| v.to_string()).unwrap_or_default()
                    ));
                }
            }
        }
        Ok(diffs)
    }
}

pub struct FleetManager {
    conn: Arc<Mutex<Connection>>,
    pub config_versioning: ConfigVersioning,
    current_version: String,
}

impl FleetManager {
    pub fn new(db_path: &str, current_version: &str) -> Result<Self, FleetError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_endpoints (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                hostname TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                os TEXT NOT NULL,
                version TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'offline',
                last_seen TEXT NOT NULL,
                installed_skills TEXT NOT NULL DEFAULT '[]',
                active_policies TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS pending_deployments (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                skill_id TEXT NOT NULL,
                target_endpoints TEXT NOT NULL,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending'
            );
            CREATE TABLE IF NOT EXISTS fleet_incidents (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                endpoint_id TEXT,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )?;
        let arc_conn = Arc::new(Mutex::new(conn));
        let config_versioning = ConfigVersioning::new(arc_conn.clone())?;
        Ok(Self {
            conn: arc_conn,
            config_versioning,
            current_version: current_version.to_string(),
        })
    }

    pub fn register_endpoint(
        &self,
        org_id: &Uuid,
        hostname: &str,
        ip_address: &str,
        os: &str,
        version: &str,
    ) -> Result<AgentEndpoint, FleetError> {
        let endpoint = AgentEndpoint {
            id: Uuid::new_v4(),
            org_id: *org_id,
            hostname: hostname.to_string(),
            ip_address: ip_address.to_string(),
            os: os.to_string(),
            version: version.to_string(),
            status: EndpointStatus::Online,
            last_seen: Utc::now(),
            installed_skills: vec![],
            active_policies: vec![],
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agent_endpoints
             (id, org_id, hostname, ip_address, os, version, status, last_seen, installed_skills, active_policies)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                endpoint.id.to_string(),
                endpoint.org_id.to_string(),
                endpoint.hostname,
                endpoint.ip_address,
                endpoint.os,
                endpoint.version,
                endpoint.status.as_str(),
                endpoint.last_seen.to_rfc3339(),
                serde_json::to_string(&endpoint.installed_skills).unwrap_or_default(),
                serde_json::to_string(&endpoint.active_policies).unwrap_or_default(),
            ],
        )?;
        tracing::info!("Registered endpoint {} for org {}", endpoint.id, org_id);
        Ok(endpoint)
    }

    pub fn heartbeat(
        &self,
        endpoint_id: &Uuid,
        status: EndpointStatus,
    ) -> Result<(), FleetError> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE agent_endpoints SET status = ?1, last_seen = ?2 WHERE id = ?3",
            params![
                status.as_str(),
                Utc::now().to_rfc3339(),
                endpoint_id.to_string(),
            ],
        )?;
        if updated == 0 {
            return Err(FleetError::EndpointNotFound(endpoint_id.to_string()));
        }
        Ok(())
    }

    pub fn get_endpoint(&self, id: &Uuid) -> Result<AgentEndpoint, FleetError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, hostname, ip_address, os, version, status, last_seen, installed_skills, active_policies
             FROM agent_endpoints WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id.to_string()])?;
        let row = rows
            .next()?
            .ok_or_else(|| FleetError::EndpointNotFound(id.to_string()))?;
        self.row_to_endpoint(row)
    }

    pub fn list_endpoints(
        &self,
        org_id: &Uuid,
        filter: Option<EndpointFilter>,
    ) -> Result<Vec<AgentEndpoint>, FleetError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, hostname, ip_address, os, version, status, last_seen, installed_skills, active_policies
             FROM agent_endpoints WHERE org_id = ?1",
        )?;
        let rows = stmt.query_map(params![org_id.to_string()], |row| {
            let id = Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default();
            let oid = Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default();
            let installed_skills: Vec<String> =
                serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default();
            let active_policies: Vec<String> =
                serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default();
            Ok(AgentEndpoint {
                id,
                org_id: oid,
                hostname: row.get(2)?,
                ip_address: row.get(3)?,
                os: row.get(4)?,
                version: row.get(5)?,
                status: EndpointStatus::from_str(&row.get::<_, String>(6)?),
                last_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                installed_skills,
                active_policies,
            })
        })?;

        let mut endpoints: Vec<AgentEndpoint> =
            rows.collect::<Result<Vec<_>, _>>().map_err(FleetError::from)?;

        if let Some(f) = filter {
            if let Some(status) = f.status {
                endpoints.retain(|e| e.status == status);
            }
            if let Some(version) = f.version {
                endpoints.retain(|e| e.version == version);
            }
            if let Some(skill) = f.skill {
                endpoints.retain(|e| e.installed_skills.contains(&skill));
            }
        }
        Ok(endpoints)
    }

    pub fn deploy_skill(
        &self,
        org_id: &Uuid,
        skill_id: &str,
        target_endpoints: Vec<Uuid>,
    ) -> Result<Uuid, FleetError> {
        let deployment_id = Uuid::new_v4();
        let conn = self.conn.lock().unwrap();
        let targets_json = serde_json::to_string(&target_endpoints.iter().map(|u| u.to_string()).collect::<Vec<_>>())?;
        conn.execute(
            "INSERT INTO pending_deployments (id, org_id, skill_id, target_endpoints, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
            params![
                deployment_id.to_string(),
                org_id.to_string(),
                skill_id,
                targets_json,
                Utc::now().to_rfc3339(),
            ],
        )?;
        tracing::info!(
            "Scheduled deployment of skill '{}' to {} endpoints",
            skill_id,
            target_endpoints.len()
        );
        Ok(deployment_id)
    }

    pub fn apply_policy(
        &self,
        org_id: &Uuid,
        policy: FleetPolicy,
        applied_by: &str,
    ) -> Result<(), FleetError> {
        self.config_versioning.save_version(
            org_id,
            &policy,
            applied_by,
            "Policy applied via fleet manager",
        )?;
        let policy_id = format!("policy_{}", Uuid::new_v4().simple());
        let conn = self.conn.lock().unwrap();
        let policy_json = serde_json::to_string(&policy)?;
        conn.execute(
            "UPDATE agent_endpoints SET active_policies = json_insert(active_policies, '$[#]', ?1)
             WHERE org_id = ?2",
            params![policy_id, org_id.to_string()],
        )?;
        tracing::info!(
            "Applied policy to all endpoints in org {}: {:?}",
            org_id,
            policy_json
        );
        Ok(())
    }

    pub fn bulk_update(&self, org_id: &Uuid, target_version: &str) -> Result<usize, FleetError> {
        // In production, this would push an update command to all endpoints via a message queue
        let endpoints = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id FROM agent_endpoints WHERE org_id = ?1 AND version != ?2",
            )?;
            let ids: Vec<String> = stmt
                .query_map(params![org_id.to_string(), target_version], |row| {
                    row.get(0)
                })?
                .collect::<Result<Vec<_>, _>>()?;
            ids.len()
        };
        tracing::info!(
            "Scheduled bulk update to {} for {} endpoints in org {}",
            target_version,
            endpoints,
            org_id
        );
        Ok(endpoints)
    }

    pub fn health_dashboard(&self, org_id: &Uuid) -> Result<FleetHealthReport, FleetError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT status, version FROM agent_endpoints WHERE org_id = ?1",
        )?;
        let rows = stmt.query_map(params![org_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut total = 0u32;
        let mut online = 0u32;
        let mut offline = 0u32;
        let mut degraded = 0u32;
        let mut outdated = 0u32;

        for row in rows.flatten() {
            total += 1;
            match row.0.as_str() {
                "online" => online += 1,
                "offline" => offline += 1,
                "degraded" => degraded += 1,
                _ => offline += 1,
            }
            if row.1 != self.current_version {
                outdated += 1;
            }
        }

        let mut stmt2 = conn.prepare(
            "SELECT description FROM fleet_incidents WHERE org_id = ?1 ORDER BY created_at DESC LIMIT 10",
        )?;
        let recent_incidents: Vec<String> = stmt2
            .query_map(params![org_id.to_string()], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(FleetHealthReport {
            total_endpoints: total,
            online,
            offline,
            degraded,
            outdated_version: outdated,
            policy_violations: 0,
            recent_incidents,
        })
    }

    fn row_to_endpoint(&self, row: &rusqlite::Row) -> Result<AgentEndpoint, FleetError> {
        let installed_skills: Vec<String> =
            serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default();
        let active_policies: Vec<String> =
            serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default();
        Ok(AgentEndpoint {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
            org_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
            hostname: row.get(2)?,
            ip_address: row.get(3)?,
            os: row.get(4)?,
            version: row.get(5)?,
            status: EndpointStatus::from_str(&row.get::<_, String>(6)?),
            last_seen: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .unwrap_or_default()
                .with_timezone(&Utc),
            installed_skills,
            active_policies,
        })
    }
}
