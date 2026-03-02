use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::types::SkillPackage;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid public key bytes: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature bytes: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Signer not trusted: key {key_hex} has no reputation record")]
    UntrustedSigner { key_hex: String },

    #[error("Signer is banned: {reason}")]
    BannedSigner { reason: String },

    #[error("Manifest hash mismatch: expected {expected}, got {actual}")]
    ManifestHashMismatch { expected: String, actual: String },

    #[error("Key registry error: {0}")]
    RegistryError(String),
}

pub type VerificationResult<T> = std::result::Result<T, SignatureError>;

#[derive(Debug, Clone)]
pub struct SignerReputation {
    pub public_key_hex: String,
    pub developer_id: Option<Uuid>,
    pub trust_level: TrustLevel,
    pub total_skills_signed: u32,
    pub violation_count: u32,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Unknown = 0,
    Community = 1,
    Verified = 2,
    Partner = 3,
    Staff = 4,
}

/// Verifies Ed25519 signatures on skill packages
pub struct SkillVerifier {
    key_registry: KeyRegistry,
}

impl SkillVerifier {
    pub fn new(key_registry: KeyRegistry) -> Self {
        Self { key_registry }
    }

    /// Verifies the Ed25519 signature of a skill package
    pub fn verify_package(&self, package: &SkillPackage) -> VerificationResult<()> {
        // 1. Reconstruct the canonical signed payload
        let signed_payload = build_signed_payload(&package.manifest_hash, &package.wasm_bytes);

        // 2. Parse the verifying key
        let key_bytes: [u8; 32] =
            package
                .signer_public_key
                .as_slice()
                .try_into()
                .map_err(|_| {
                    SignatureError::InvalidPublicKey(format!(
                        "expected 32 bytes, got {}",
                        package.signer_public_key.len()
                    ))
                })?;
        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

        // 3. Parse the signature
        let sig_bytes: [u8; 64] = package.signature.as_slice().try_into().map_err(|_| {
            SignatureError::InvalidSignature(format!(
                "expected 64 bytes, got {}",
                package.signature.len()
            ))
        })?;
        let signature = Signature::from_bytes(&sig_bytes);

        // 4. Verify
        verifying_key
            .verify(&signed_payload, &signature)
            .map_err(|_| SignatureError::VerificationFailed)?;

        // 5. Verify manifest hash matches WASM content
        let computed_hash = compute_wasm_hash(&package.wasm_bytes);
        if computed_hash != package.manifest_hash {
            return Err(SignatureError::ManifestHashMismatch {
                expected: package.manifest_hash.clone(),
                actual: computed_hash,
            });
        }

        debug!(
            "Package {} signature verified successfully",
            package.metadata.id
        );
        Ok(())
    }

    /// Checks the signer's reputation and trust level
    pub fn verify_signer_reputation(
        &self,
        public_key_bytes: &[u8],
    ) -> VerificationResult<&SignerReputation> {
        let key_hex = hex_encode(public_key_bytes);

        let reputation = self.key_registry.get(&key_hex).ok_or_else(|| {
            warn!("Unknown signer key: {}", &key_hex[..16]);
            SignatureError::UntrustedSigner {
                key_hex: key_hex[..16].to_string(),
            }
        })?;

        if reputation.is_banned {
            return Err(SignatureError::BannedSigner {
                reason: reputation
                    .ban_reason
                    .clone()
                    .unwrap_or_else(|| "Policy violation".to_string()),
            });
        }

        Ok(reputation)
    }

    pub fn is_trusted(&self, public_key_bytes: &[u8], min_trust: TrustLevel) -> bool {
        let key_hex = hex_encode(public_key_bytes);
        self.key_registry
            .get(&key_hex)
            .map(|rep| !rep.is_banned && rep.trust_level >= min_trust)
            .unwrap_or(false)
    }
}

/// Manages trusted signing keys stored in memory (backed by SQLite in production)
pub struct KeyRegistry {
    keys: HashMap<String, SignerReputation>,
}

impl KeyRegistry {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn register_key(
        &mut self,
        public_key_bytes: &[u8],
        developer_id: Option<Uuid>,
        trust_level: TrustLevel,
    ) -> VerificationResult<()> {
        // Validate key is a valid Ed25519 public key
        let key_bytes: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
            SignatureError::InvalidPublicKey(format!(
                "expected 32 bytes, got {}",
                public_key_bytes.len()
            ))
        })?;
        VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| SignatureError::InvalidPublicKey(e.to_string()))?;

        let key_hex = hex_encode(public_key_bytes);
        self.keys.insert(
            key_hex.clone(),
            SignerReputation {
                public_key_hex: key_hex,
                developer_id,
                trust_level,
                total_skills_signed: 0,
                violation_count: 0,
                is_banned: false,
                ban_reason: None,
            },
        );
        Ok(())
    }

    pub fn get(&self, key_hex: &str) -> Option<&SignerReputation> {
        self.keys.get(key_hex)
    }

    pub fn ban_key(&mut self, key_hex: &str, reason: &str) -> bool {
        if let Some(rep) = self.keys.get_mut(key_hex) {
            rep.is_banned = true;
            rep.ban_reason = Some(reason.to_string());
            true
        } else {
            false
        }
    }

    pub fn increment_signed_count(&mut self, key_hex: &str) {
        if let Some(rep) = self.keys.get_mut(key_hex) {
            rep.total_skills_signed += 1;
        }
    }

    pub fn record_violation(&mut self, key_hex: &str) {
        if let Some(rep) = self.keys.get_mut(key_hex) {
            rep.violation_count += 1;
            if rep.violation_count >= 3 {
                rep.is_banned = true;
                rep.ban_reason = Some(format!(
                    "Auto-banned after {} violations",
                    rep.violation_count
                ));
            }
        }
    }

    pub fn trusted_key_count(&self) -> usize {
        self.keys
            .values()
            .filter(|r| !r.is_banned && r.trust_level >= TrustLevel::Verified)
            .count()
    }

    /// Load keys from a TOML/JSON config slice of (hex_key, trust_level) pairs
    pub fn load_trusted_keys(
        &mut self,
        entries: &[(Vec<u8>, TrustLevel)],
    ) -> VerificationResult<()> {
        for (key_bytes, trust) in entries {
            self.register_key(key_bytes, None, trust.clone())?;
        }
        Ok(())
    }
}

impl Default for KeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Builds the canonical payload that is signed: SHA256(manifest_hash || wasm_bytes)
pub fn build_signed_payload(manifest_hash: &str, wasm_bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(manifest_hash.as_bytes());
    hasher.update(b"||");
    hasher.update(wasm_bytes);
    hasher.finalize().to_vec()
}

/// Computes SHA256 of WASM bytes, returned as lowercase hex
pub fn compute_wasm_hash(wasm_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
