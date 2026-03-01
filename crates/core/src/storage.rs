use rusqlite::{Connection, params, OptionalExtension};
use chrono::Utc;
use uuid::Uuid;
use std::path::Path;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn in_memory() -> crate::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> crate::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conversation 
            ON messages(conversation_id, created_at);

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_conversation(&self, id: Uuid) -> crate::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO conversations (id, created_at, updated_at) VALUES (?1, ?2, ?3)",
            params![id.to_string(), now, now],
        )?;
        Ok(())
    }

    pub fn save_message(
        &self,
        id: Uuid,
        conversation_id: Uuid,
        role: &str,
        content: &str,
    ) -> crate::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id.to_string(), conversation_id.to_string(), role, content, now],
        )?;

        self.conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id.to_string()],
        )?;

        Ok(())
    }

    pub fn get_conversation_messages(&self, conversation_id: Uuid) -> crate::Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, role, content, created_at FROM messages 
             WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;

        let messages = stmt
            .query_map(params![conversation_id.to_string()], |row| {
                Ok(Message {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                    role: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    pub fn set_config(&self, key: &str, value: &str) -> crate::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> crate::Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM config WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: String,
}
