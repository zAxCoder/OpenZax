use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("token has expired")]
    Expired,
    #[error("token has been revoked")]
    Revoked,
    #[error("signature verification failed")]
    InvalidSignature,
    #[error("permission not granted: {0:?}")]
    PermissionDenied(Permission),
    #[error("delegation would exceed grantor permissions")]
    ExceedsGrantorPermissions,
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("ed25519 error: {0}")]
    Crypto(#[from] ed25519_dalek::SignatureError),
    #[error("nonce too short")]
    InvalidNonce,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    FsRead(PathBuf),
    FsWrite(PathBuf),
    FsExecute(PathBuf),
    NetHttp(String),
    NetWebSocket(String),
    ToolCall(String),
    AgentSpawn(String),
    EnvRead(String),
    KvStore(String),
    LogWrite,
}

impl Permission {
    /// Returns true if `self` is at least as permissive as `other`, used for
    /// delegation checks. Wildcard string `"*"` subsumes any value of the same
    /// variant.
    pub fn subsumes(&self, other: &Permission) -> bool {
        match (self, other) {
            (Permission::FsRead(a), Permission::FsRead(b)) => path_subsumes(a, b),
            (Permission::FsWrite(a), Permission::FsWrite(b)) => path_subsumes(a, b),
            (Permission::FsExecute(a), Permission::FsExecute(b)) => path_subsumes(a, b),
            (Permission::NetHttp(a), Permission::NetHttp(b)) => wildcard_subsumes(a, b),
            (Permission::NetWebSocket(a), Permission::NetWebSocket(b)) => wildcard_subsumes(a, b),
            (Permission::ToolCall(a), Permission::ToolCall(b)) => wildcard_subsumes(a, b),
            (Permission::AgentSpawn(a), Permission::AgentSpawn(b)) => wildcard_subsumes(a, b),
            (Permission::EnvRead(a), Permission::EnvRead(b)) => wildcard_subsumes(a, b),
            (Permission::KvStore(a), Permission::KvStore(b)) => wildcard_subsumes(a, b),
            (Permission::LogWrite, Permission::LogWrite) => true,
            _ => false,
        }
    }
}

fn path_subsumes(grantor: &PathBuf, requested: &PathBuf) -> bool {
    if grantor == &PathBuf::from("*") {
        return true;
    }
    requested.starts_with(grantor)
}

fn wildcard_subsumes(grantor: &str, requested: &str) -> bool {
    grantor == "*" || grantor == requested
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub token_id: Uuid,
    pub holder_id: String,
    pub permissions: Vec<Permission>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub signature: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl CapabilityToken {
    /// Checks whether this token is temporally valid (not expired).
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() > exp
        } else {
            false
        }
    }

    /// Returns the canonical byte payload that was / should be signed.
    pub fn signing_payload(&self) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct Payload<'a> {
            token_id: Uuid,
            holder_id: &'a str,
            permissions: &'a Vec<Permission>,
            issued_at: DateTime<Utc>,
            expires_at: Option<DateTime<Utc>>,
            nonce: &'a Vec<u8>,
        }
        let p = Payload {
            token_id: self.token_id,
            holder_id: &self.holder_id,
            permissions: &self.permissions,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            nonce: &self.nonce,
        };
        Ok(serde_json::to_vec(&p)?)
    }

    /// Returns true if `perm` is covered by this token's permission list.
    pub fn has_permission(&self, perm: &Permission) -> bool {
        self.permissions.iter().any(|p| p.subsumes(perm))
    }
}

/// Bloom filter stub backed by a `HashSet` for simplicity.
#[derive(Debug, Default)]
pub struct RevocationFilter {
    revoked: HashSet<Uuid>,
}

impl RevocationFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, token_id: Uuid) {
        self.revoked.insert(token_id);
    }

    pub fn contains(&self, token_id: &Uuid) -> bool {
        self.revoked.contains(token_id)
    }
}

pub struct CapabilityAuthority {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    revocation_filter: RevocationFilter,
}

impl CapabilityAuthority {
    /// Creates a new authority with a freshly generated Ed25519 key pair.
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
            revocation_filter: RevocationFilter::new(),
        }
    }

    /// Creates an authority from an existing signing key bytes (32 bytes).
    pub fn from_key_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
            revocation_filter: RevocationFilter::new(),
        }
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Mints a new capability token for the given holder with the given
    /// permissions and optional TTL.
    pub fn mint(
        &self,
        holder_id: impl Into<String>,
        permissions: Vec<Permission>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<CapabilityToken> {
        let mut nonce = vec![0u8; 32];
        rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);

        let mut token = CapabilityToken {
            token_id: Uuid::new_v4(),
            holder_id: holder_id.into(),
            permissions,
            issued_at: Utc::now(),
            expires_at,
            signature: Vec::new(),
            nonce,
        };

        let payload = token.signing_payload()?;
        let signature: Signature = self.signing_key.sign(&payload);
        token.signature = signature.to_bytes().to_vec();
        Ok(token)
    }

    /// Verifies the signature on a token and checks revocation + expiry.
    pub fn verify(&self, token: &CapabilityToken) -> Result<()> {
        if self.revocation_filter.contains(&token.token_id) {
            return Err(Error::Revoked);
        }
        if token.is_expired() {
            return Err(Error::Expired);
        }

        let payload = token.signing_payload()?;
        let sig_bytes: [u8; 64] = token
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidSignature)?;
        let signature = Signature::from_bytes(&sig_bytes);
        self.verifying_key
            .verify(&payload, &signature)
            .map_err(|_| Error::InvalidSignature)?;
        Ok(())
    }

    /// Creates a child token that can only hold a subset of the parent's
    /// permissions.
    pub fn delegate(
        &self,
        parent: &CapabilityToken,
        holder_id: impl Into<String>,
        requested: Vec<Permission>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<CapabilityToken> {
        self.verify(parent)?;

        for perm in &requested {
            if !parent.has_permission(perm) {
                return Err(Error::ExceedsGrantorPermissions);
            }
        }

        // Child cannot outlive parent.
        let effective_expiry = match (parent.expires_at, expires_at) {
            (Some(p), Some(c)) => Some(p.min(c)),
            (Some(p), None) => Some(p),
            (None, c) => c,
        };

        self.mint(holder_id, requested, effective_expiry)
    }

    /// Adds a token to the revocation filter.
    pub fn revoke(&mut self, token_id: Uuid) {
        self.revocation_filter.insert(token_id);
    }

    /// Checks whether a token ID has been revoked.
    pub fn is_revoked(&self, token_id: &Uuid) -> bool {
        self.revocation_filter.contains(token_id)
    }
}

impl Default for CapabilityAuthority {
    fn default() -> Self {
        Self::new()
    }
}
