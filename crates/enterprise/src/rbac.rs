use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    SuperAdmin,
    OrgAdmin,
    TeamAdmin,
    Developer,
    Viewer,
    Custom(String),
}

impl Role {
    pub fn as_str(&self) -> String {
        match self {
            Role::SuperAdmin => "super_admin".to_string(),
            Role::OrgAdmin => "org_admin".to_string(),
            Role::TeamAdmin => "team_admin".to_string(),
            Role::Developer => "developer".to_string(),
            Role::Viewer => "viewer".to_string(),
            Role::Custom(name) => format!("custom:{}", name),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "super_admin" => Role::SuperAdmin,
            "org_admin" => Role::OrgAdmin,
            "team_admin" => Role::TeamAdmin,
            "developer" => Role::Developer,
            "viewer" => Role::Viewer,
            other => {
                if let Some(name) = other.strip_prefix("custom:") {
                    Role::Custom(name.to_string())
                } else {
                    Role::Custom(other.to_string())
                }
            }
        }
    }

    pub fn default_permissions(&self) -> HashSet<Permission> {
        match self {
            Role::SuperAdmin => Permission::all().into_iter().collect(),
            Role::OrgAdmin => [
                Permission::ManageOrg,
                Permission::ManageTeams,
                Permission::ManageUsers,
                Permission::ManageSkills,
                Permission::InstallSkills,
                Permission::ExecuteSkills,
                Permission::ViewAuditLogs,
                Permission::ExportAuditLogs,
                Permission::ManageFleet,
                Permission::ConfigurePolicy,
                Permission::ViewDashboard,
                Permission::ManageApiKeys,
                Permission::ManageBilling,
            ]
            .into_iter()
            .collect(),
            Role::TeamAdmin => [
                Permission::ManageTeams,
                Permission::ManageUsers,
                Permission::InstallSkills,
                Permission::ExecuteSkills,
                Permission::ViewDashboard,
                Permission::ManageApiKeys,
            ]
            .into_iter()
            .collect(),
            Role::Developer => [
                Permission::InstallSkills,
                Permission::ExecuteSkills,
                Permission::ViewDashboard,
                Permission::ManageApiKeys,
            ]
            .into_iter()
            .collect(),
            Role::Viewer => [Permission::ViewDashboard].into_iter().collect(),
            Role::Custom(_) => HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ManageOrg,
    ManageTeams,
    ManageUsers,
    ManageSkills,
    InstallSkills,
    ExecuteSkills,
    ViewAuditLogs,
    ExportAuditLogs,
    ManageFleet,
    ConfigurePolicy,
    ViewDashboard,
    ManageApiKeys,
    ManageBilling,
}

impl Permission {
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::ManageOrg,
            Permission::ManageTeams,
            Permission::ManageUsers,
            Permission::ManageSkills,
            Permission::InstallSkills,
            Permission::ExecuteSkills,
            Permission::ViewAuditLogs,
            Permission::ExportAuditLogs,
            Permission::ManageFleet,
            Permission::ConfigurePolicy,
            Permission::ViewDashboard,
            Permission::ManageApiKeys,
            Permission::ManageBilling,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ManageOrg => "manage_org",
            Permission::ManageTeams => "manage_teams",
            Permission::ManageUsers => "manage_users",
            Permission::ManageSkills => "manage_skills",
            Permission::InstallSkills => "install_skills",
            Permission::ExecuteSkills => "execute_skills",
            Permission::ViewAuditLogs => "view_audit_logs",
            Permission::ExportAuditLogs => "export_audit_logs",
            Permission::ManageFleet => "manage_fleet",
            Permission::ConfigurePolicy => "configure_policy",
            Permission::ViewDashboard => "view_dashboard",
            Permission::ManageApiKeys => "manage_api_keys",
            Permission::ManageBilling => "manage_billing",
        }
    }

    pub fn from_str(s: &str) -> Option<Permission> {
        match s {
            "manage_org" => Some(Permission::ManageOrg),
            "manage_teams" => Some(Permission::ManageTeams),
            "manage_users" => Some(Permission::ManageUsers),
            "manage_skills" => Some(Permission::ManageSkills),
            "install_skills" => Some(Permission::InstallSkills),
            "execute_skills" => Some(Permission::ExecuteSkills),
            "view_audit_logs" => Some(Permission::ViewAuditLogs),
            "export_audit_logs" => Some(Permission::ExportAuditLogs),
            "manage_fleet" => Some(Permission::ManageFleet),
            "configure_policy" => Some(Permission::ConfigurePolicy),
            "view_dashboard" => Some(Permission::ViewDashboard),
            "manage_api_keys" => Some(Permission::ManageApiKeys),
            "manage_billing" => Some(Permission::ManageBilling),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub role_name: String,
    pub permissions: Vec<Permission>,
    pub description: String,
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: String,
    pub organization_id: String,
    pub team_id: Option<String>,
    pub role: Role,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

pub struct RbacEngine {
    conn: Arc<Mutex<Connection>>,
    /// Custom permission overrides for custom roles: role_name -> Vec<Permission>
    custom_role_permissions: std::collections::HashMap<String, Vec<Permission>>,
}

impl RbacEngine {
    pub fn new(db_path: &str) -> Result<Self, RbacError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS user_roles (
                user_id TEXT NOT NULL,
                organization_id TEXT NOT NULL,
                team_id TEXT,
                role TEXT NOT NULL,
                granted_by TEXT NOT NULL,
                granted_at TEXT NOT NULL,
                PRIMARY KEY (user_id, organization_id)
            );
            CREATE TABLE IF NOT EXISTS custom_role_permissions (
                role_name TEXT NOT NULL,
                permission TEXT NOT NULL,
                PRIMARY KEY (role_name, permission)
            );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            custom_role_permissions: std::collections::HashMap::new(),
        })
    }

    pub fn assign_role(
        &self,
        user_id: &str,
        org_id: &str,
        role: Role,
        granted_by: &str,
        team_id: Option<&str>,
    ) -> Result<(), RbacError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO user_roles
             (user_id, organization_id, team_id, role, granted_by, granted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user_id,
                org_id,
                team_id,
                role.as_str(),
                granted_by,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn revoke_role(&self, user_id: &str, org_id: &str) -> Result<(), RbacError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM user_roles WHERE user_id = ?1 AND organization_id = ?2",
            params![user_id, org_id],
        )?;
        Ok(())
    }

    pub fn has_permission(
        &self,
        user_id: &str,
        org_id: &str,
        permission: &Permission,
    ) -> Result<bool, RbacError> {
        let roles = self.get_user_roles(user_id)?;
        for user_role in &roles {
            if user_role.organization_id != org_id {
                continue;
            }
            let perms = self.permissions_for_role(&user_role.role);
            if perms.contains(permission) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_user_roles(&self, user_id: &str) -> Result<Vec<UserRole>, RbacError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT user_id, organization_id, team_id, role, granted_by, granted_at
             FROM user_roles WHERE user_id = ?1",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(UserRole {
                user_id: row.get(0)?,
                organization_id: row.get(1)?,
                team_id: row.get(2)?,
                role: Role::from_str(&row.get::<_, String>(3)?),
                granted_by: row.get(4)?,
                granted_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(RbacError::from)
    }

    pub fn list_org_users(&self, org_id: &str) -> Result<Vec<(String, Role)>, RbacError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT user_id, role FROM user_roles WHERE organization_id = ?1",
        )?;
        let rows = stmt.query_map(params![org_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let pairs = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(pairs
            .into_iter()
            .map(|(uid, role_str)| (uid, Role::from_str(&role_str)))
            .collect())
    }

    pub fn define_custom_role(
        &mut self,
        role_name: &str,
        permissions: Vec<Permission>,
    ) -> Result<(), RbacError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM custom_role_permissions WHERE role_name = ?1",
            params![role_name],
        )?;
        for perm in &permissions {
            conn.execute(
                "INSERT INTO custom_role_permissions (role_name, permission) VALUES (?1, ?2)",
                params![role_name, perm.as_str()],
            )?;
        }
        self.custom_role_permissions
            .insert(role_name.to_string(), permissions);
        Ok(())
    }

    fn permissions_for_role(&self, role: &Role) -> HashSet<Permission> {
        match role {
            Role::Custom(name) => {
                if let Some(perms) = self.custom_role_permissions.get(name) {
                    perms.iter().cloned().collect()
                } else {
                    HashSet::new()
                }
            }
            _ => role.default_permissions(),
        }
    }
}

/// Middleware-like enforcer that checks permissions before allowing operations.
pub struct PolicyEnforcer {
    rbac: Arc<RbacEngine>,
}

impl PolicyEnforcer {
    pub fn new(rbac: Arc<RbacEngine>) -> Self {
        Self { rbac }
    }

    pub fn enforce(
        &self,
        user_id: &str,
        org_id: &str,
        permission: &Permission,
    ) -> Result<(), RbacError> {
        let allowed = self.rbac.has_permission(user_id, org_id, permission)?;
        if !allowed {
            return Err(RbacError::PermissionDenied(format!(
                "User '{}' lacks {:?} in org '{}'",
                user_id, permission, org_id
            )));
        }
        Ok(())
    }

    pub fn enforce_any(
        &self,
        user_id: &str,
        org_id: &str,
        permissions: &[Permission],
    ) -> Result<(), RbacError> {
        for perm in permissions {
            if self.rbac.has_permission(user_id, org_id, perm)? {
                return Ok(());
            }
        }
        Err(RbacError::PermissionDenied(format!(
            "User '{}' lacks all of {:?} in org '{}'",
            user_id, permissions, org_id
        )))
    }

    pub fn enforce_all(
        &self,
        user_id: &str,
        org_id: &str,
        permissions: &[Permission],
    ) -> Result<(), RbacError> {
        for perm in permissions {
            self.enforce(user_id, org_id, perm)?;
        }
        Ok(())
    }
}
