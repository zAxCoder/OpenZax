use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::sync::Mutex;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    error::Result,
    types::{
        DeveloperProfile, Review, ReviewStatus, Skill, SkillCategory, SkillSearchQuery,
        SkillSortOrder,
    },
};

pub struct MarketplaceDb {
    conn: Mutex<Connection>,
}

impl MarketplaceDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn with_conn<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Connection) -> T,
    {
        let conn = self.conn.lock().expect("MarketplaceDb mutex poisoned");
        f(&conn)
    }

    pub fn initialize(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

            conn.execute_batch(r#"
                CREATE TABLE IF NOT EXISTS developers (
                    id          TEXT PRIMARY KEY,
                    username    TEXT NOT NULL UNIQUE,
                    email       TEXT NOT NULL UNIQUE,
                    bio         TEXT NOT NULL DEFAULT '',
                    avatar_url  TEXT,
                    skills_published INTEGER NOT NULL DEFAULT 0,
                    total_downloads  INTEGER NOT NULL DEFAULT 0,
                    total_revenue_cents INTEGER NOT NULL DEFAULT 0,
                    verified    INTEGER NOT NULL DEFAULT 0,
                    created_at  TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS skills (
                    id                  TEXT PRIMARY KEY,
                    name                TEXT NOT NULL,
                    version             TEXT NOT NULL,
                    description         TEXT NOT NULL,
                    author_id           TEXT NOT NULL,
                    author_name         TEXT NOT NULL,
                    license             TEXT NOT NULL,
                    category            TEXT NOT NULL,
                    tags                TEXT NOT NULL DEFAULT '[]',
                    permissions_required TEXT NOT NULL DEFAULT '[]',
                    download_count      INTEGER NOT NULL DEFAULT 0,
                    rating_avg          REAL NOT NULL DEFAULT 0.0,
                    rating_count        INTEGER NOT NULL DEFAULT 0,
                    price_cents         INTEGER NOT NULL DEFAULT 0,
                    review_status       TEXT NOT NULL DEFAULT 'pending',
                    created_at          TEXT NOT NULL,
                    updated_at          TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS skill_packages (
                    skill_id            TEXT PRIMARY KEY REFERENCES skills(id),
                    wasm_bytes          BLOB NOT NULL,
                    signature           BLOB NOT NULL,
                    signer_public_key   BLOB NOT NULL,
                    manifest_hash       TEXT NOT NULL,
                    stored_at           TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS reviews (
                    id                  TEXT PRIMARY KEY,
                    skill_id            TEXT NOT NULL REFERENCES skills(id),
                    reviewer_id         TEXT NOT NULL,
                    rating              INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
                    comment             TEXT NOT NULL DEFAULT '',
                    is_community_review INTEGER NOT NULL DEFAULT 0,
                    created_at          TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS audit_log (
                    id          TEXT PRIMARY KEY,
                    entity_type TEXT NOT NULL,
                    entity_id   TEXT NOT NULL,
                    action      TEXT NOT NULL,
                    actor_id    TEXT,
                    details     TEXT,
                    occurred_at TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);
                CREATE INDEX IF NOT EXISTS idx_skills_author ON skills(author_id);
                CREATE INDEX IF NOT EXISTS idx_skills_status ON skills(review_status);
                CREATE INDEX IF NOT EXISTS idx_reviews_skill ON reviews(skill_id);
                CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);
            "#)?;

            conn.execute_batch(r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
                    id UNINDEXED,
                    name,
                    description,
                    tags,
                    author_name,
                    content=skills,
                    content_rowid=rowid
                );

                CREATE TRIGGER IF NOT EXISTS skills_fts_insert AFTER INSERT ON skills BEGIN
                    INSERT INTO skills_fts(rowid, id, name, description, tags, author_name)
                    VALUES (new.rowid, new.id, new.name, new.description, new.tags, new.author_name);
                END;

                CREATE TRIGGER IF NOT EXISTS skills_fts_update AFTER UPDATE ON skills BEGIN
                    UPDATE skills_fts SET name=new.name, description=new.description,
                        tags=new.tags, author_name=new.author_name
                    WHERE id=new.id;
                END;

                CREATE TRIGGER IF NOT EXISTS skills_fts_delete AFTER DELETE ON skills BEGIN
                    DELETE FROM skills_fts WHERE id=old.id;
                END;
            "#)?;

            info!("Marketplace database initialized");
            Ok(())
        })
    }

    // ── Skill CRUD ──────────────────────────────────────────────────────────

    pub fn insert_skill(&self, skill: &Skill) -> Result<()> {
        let tags = serde_json::to_string(&skill.tags)?;
        let perms = serde_json::to_string(&skill.permissions_required)?;

        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT INTO skills (id, name, version, description, author_id, author_name,
                   license, category, tags, permissions_required, download_count, rating_avg,
                   rating_count, price_cents, review_status, created_at, updated_at)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)"#,
                params![
                    skill.id.to_string(),
                    skill.name,
                    skill.version,
                    skill.description,
                    skill.author_id.to_string(),
                    skill.author_name,
                    skill.license,
                    skill.category.to_string(),
                    tags,
                    perms,
                    skill.download_count as i64,
                    skill.rating_avg as f64,
                    skill.rating_count as i64,
                    skill.price_cents as i64,
                    skill.review_status.as_db_str(),
                    skill.created_at.to_rfc3339(),
                    skill.updated_at.to_rfc3339(),
                ],
            )?;
            debug!("Inserted skill {}", skill.id);
            Ok(())
        })
    }

    pub fn get_skill(&self, id: Uuid) -> Result<Option<Skill>> {
        self.with_conn(|conn| {
            let row = conn
                .query_row(
                    "SELECT * FROM skills WHERE id = ?1",
                    params![id.to_string()],
                    row_to_skill,
                )
                .optional()?;
            Ok(row)
        })
    }

    pub fn update_skill_status(&self, id: Uuid, status: &ReviewStatus) -> Result<()> {
        let updated_at = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET review_status=?1, updated_at=?2 WHERE id=?3",
                params![status.as_db_str(), updated_at, id.to_string()],
            )?;
            Ok(())
        })
    }

    pub fn increment_download_count(&self, id: Uuid) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET download_count = download_count + 1 WHERE id = ?1",
                params![id.to_string()],
            )?;
            Ok(())
        })
    }

    pub fn search_skills(&self, query: &SkillSearchQuery) -> Result<(Vec<Skill>, u64)> {
        let offset = ((query.page.saturating_sub(1)) * query.per_page) as i64;
        let limit = query.per_page as i64;

        let mut conditions = vec!["s.review_status = 'approved'".to_string()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref cat) = query.category {
            conditions.push(format!("s.category = ?{}", bind_values.len() + 1));
            bind_values.push(cat.to_string());
        }

        if query.free_only {
            conditions.push("s.price_cents = 0".to_string());
        } else if let Some(max_price) = query.max_price_cents {
            conditions.push(format!("s.price_cents <= ?{}", bind_values.len() + 1));
            bind_values.push(max_price.to_string());
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));

        let order_clause = match query.sort {
            SkillSortOrder::Newest => "s.created_at DESC",
            SkillSortOrder::Rating => "s.rating_avg DESC",
            SkillSortOrder::Downloads => "s.download_count DESC",
            SkillSortOrder::PriceLow => "s.price_cents ASC",
            SkillSortOrder::PriceHigh => "s.price_cents DESC",
            SkillSortOrder::Trending => {
                // Score = log(downloads+1) + 3*rating_count / (hours_since_publish + 2)^1.5
                "(((s.download_count + 1) + 3.0 * s.rating_count) / \
                  ((JULIANDAY('now') - JULIANDAY(s.created_at)) * 24.0 + 2.0)) DESC"
            }
        };

        if let Some(ref text) = query.query {
            let fts_sql = r#"SELECT s.* FROM skills s
                   JOIN skills_fts f ON f.id = s.id
                   WHERE skills_fts MATCH ?1 AND s.review_status = 'approved'
                   ORDER BY rank LIMIT ?2 OFFSET ?3"#
                .to_string();

            return self.with_conn(|conn| {
                let mut stmt = conn.prepare(&fts_sql)?;
                let skills: Vec<Skill> = stmt
                    .query_map(params![text, limit, offset], row_to_skill)?
                    .filter_map(|r| r.ok())
                    .collect();
                let count = skills.len() as u64;
                Ok((skills, count))
            });
        }

        let count_sql = format!("SELECT COUNT(*) FROM skills s {where_clause}");
        let data_sql = format!(
            "SELECT s.* FROM skills s {where_clause} ORDER BY {order_clause} LIMIT {limit} OFFSET {offset}"
        );

        self.with_conn(|conn| {
            let total: i64 = {
                let mut stmt = conn.prepare(&count_sql)?;
                let mut rows = stmt.query(rusqlite::params_from_iter(bind_values.iter()))?;
                rows.next()?.and_then(|r| r.get(0).ok()).unwrap_or(0)
            };

            let mut stmt = conn.prepare(&data_sql)?;
            let skills: Vec<Skill> = stmt
                .query_map(rusqlite::params_from_iter(bind_values.iter()), row_to_skill)?
                .filter_map(|r| r.ok())
                .collect();

            Ok((skills, total as u64))
        })
    }

    pub fn get_featured_skills(&self, limit: u32) -> Result<Vec<Skill>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT * FROM skills
                   WHERE review_status = 'approved'
                   ORDER BY download_count DESC, rating_avg DESC
                   LIMIT ?1"#,
            )?;
            let skills = stmt
                .query_map(params![limit as i64], row_to_skill)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(skills)
        })
    }

    // ── Package storage ──────────────────────────────────────────────────────

    pub fn store_package(
        &self,
        skill_id: Uuid,
        wasm_bytes: &[u8],
        signature: &[u8],
        pubkey: &[u8],
        hash: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT OR REPLACE INTO skill_packages (skill_id, wasm_bytes, signature, signer_public_key, manifest_hash, stored_at)
                   VALUES (?1,?2,?3,?4,?5,?6)"#,
                params![skill_id.to_string(), wasm_bytes, signature, pubkey, hash, now],
            )?;
            Ok(())
        })
    }

    pub fn get_package_bytes(&self, skill_id: Uuid) -> Result<Option<Vec<u8>>> {
        self.with_conn(|conn| {
            let row = conn
                .query_row(
                    "SELECT wasm_bytes FROM skill_packages WHERE skill_id = ?1",
                    params![skill_id.to_string()],
                    |row| row.get::<_, Vec<u8>>(0),
                )
                .optional()?;
            Ok(row)
        })
    }

    // ── Reviews ──────────────────────────────────────────────────────────────

    pub fn insert_review(&self, review: &Review) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT INTO reviews (id, skill_id, reviewer_id, rating, comment, is_community_review, created_at)
                   VALUES (?1,?2,?3,?4,?5,?6,?7)"#,
                params![
                    review.id.to_string(),
                    review.skill_id.to_string(),
                    review.reviewer_id.to_string(),
                    review.rating as i64,
                    review.comment,
                    review.is_community_review as i64,
                    review.created_at.to_rfc3339(),
                ],
            )?;

            conn.execute(
                r#"UPDATE skills SET
                   rating_avg = (SELECT AVG(CAST(rating AS REAL)) FROM reviews WHERE skill_id = ?1),
                   rating_count = (SELECT COUNT(*) FROM reviews WHERE skill_id = ?1),
                   updated_at = ?2
                   WHERE id = ?1"#,
                params![review.skill_id.to_string(), Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    pub fn list_reviews(&self, skill_id: Uuid) -> Result<Vec<Review>> {
        self.with_conn(|conn| {
            let mut stmt =
                conn.prepare("SELECT * FROM reviews WHERE skill_id = ?1 ORDER BY created_at DESC")?;
            let reviews = stmt
                .query_map(params![skill_id.to_string()], row_to_review)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(reviews)
        })
    }

    // ── Developers ───────────────────────────────────────────────────────────

    pub fn insert_developer(&self, dev: &DeveloperProfile) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT INTO developers (id, username, email, bio, avatar_url, skills_published,
                   total_downloads, total_revenue_cents, verified, created_at)
                   VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"#,
                params![
                    dev.id.to_string(),
                    dev.username,
                    dev.email,
                    dev.bio,
                    dev.avatar_url,
                    dev.skills_published as i64,
                    dev.total_downloads as i64,
                    dev.total_revenue_cents as i64,
                    dev.verified as i64,
                    dev.created_at.to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_developer(&self, id: Uuid) -> Result<Option<DeveloperProfile>> {
        self.with_conn(|conn| {
            let row = conn
                .query_row(
                    "SELECT * FROM developers WHERE id = ?1",
                    params![id.to_string()],
                    row_to_developer,
                )
                .optional()?;
            Ok(row)
        })
    }

    // ── Audit Log ────────────────────────────────────────────────────────────

    pub fn log_action(
        &self,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        actor_id: Option<&str>,
        details: Option<&str>,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_log (id, entity_type, entity_id, action, actor_id, details, occurred_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![id, entity_type, entity_id, action, actor_id, details, now],
            )?;
            Ok(())
        })
    }
}

// ── Free row-mapper functions (no &self borrow needed) ───────────────────────

fn row_to_skill(row: &Row<'_>) -> rusqlite::Result<Skill> {
    let tags_str: String = row.get("tags")?;
    let perms_str: String = row.get("permissions_required")?;
    let status_str: String = row.get("review_status")?;
    let category_str: String = row.get("category")?;
    let created_str: String = row.get("created_at")?;
    let updated_str: String = row.get("updated_at")?;

    Ok(Skill {
        id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap_or_else(|_| Uuid::nil()),
        name: row.get("name")?,
        version: row.get("version")?,
        description: row.get("description")?,
        author_id: Uuid::parse_str(&row.get::<_, String>("author_id")?)
            .unwrap_or_else(|_| Uuid::nil()),
        author_name: row.get("author_name")?,
        license: row.get("license")?,
        category: category_str.parse().unwrap_or(SkillCategory::Utilities),
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        permissions_required: serde_json::from_str(&perms_str).unwrap_or_default(),
        download_count: row.get::<_, i64>("download_count")? as u64,
        rating_avg: row.get::<_, f64>("rating_avg")? as f32,
        rating_count: row.get::<_, i64>("rating_count")? as u32,
        price_cents: row.get::<_, i64>("price_cents")? as u32,
        review_status: ReviewStatus::from_db_str(&status_str),
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn row_to_review(row: &Row<'_>) -> rusqlite::Result<Review> {
    let created_str: String = row.get("created_at")?;
    Ok(Review {
        id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap_or_else(|_| Uuid::nil()),
        skill_id: Uuid::parse_str(&row.get::<_, String>("skill_id")?)
            .unwrap_or_else(|_| Uuid::nil()),
        reviewer_id: Uuid::parse_str(&row.get::<_, String>("reviewer_id")?)
            .unwrap_or_else(|_| Uuid::nil()),
        rating: row.get::<_, i64>("rating")? as u8,
        comment: row.get("comment")?,
        is_community_review: row.get::<_, i64>("is_community_review")? != 0,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

fn row_to_developer(row: &Row<'_>) -> rusqlite::Result<DeveloperProfile> {
    let created_str: String = row.get("created_at")?;
    Ok(DeveloperProfile {
        id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap_or_else(|_| Uuid::nil()),
        username: row.get("username")?,
        email: row.get("email")?,
        bio: row.get("bio")?,
        avatar_url: row.get("avatar_url")?,
        skills_published: row.get::<_, i64>("skills_published")? as u32,
        total_downloads: row.get::<_, i64>("total_downloads")? as u64,
        total_revenue_cents: row.get::<_, i64>("total_revenue_cents")? as u64,
        verified: row.get::<_, i64>("verified")? != 0,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}
