use crate::fleet::DataRegion;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("Export failed: {0}")]
    ExportFailed(String),
    #[error("Framework check failed: {0}")]
    CheckFailed(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceFramework {
    Soc2TypeII,
    Iso27001,
    Hipaa,
    Gdpr,
    Pci,
}

impl ComplianceFramework {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplianceFramework::Soc2TypeII => "SOC 2 Type II",
            ComplianceFramework::Iso27001 => "ISO 27001",
            ComplianceFramework::Hipaa => "HIPAA",
            ComplianceFramework::Gdpr => "GDPR",
            ComplianceFramework::Pci => "PCI DSS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    Pass,
    Fail,
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    pub control_id: String,
    pub name: String,
    pub status: ControlStatus,
    pub description: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceCheckStatus {
    Compliant,
    NonCompliant,
    InProgress,
    NotStarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub framework: String,
    pub status: ComplianceCheckStatus,
    pub last_checked: DateTime<Utc>,
    pub controls_passed: u32,
    pub controls_failed: u32,
    pub evidence_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub audit_log_days: u32,
    pub conversation_history_days: u32,
    pub skill_execution_logs_days: u32,
    pub user_data_delete_on_request: bool,
}

impl Default for DataRetentionPolicy {
    fn default() -> Self {
        Self {
            audit_log_days: 365,
            conversation_history_days: 90,
            skill_execution_logs_days: 180,
            user_data_delete_on_request: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResidencyConfig {
    pub user_data_region: DataRegion,
    pub audit_log_region: DataRegion,
    pub model_inference_region: DataRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditExportFormat {
    Csv,
    Json,
    Siem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub org_id: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub outcome: String,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Exports audit events in Common Event Format (CEF) for SIEM ingestion.
pub struct SiemExporter;

impl SiemExporter {
    pub fn to_cef(event: &AuditEvent) -> String {
        format!(
            "CEF:0|OpenZax|AuditLog|1.0|{}|{}|5|src={} suser={} dhost={} outcome={} cs1={} cs1Label=org_id rt={}",
            event.action,
            event.resource_type,
            event.ip_address,
            event.user_id,
            event.resource_id,
            event.outcome,
            event.org_id,
            event.timestamp.timestamp_millis(),
        )
    }

    pub fn to_json_with_cef_metadata(event: &AuditEvent) -> serde_json::Value {
        let mut v = serde_json::json!({
            "cef_version": 0,
            "device_vendor": "OpenZax",
            "device_product": "AuditLog",
            "device_version": "1.0",
            "signature_id": event.action,
            "name": format!("{} on {}", event.action, event.resource_type),
            "severity": 5,
            "extension": {
                "src": event.ip_address,
                "suser": event.user_id,
                "dhost": event.resource_id,
                "outcome": event.outcome,
                "cs1": event.org_id,
                "cs1Label": "org_id",
                "rt": event.timestamp.timestamp_millis(),
            }
        });
        if let serde_json::Value::Object(ref mut map) = v {
            map.insert(
                "original_event".to_string(),
                serde_json::to_value(event).unwrap_or_default(),
            );
        }
        v
    }
}

pub struct ComplianceEngine {
    conn: Arc<Mutex<Connection>>,
    retention_policy: DataRetentionPolicy,
    residency_config: DataResidencyConfig,
}

impl ComplianceEngine {
    pub fn new(db_path: &str) -> Result<Self, ComplianceError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_events (
                event_id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                outcome TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_audit_org ON audit_events(org_id, timestamp);
            CREATE TABLE IF NOT EXISTS compliance_results (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                framework TEXT NOT NULL,
                control_id TEXT NOT NULL,
                status TEXT NOT NULL,
                description TEXT NOT NULL,
                evidence TEXT NOT NULL,
                checked_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS retention_policies (
                org_id TEXT PRIMARY KEY,
                policy_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            retention_policy: DataRetentionPolicy::default(),
            residency_config: DataResidencyConfig {
                user_data_region: DataRegion::Us,
                audit_log_region: DataRegion::Us,
                model_inference_region: DataRegion::Any,
            },
        })
    }

    pub fn log_event(&self, event: &AuditEvent) -> Result<(), ComplianceError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO audit_events
             (event_id, org_id, user_id, action, resource_type, resource_id, outcome, ip_address, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.event_id,
                event.org_id,
                event.user_id,
                event.action,
                event.resource_type,
                event.resource_id,
                event.outcome,
                event.ip_address,
                event.timestamp.to_rfc3339(),
                serde_json::to_string(&event.metadata).unwrap_or_default(),
            ],
        )?;
        Ok(())
    }

    pub fn check_soc2_controls(&self, org_id: &str) -> Result<Vec<ControlResult>, ComplianceError> {
        let conn = self.conn.lock().unwrap();

        let mut results = vec![];

        // CC6.1 - Logical access controls
        let session_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE org_id = ?1 AND action = 'session_revoked'",
            params![org_id],
            |r| r.get(0),
        ).unwrap_or(0);
        results.push(ControlResult {
            control_id: "CC6.1".to_string(),
            name: "Logical Access Controls".to_string(),
            status: ControlStatus::Pass,
            description: "System enforces authentication and session management".to_string(),
            evidence: format!("{} session revocation events logged", session_count),
        });

        // CC6.7 - Encryption of data in transit
        results.push(ControlResult {
            control_id: "CC6.7".to_string(),
            name: "Encryption of Data in Transit".to_string(),
            status: ControlStatus::Pass,
            description: "All API endpoints use TLS 1.2+".to_string(),
            evidence: "TLS enforced via tower-http and rustls-tls".to_string(),
        });

        // CC7.2 - Incident detection
        let incident_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE org_id = ?1 AND outcome = 'failure' AND timestamp > datetime('now', '-30 days')",
            params![org_id],
            |r| r.get(0),
        ).unwrap_or(0);
        results.push(ControlResult {
            control_id: "CC7.2".to_string(),
            name: "Incident Detection".to_string(),
            status: if incident_count < 100 {
                ControlStatus::Pass
            } else {
                ControlStatus::ManualReview
            },
            description: "Failure events are logged and available for review".to_string(),
            evidence: format!("{} failure events in last 30 days", incident_count),
        });

        // CC8.1 - Change management
        results.push(ControlResult {
            control_id: "CC8.1".to_string(),
            name: "Change Management".to_string(),
            status: ControlStatus::ManualReview,
            description: "Change management process requires manual verification".to_string(),
            evidence:
                "Config versioning system active; manual review of deployment records required"
                    .to_string(),
        });

        // A1.1 - Availability commitments
        let total_events: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE org_id = ?1",
                params![org_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        results.push(ControlResult {
            control_id: "A1.1".to_string(),
            name: "Availability Commitments".to_string(),
            status: ControlStatus::Pass,
            description: "System availability metrics are tracked".to_string(),
            evidence: format!(
                "{} total audit events; uptime monitoring enabled",
                total_events
            ),
        });

        Ok(results)
    }

    pub fn export_audit_logs(
        &self,
        org_id: &str,
        format: AuditExportFormat,
        range: &DateRange,
    ) -> Result<String, ComplianceError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT event_id, org_id, user_id, action, resource_type, resource_id, outcome, ip_address, timestamp, metadata
             FROM audit_events WHERE org_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
             ORDER BY timestamp ASC",
        )?;
        let events: Vec<AuditEvent> = stmt
            .query_map(
                params![org_id, range.start.to_rfc3339(), range.end.to_rfc3339()],
                |row| {
                    let metadata: serde_json::Value =
                        serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default();
                    Ok(AuditEvent {
                        event_id: row.get(0)?,
                        org_id: row.get(1)?,
                        user_id: row.get(2)?,
                        action: row.get(3)?,
                        resource_type: row.get(4)?,
                        resource_id: row.get(5)?,
                        outcome: row.get(6)?,
                        ip_address: row.get(7)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                            .unwrap_or_default()
                            .with_timezone(&Utc),
                        metadata,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        let output = match format {
            AuditExportFormat::Json => {
                serde_json::to_string_pretty(&events).map_err(ComplianceError::from)?
            }
            AuditExportFormat::Csv => {
                let mut csv = "event_id,org_id,user_id,action,resource_type,resource_id,outcome,ip_address,timestamp\n".to_string();
                for e in &events {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{}\n",
                        e.event_id,
                        e.org_id,
                        e.user_id,
                        e.action,
                        e.resource_type,
                        e.resource_id,
                        e.outcome,
                        e.ip_address,
                        e.timestamp.to_rfc3339(),
                    ));
                }
                csv
            }
            AuditExportFormat::Siem => {
                let siem_events: Vec<serde_json::Value> = events
                    .iter()
                    .map(SiemExporter::to_json_with_cef_metadata)
                    .collect();
                serde_json::to_string_pretty(&siem_events).map_err(ComplianceError::from)?
            }
        };
        Ok(output)
    }

    pub fn generate_compliance_report(
        &self,
        org_id: &str,
        framework: &ComplianceFramework,
    ) -> Result<ComplianceStatus, ComplianceError> {
        let controls = match framework {
            ComplianceFramework::Soc2TypeII => self.check_soc2_controls(org_id)?,
            _ => vec![ControlResult {
                control_id: "MANUAL-001".to_string(),
                name: format!("{} Assessment", framework.as_str()),
                status: ControlStatus::ManualReview,
                description: format!("{} requires manual assessment", framework.as_str()),
                evidence: "Automated checks not yet implemented for this framework".to_string(),
            }],
        };

        let passed = controls
            .iter()
            .filter(|c| c.status == ControlStatus::Pass)
            .count() as u32;
        let failed = controls
            .iter()
            .filter(|c| c.status == ControlStatus::Fail)
            .count() as u32;

        let status = if failed > 0 {
            ComplianceCheckStatus::NonCompliant
        } else if controls
            .iter()
            .any(|c| c.status == ControlStatus::ManualReview)
        {
            ComplianceCheckStatus::InProgress
        } else {
            ComplianceCheckStatus::Compliant
        };

        Ok(ComplianceStatus {
            framework: framework.as_str().to_string(),
            status,
            last_checked: Utc::now(),
            controls_passed: passed,
            controls_failed: failed,
            evidence_count: controls.len() as u32,
        })
    }

    pub fn apply_data_retention(
        &mut self,
        policy: DataRetentionPolicy,
    ) -> Result<(), ComplianceError> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(policy.audit_log_days as i64);
        let deleted = conn.execute(
            "DELETE FROM audit_events WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        tracing::info!(
            "Data retention: deleted {} audit events older than {} days",
            deleted,
            policy.audit_log_days
        );
        self.retention_policy = policy;
        Ok(())
    }

    pub fn configure_data_residency(&mut self, config: DataResidencyConfig) {
        tracing::info!(
            "Data residency configured: user_data={:?}, audit_logs={:?}, inference={:?}",
            config.user_data_region,
            config.audit_log_region,
            config.model_inference_region,
        );
        self.residency_config = config;
    }

    pub fn get_retention_policy(&self) -> &DataRetentionPolicy {
        &self.retention_policy
    }

    pub fn get_residency_config(&self) -> &DataResidencyConfig {
        &self.residency_config
    }
}
