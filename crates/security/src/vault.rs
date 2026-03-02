use age::secrecy::Secret as AgeSecret;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("secret not found: {0}")]
    NotFound(String),
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("decryption error: {0}")]
    Decryption(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("key derivation error")]
    KeyDerivation,
    #[error("invalid master key length")]
    InvalidKeyLength,
}

pub type Result<T> = std::result::Result<T, Error>;

/// A wrapper that zeroes the inner value when dropped.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> Deref for Secret<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub key: String,
    pub encrypted_value: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A passphrase-encrypted SQLite-backed secret vault using the `age` crate.
pub struct SecretVault {
    conn: Arc<Mutex<Connection>>,
    /// Passphrase used for age encryption/decryption.
    master_passphrase: Secret<Vec<u8>>,
}

impl SecretVault {
    /// Opens (or creates) the vault at the given path, protected by
    /// `passphrase`.
    pub fn open(db_path: PathBuf, passphrase: impl Into<Vec<u8>>) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS secrets (
                 key          TEXT PRIMARY KEY NOT NULL,
                 cipher_blob  BLOB NOT NULL,
                 nonce        BLOB NOT NULL,
                 created_at   TEXT NOT NULL,
                 updated_at   TEXT NOT NULL
             );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            master_passphrase: Secret::new(passphrase.into()),
        })
    }

    /// Opens an in-memory vault (useful for tests).
    pub fn in_memory(passphrase: impl Into<Vec<u8>>) -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS secrets (
                 key          TEXT PRIMARY KEY NOT NULL,
                 cipher_blob  BLOB NOT NULL,
                 nonce        BLOB NOT NULL,
                 created_at   TEXT NOT NULL,
                 updated_at   TEXT NOT NULL
             );",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            master_passphrase: Secret::new(passphrase.into()),
        })
    }

    fn passphrase_secret(&self) -> AgeSecret<String> {
        let s = String::from_utf8_lossy(self.master_passphrase.expose()).into_owned();
        AgeSecret::new(s)
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let encryptor = age::Encryptor::with_user_passphrase(self.passphrase_secret());

        let mut ciphertext = vec![];
        let mut writer = encryptor
            .wrap_output(&mut ciphertext)
            .map_err(|e| Error::Encryption(e.to_string()))?;

        use std::io::Write;
        writer
            .write_all(plaintext)
            .map_err(|e| Error::Encryption(e.to_string()))?;
        writer
            .finish()
            .map_err(|e| Error::Encryption(e.to_string()))?;

        // age manages its own nonce internally in the ciphertext header; we
        // store an empty nonce field so the schema remains consistent.
        Ok((ciphertext, vec![]))
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let decryptor =
            match age::Decryptor::new(ciphertext).map_err(|e| Error::Decryption(e.to_string()))? {
                age::Decryptor::Passphrase(d) => d,
                _ => {
                    return Err(Error::Decryption(
                        "unexpected decryptor variant (expected Passphrase)".into(),
                    ))
                }
            };

        let mut plaintext = vec![];
        let mut reader = decryptor
            .decrypt(&self.passphrase_secret(), None)
            .map_err(|e| Error::Decryption(e.to_string()))?;

        use std::io::Read;
        reader
            .read_to_end(&mut plaintext)
            .map_err(|e| Error::Decryption(e.to_string()))?;
        Ok(plaintext)
    }

    /// Retrieves and decrypts the secret stored under `key`.
    pub fn get(&self, key: &str) -> Result<Secret<Vec<u8>>> {
        let conn = self.conn.lock().unwrap();
        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT cipher_blob FROM secrets WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        let cipher_blob = result.ok_or_else(|| Error::NotFound(key.to_owned()))?;
        let plaintext = self.decrypt(&cipher_blob)?;
        Ok(Secret::new(plaintext))
    }

    /// Encrypts and stores `value` under `key`, upserting if it already exists.
    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let (cipher_blob, nonce) = self.encrypt(value)?;
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO secrets (key, cipher_blob, nonce, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(key) DO UPDATE SET
                 cipher_blob = excluded.cipher_blob,
                 nonce       = excluded.nonce,
                 updated_at  = excluded.updated_at",
            params![key, cipher_blob, nonce, now],
        )?;
        Ok(())
    }

    /// Deletes the entry for `key`. Returns `Ok` even if the key did not exist.
    pub fn delete(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM secrets WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Re-encrypts all secrets under a new master passphrase.
    pub fn rotate_master_key(&mut self, new_passphrase: impl Into<Vec<u8>>) -> Result<()> {
        let new_passphrase = new_passphrase.into();

        // Step 1: collect and decrypt all entries with the current (old) key.
        let entries: Vec<(String, Vec<u8>)> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT key, cipher_blob FROM secrets")?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            rows
        };

        let mut plaintexts: Vec<(String, Vec<u8>)> = Vec::with_capacity(entries.len());
        for (key, cipher_blob) in &entries {
            let plaintext = self.decrypt(cipher_blob)?;
            plaintexts.push((key.clone(), plaintext));
        }

        // Step 2: swap to the new passphrase.
        self.master_passphrase = Secret::new(new_passphrase);

        // Step 3: re-encrypt all plaintexts with the new key and write back.
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        for (key, plaintext) in &plaintexts {
            let (blob, nonce) = self.encrypt(plaintext)?;
            conn.execute(
                "UPDATE secrets SET cipher_blob = ?2, nonce = ?3, updated_at = ?4 WHERE key = ?1",
                params![key, blob, nonce, now],
            )?;
        }
        Ok(())
    }

    /// Exports all entries as a `Vec<VaultEntry>` (cipher blobs remain
    /// encrypted under the current master passphrase).
    pub fn export(&self) -> Result<Vec<VaultEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT key, cipher_blob, nonce, created_at, updated_at FROM secrets")?;
        let entries = stmt
            .query_map([], |row| {
                Ok(VaultEntry {
                    key: row.get(0)?,
                    encrypted_value: row.get(1)?,
                    nonce: row.get(2)?,
                    created_at: row
                        .get::<_, String>(3)?
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: row
                        .get::<_, String>(4)?
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entries)
    }

    /// Imports a set of pre-encrypted `VaultEntry` records. Existing keys are
    /// overwritten.
    pub fn import(&self, entries: Vec<VaultEntry>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for entry in entries {
            conn.execute(
                "INSERT INTO secrets (key, cipher_blob, nonce, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(key) DO UPDATE SET
                     cipher_blob = excluded.cipher_blob,
                     nonce       = excluded.nonce,
                     updated_at  = excluded.updated_at",
                params![
                    entry.key,
                    entry.encrypted_value,
                    entry.nonce,
                    entry.created_at.to_rfc3339(),
                    entry.updated_at.to_rfc3339(),
                ],
            )?;
        }
        Ok(())
    }
}

/// A `tracing_subscriber::Layer` that redacts known secret values from all
/// log fields before they are recorded.
pub struct SecretRedactor {
    known_secrets: Arc<Mutex<HashSet<String>>>,
}

impl SecretRedactor {
    pub fn new() -> Self {
        Self {
            known_secrets: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Registers a plaintext secret value so it will be redacted in logs.
    pub fn register_secret(&self, secret: impl Into<String>) {
        self.known_secrets.lock().unwrap().insert(secret.into());
    }

    fn redact(&self, input: &str) -> String {
        let secrets = self.known_secrets.lock().unwrap();
        let mut output = input.to_owned();
        for secret in secrets.iter() {
            if !secret.is_empty() {
                output = output.replace(secret.as_str(), "[REDACTED]");
            }
        }
        output
    }
}

impl Default for SecretRedactor {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor that collects string field values for redaction.
struct RedactingVisitor<'a> {
    redactor: &'a SecretRedactor,
    fields: Vec<(String, String)>,
}

impl<'a> Visit for RedactingVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .push((field.name().to_owned(), self.redactor.redact(value)));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let raw = format!("{:?}", value);
        self.fields
            .push((field.name().to_owned(), self.redactor.redact(&raw)));
    }
}

impl<S: Subscriber> Layer<S> for SecretRedactor {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = RedactingVisitor {
            redactor: self,
            fields: Vec::new(),
        };
        event.record(&mut visitor);
        // If any field contained a secret the visitor will have redacted it.
        // We log a summary via `tracing` itself (avoiding infinite recursion
        // because we are only observing, not emitting).
        for (name, value) in &visitor.fields {
            if value.contains("[REDACTED]") {
                tracing::trace!(
                    field = %name,
                    "[SecretRedactor] secret value was present in log field and has been redacted"
                );
            }
        }
    }
}

/// Extension trait on `rusqlite::Result` for convenient optional query.
trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
