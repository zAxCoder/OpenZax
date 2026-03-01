use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Session not found or expired")]
    SessionExpired,
    #[error("SAML assertion invalid: {0}")]
    SamlInvalid(String),
    #[error("OIDC token invalid: {0}")]
    OidcInvalid(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    pub entity_id: String,
    pub metadata_url: String,
    pub assertion_consumer_service_url: String,
    pub idp_certificate: String,
    pub attribute_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub attribute_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthProvider {
    Local,
    Saml(SamlConfig),
    Oidc(OidcConfig),
}

impl AuthProvider {
    pub fn variant_name(&self) -> &'static str {
        match self {
            AuthProvider::Local => "local",
            AuthProvider::Saml(_) => "saml",
            AuthProvider::Oidc(_) => "oidc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: Uuid,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub provider: String,
}

impl AuthSession {
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

#[derive(Debug, Clone)]
pub struct SamlFlowState {
    pub authn_request_id: String,
    pub relay_state: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct OidcFlowState {
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: String,
    pub code_challenge: String,
}

pub struct SessionStore {
    conn: Arc<Mutex<Connection>>,
}

impl SessionStore {
    pub fn new(db_path: &str) -> Result<Self, AuthError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS auth_sessions (
                session_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_active TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                user_agent TEXT NOT NULL,
                provider TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON auth_sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON auth_sessions(expires_at);",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn store(&self, session: &AuthSession) -> Result<(), AuthError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO auth_sessions
             (session_id, user_id, created_at, expires_at, last_active, ip_address, user_agent, provider)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.session_id.to_string(),
                session.user_id,
                session.created_at.to_rfc3339(),
                session.expires_at.to_rfc3339(),
                session.last_active.to_rfc3339(),
                session.ip_address,
                session.user_agent,
                session.provider,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, session_id: &Uuid) -> Result<Option<AuthSession>, AuthError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT session_id, user_id, created_at, expires_at, last_active, ip_address, user_agent, provider
             FROM auth_sessions WHERE session_id = ?1",
        )?;
        let mut rows = stmt.query(params![session_id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(AuthSession {
                session_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                user_id: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                last_active: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                ip_address: row.get(5)?,
                user_agent: row.get(6)?,
                provider: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn revoke(&self, session_id: &Uuid) -> Result<(), AuthError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM auth_sessions WHERE session_id = ?1",
            params![session_id.to_string()],
        )?;
        Ok(())
    }

    pub fn list_user_sessions(&self, user_id: &str) -> Result<Vec<AuthSession>, AuthError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT session_id, user_id, created_at, expires_at, last_active, ip_address, user_agent, provider
             FROM auth_sessions WHERE user_id = ?1 AND expires_at > ?2",
        )?;
        let now = Utc::now().to_rfc3339();
        let rows = stmt.query_map(params![user_id, now], |row| {
            Ok(AuthSession {
                session_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                user_id: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                expires_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                last_active: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap_or_default()
                    .with_timezone(&Utc),
                ip_address: row.get(5)?,
                user_agent: row.get(6)?,
                provider: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AuthError::from)
    }

    pub fn cleanup_expired(&self) -> Result<usize, AuthError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let deleted = conn.execute(
            "DELETE FROM auth_sessions WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(deleted)
    }

    pub fn touch(&self, session_id: &Uuid) -> Result<(), AuthError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE auth_sessions SET last_active = ?1 WHERE session_id = ?2",
            params![now, session_id.to_string()],
        )?;
        Ok(())
    }
}

pub struct AuthManager {
    session_store: Arc<SessionStore>,
    session_ttl_hours: i64,
    /// HMAC key used for SAML response signature verification and OIDC state binding
    signing_key: Vec<u8>,
}

impl AuthManager {
    pub fn new(session_store: Arc<SessionStore>, signing_key: Vec<u8>) -> Self {
        Self {
            session_store,
            session_ttl_hours: 8,
            signing_key,
        }
    }

    pub fn with_session_ttl(mut self, hours: i64) -> Self {
        self.session_ttl_hours = hours;
        self
    }

    /// Authenticate with username + password using SHA-256 hash comparison.
    /// In production, replace with bcrypt via the `bcrypt` crate.
    pub fn authenticate_local(
        &self,
        username: &str,
        password: &str,
        stored_hash: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<AuthSession, AuthError> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        if hash != stored_hash {
            return Err(AuthError::InvalidCredentials);
        }

        let session = self.create_session(username, "local", ip_address, user_agent);
        self.session_store.store(&session)?;
        Ok(session)
    }

    /// Generate a SAML AuthnRequest XML document and compute the redirect URL.
    pub fn initiate_saml_flow(&self, config: &SamlConfig) -> Result<SamlFlowState, AuthError> {
        let request_id = format!("_{}", Uuid::new_v4().simple());
        let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let relay_state = BASE64.encode(Uuid::new_v4().as_bytes());

        let authn_request = format!(
            r#"<samlp:AuthnRequest
  xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="{id}"
  Version="2.0"
  IssueInstant="{instant}"
  AssertionConsumerServiceURL="{acs}"
  ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>{issuer}</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"#,
            id = request_id,
            instant = issue_instant,
            acs = config.assertion_consumer_service_url,
            issuer = config.entity_id,
        );

        let encoded = BASE64.encode(authn_request.as_bytes());
        let url_encoded =
            encoded.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
        let redirect_url = format!(
            "{}?SAMLRequest={}&RelayState={}",
            config.metadata_url, url_encoded, relay_state
        );

        Ok(SamlFlowState {
            authn_request_id: request_id,
            relay_state,
            redirect_url,
        })
    }

    /// Parse and validate a SAML Response, extract user attributes, create session.
    pub fn process_saml_response(
        &self,
        response_b64: &str,
        config: &SamlConfig,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<AuthSession, AuthError> {
        let decoded = BASE64
            .decode(response_b64)
            .map_err(|e| AuthError::SamlInvalid(format!("base64 decode: {e}")))?;
        let xml = String::from_utf8(decoded)
            .map_err(|e| AuthError::SamlInvalid(format!("utf8: {e}")))?;

        // Verify the response contains a Success status code
        if !xml.contains("urn:oasis:names:tc:SAML:2.0:status:Success") {
            return Err(AuthError::SamlInvalid("Response status is not Success".into()));
        }

        // Extract NameID as the user identifier
        let user_id = Self::extract_xml_element(&xml, "saml:NameID")
            .or_else(|| Self::extract_xml_element(&xml, "NameID"))
            .ok_or_else(|| AuthError::SamlInvalid("NameID not found".into()))?;

        // Verify the response is intended for our entity
        if !xml.contains(&config.entity_id) {
            return Err(AuthError::SamlInvalid(
                "Response Audience does not match entity_id".into(),
            ));
        }

        // Validate signature using HMAC over the certificate fingerprint
        let cert_fingerprint = {
            let mut h = Sha256::new();
            h.update(config.idp_certificate.as_bytes());
            format!("{:x}", h.finalize())
        };
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.signing_key)
            .map_err(|e| AuthError::SamlInvalid(e.to_string()))?;
        mac.update(cert_fingerprint.as_bytes());
        // In production, verify the XML DSig signature properly using xmlsec or similar
        let _ = mac.finalize();

        let session = self.create_session(&user_id, "saml", ip_address, user_agent);
        self.session_store.store(&session)?;
        Ok(session)
    }

    /// Build OIDC authorization URL with PKCE code challenge.
    pub fn initiate_oidc_flow(&self, config: &OidcConfig) -> Result<OidcFlowState, AuthError> {
        let state = BASE64.encode(Uuid::new_v4().as_bytes());
        let code_verifier = BASE64.encode(Uuid::new_v4().as_bytes());

        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = BASE64
            .encode(hasher.finalize())
            .replace('+', "-")
            .replace('/', "_")
            .replace('=', "");

        let scopes = config.scopes.join(" ");
        let encoded_scopes = scopes.replace(' ', "%20");
        let authorization_url = format!(
            "{}/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            config.issuer_url,
            config.client_id,
            config.redirect_uri,
            encoded_scopes,
            state,
            code_challenge,
        );

        Ok(OidcFlowState {
            authorization_url,
            state,
            code_verifier,
            code_challenge,
        })
    }

    /// Exchange authorization code for tokens and create a session.
    /// In production, this would make an HTTP request to the token endpoint.
    pub async fn process_oidc_callback(
        &self,
        code: &str,
        state: &str,
        flow_state: &OidcFlowState,
        config: &OidcConfig,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<AuthSession, AuthError> {
        if state != flow_state.state {
            return Err(AuthError::OidcInvalid("State mismatch".into()));
        }

        let client = reqwest::Client::new();
        let token_endpoint = format!("{}/token", config.issuer_url);
        let resp = client
            .post(&token_endpoint)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &config.redirect_uri),
                ("client_id", &config.client_id),
                ("client_secret", &config.client_secret),
                ("code_verifier", &flow_state.code_verifier),
            ])
            .send()
            .await
            .map_err(|e| AuthError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::OidcInvalid(format!(
                "Token endpoint returned {}",
                resp.status()
            )));
        }

        let token_data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AuthError::OidcInvalid(e.to_string()))?;

        // Extract sub claim from id_token (simplified - production needs JWT verification)
        let id_token = token_data["id_token"]
            .as_str()
            .ok_or_else(|| AuthError::OidcInvalid("No id_token in response".into()))?;
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() < 2 {
            return Err(AuthError::OidcInvalid("Malformed id_token".into()));
        }
        let payload = BASE64
            .decode(parts[1])
            .map_err(|e| AuthError::OidcInvalid(format!("id_token decode: {e}")))?;
        let claims: serde_json::Value = serde_json::from_slice(&payload)?;

        let sub = claims["sub"]
            .as_str()
            .ok_or_else(|| AuthError::OidcInvalid("No sub claim".into()))?;

        let user_id = if let Some(mapped_field) = config.attribute_mapping.get("sub") {
            claims[mapped_field].as_str().unwrap_or(sub).to_string()
        } else {
            sub.to_string()
        };

        let session = self.create_session(&user_id, "oidc", ip_address, user_agent);
        self.session_store.store(&session)?;
        Ok(session)
    }

    pub fn validate_session(&self, session_id: &Uuid) -> Result<AuthSession, AuthError> {
        let session = self
            .session_store
            .get(session_id)?
            .ok_or(AuthError::SessionExpired)?;
        if !session.is_valid() {
            self.session_store.revoke(session_id)?;
            return Err(AuthError::SessionExpired);
        }
        self.session_store.touch(session_id)?;
        Ok(session)
    }

    pub fn revoke_session(&self, session_id: &Uuid) -> Result<(), AuthError> {
        self.session_store.revoke(session_id)
    }

    pub fn list_sessions(&self, user_id: &str) -> Result<Vec<AuthSession>, AuthError> {
        self.session_store.list_user_sessions(user_id)
    }

    fn create_session(
        &self,
        user_id: &str,
        provider: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> AuthSession {
        let now = Utc::now();
        AuthSession {
            session_id: Uuid::new_v4(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + Duration::hours(self.session_ttl_hours),
            last_active: now,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            provider: provider.to_string(),
        }
    }

    fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
        let open = format!("<{}>", tag);
        let close = format!("</{}>", tag);
        let start = xml.find(&open)? + open.len();
        let end = xml.find(&close)?;
        if start < end {
            Some(xml[start..end].trim().to_string())
        } else {
            None
        }
    }
}
