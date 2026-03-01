use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    error::{MarketplaceError, Result},
    scanner::Tier1Scanner,
    storage::MarketplaceDb,
    types::{
        PaginatedResponse, Review, ReviewStatus, Skill, SkillCategory,
        SkillSearchQuery, SkillSortOrder,
    },
    verification::{KeyRegistry, SkillVerifier},
};

pub struct AppState {
    pub db: Arc<MarketplaceDb>,
    pub scanner: Arc<Tier1Scanner>,
    pub verifier: Arc<SkillVerifier>,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(db_path: &str, jwt_secret: impl Into<String>) -> Result<Self> {
        let db = MarketplaceDb::open(db_path)?;
        let key_registry = KeyRegistry::new();
        let verifier = SkillVerifier::new(key_registry);

        Ok(Self {
            db: Arc::new(db),
            scanner: Arc::new(Tier1Scanner::new()),
            verifier: Arc::new(verifier),
            jwt_secret: jwt_secret.into(),
        })
    }
}

pub fn build_router(state: Arc<AppState>) -> Router {
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    Router::new()
        .route("/v1/skills", get(list_skills).post(submit_skill))
        .route("/v1/skills/:id", get(get_skill))
        .route("/v1/skills/:id/download", get(download_skill))
        .route("/v1/skills/:id/reviews", get(list_reviews).post(submit_review))
        .route("/v1/developers/:id", get(get_developer))
        .route("/v1/search", get(search_skills))
        .route("/v1/featured", get(featured_skills))
        .route("/v1/auth/login", post(login))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ── Request/Response types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListSkillsParams {
    pub category: Option<String>,
    pub tags: Option<String>,
    pub max_price: Option<u32>,
    pub free_only: Option<bool>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self { data, meta: None })
    }
}

#[derive(Debug, Deserialize)]
pub struct SubmitSkillRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub category: String,
    pub tags: Vec<String>,
    pub permissions_required: Vec<String>,
    pub price_cents: u32,
    /// Base64-encoded WASM bytes
    pub wasm_base64: String,
    /// Hex-encoded Ed25519 signature
    pub signature_hex: String,
    /// Hex-encoded signer public key
    pub signer_pubkey_hex: String,
    pub manifest_hash: String,
    pub author_id: String,
    pub author_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReviewRequest {
    pub reviewer_id: String,
    pub rating: u8,
    pub comment: String,
    pub is_community_review: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub developer_id: String,
    pub expires_in: u64,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_skills(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListSkillsParams>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let category = params.category
        .as_deref()
        .map(|c| c.parse::<SkillCategory>())
        .transpose()?;

    let tags: Vec<String> = params.tags
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let sort = match params.sort.as_deref() {
        Some("newest") => SkillSortOrder::Newest,
        Some("rating") => SkillSortOrder::Rating,
        Some("downloads") => SkillSortOrder::Downloads,
        Some("price_low") => SkillSortOrder::PriceLow,
        Some("price_high") => SkillSortOrder::PriceHigh,
        _ => SkillSortOrder::Trending,
    };

    let query = SkillSearchQuery {
        query: None,
        category,
        tags,
        max_price_cents: params.max_price,
        free_only: params.free_only.unwrap_or(false),
        sort,
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(20).min(100),
    };

    let (skills, total) = state.db.search_skills(&query)?;
    let response = PaginatedResponse::new(skills, total, query.page, query.per_page);
    Ok(ApiResponse::ok(response))
}

async fn get_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let skill = state.db.get_skill(id)?
        .ok_or(MarketplaceError::SkillNotFound(id))?;
    Ok(ApiResponse::ok(skill))
}

async fn submit_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitSkillRequest>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    // Decode WASM and signature
    let wasm_bytes = base64_decode(&req.wasm_base64)
        .map_err(|e| MarketplaceError::InvalidWasm(format!("base64 decode failed: {e}")))?;
    let signature = hex_decode(&req.signature_hex)
        .map_err(|e| MarketplaceError::SignatureError(format!("signature decode: {e}")))?;
    let signer_pubkey = hex_decode(&req.signer_pubkey_hex)
        .map_err(|e| MarketplaceError::SignatureError(format!("pubkey decode: {e}")))?;

    // Size check: 50 MB limit
    const MAX_WASM_SIZE: usize = 50 * 1024 * 1024;
    if wasm_bytes.len() > MAX_WASM_SIZE {
        return Err(MarketplaceError::PackageTooLarge { size: wasm_bytes.len(), limit: MAX_WASM_SIZE });
    }

    let category = req.category.parse::<SkillCategory>()?;
    let author_id = Uuid::parse_str(&req.author_id)
        .map_err(|_| MarketplaceError::InvalidCategory("invalid author_id UUID".to_string()))?;

    // Tier 1 scan
    let scan_result = state.scanner.scan(&wasm_bytes, &req.permissions_required);
    let initial_status = if scan_result.passed {
        ReviewStatus::AutoScanPassed
    } else {
        warn!("Skill '{}' failed auto-scan: score={}", req.name, scan_result.risk_score);
        ReviewStatus::AutoScanFailed
    };

    let now = Utc::now();
    let skill = Skill {
        id: Uuid::new_v4(),
        name: req.name,
        version: req.version,
        description: req.description,
        author_id,
        author_name: req.author_name,
        license: req.license,
        category,
        tags: req.tags,
        permissions_required: req.permissions_required,
        download_count: 0,
        rating_avg: 0.0,
        rating_count: 0,
        price_cents: req.price_cents,
        review_status: initial_status.clone(),
        created_at: now,
        updated_at: now,
    };

    state.db.insert_skill(&skill)?;
    state.db.store_package(skill.id, &wasm_bytes, &signature, &signer_pubkey, &req.manifest_hash)?;
    state.db.log_action("skill", &skill.id.to_string(), "submitted", Some(&req.author_id), None)?;

    info!("Skill {} ({}) submitted by {}", skill.id, skill.name, skill.author_name);

    Ok((StatusCode::CREATED, ApiResponse::ok(skill)))
}

async fn download_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let skill = state.db.get_skill(id)?
        .ok_or(MarketplaceError::SkillNotFound(id))?;

    if skill.review_status != ReviewStatus::Approved {
        return Err(MarketplaceError::PermissionDenied(
            "Skill is not yet approved for download".to_string(),
        ));
    }

    let wasm_bytes = state.db.get_package_bytes(id)?
        .ok_or(MarketplaceError::SkillNotFound(id))?;

    state.db.increment_download_count(id)?;

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/wasm")
        .header("Content-Disposition", format!("attachment; filename=\"{}.wasm\"", skill.name))
        .header("Content-Length", wasm_bytes.len().to_string())
        .body(axum::body::Body::from(wasm_bytes))
        .unwrap();

    Ok(response)
}

async fn submit_review(
    State(state): State<Arc<AppState>>,
    Path(skill_id): Path<Uuid>,
    Json(req): Json<SubmitReviewRequest>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    if !(1..=5).contains(&req.rating) {
        return Err(MarketplaceError::InvalidRating);
    }

    let reviewer_id = Uuid::parse_str(&req.reviewer_id)
        .map_err(|_| MarketplaceError::InvalidCategory("invalid reviewer_id".to_string()))?;

    state.db.get_skill(skill_id)?
        .ok_or(MarketplaceError::SkillNotFound(skill_id))?;

    let review = Review {
        id: Uuid::new_v4(),
        skill_id,
        reviewer_id,
        rating: req.rating,
        comment: req.comment,
        is_community_review: req.is_community_review,
        created_at: Utc::now(),
    };

    state.db.insert_review(&review)?;
    Ok((StatusCode::CREATED, ApiResponse::ok(review)))
}

async fn list_reviews(
    State(state): State<Arc<AppState>>,
    Path(skill_id): Path<Uuid>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    state.db.get_skill(skill_id)?
        .ok_or(MarketplaceError::SkillNotFound(skill_id))?;
    let reviews = state.db.list_reviews(skill_id)?;
    Ok(ApiResponse::ok(reviews))
}

async fn get_developer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let dev = state.db.get_developer(id)?
        .ok_or(MarketplaceError::DeveloperNotFound(id))?;
    Ok(ApiResponse::ok(dev))
}

async fn search_skills(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let query = SkillSearchQuery {
        query: Some(params.q),
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(20).min(100),
        ..Default::default()
    };
    let (skills, total) = state.db.search_skills(&query)?;
    let response = PaginatedResponse::new(skills, total, query.page, query.per_page);
    Ok(ApiResponse::ok(response))
}

async fn featured_skills(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    let skills = state.db.get_featured_skills(12)?;
    Ok(ApiResponse::ok(skills))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<impl IntoResponse, MarketplaceError> {
    // Simplified auth: in production, verify hashed password against DB
    // Here we return a placeholder JWT structure
    let _ = state.jwt_secret.as_str(); // used in real JWT signing
    warn!("Login attempt for user '{}' - using placeholder auth", req.username);

    // Real implementation would: query developer by username, verify bcrypt hash,
    // then sign a JWT with exp claim using the jwt_secret
    let token = format!("placeholder-jwt-for-{}", req.username);
    let developer_id = Uuid::new_v4().to_string();

    Ok(Json(LoginResponse {
        token,
        developer_id,
        expires_in: 86400,
    }))
}

// ── Decode helpers ────────────────────────────────────────────────────────────

fn base64_decode(s: &str) -> std::result::Result<Vec<u8>, String> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
    if s.bytes().all(|b| alphabet.contains(&b)) {
        // Simplified: decode manually
        decode_base64_simple(s)
    } else {
        Err("invalid base64 characters".to_string())
    }
}

fn decode_base64_simple(input: &str) -> std::result::Result<Vec<u8>, String> {
    let chars: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    let table: &[i8; 128] = &{
        let mut t = [-1i8; 128];
        for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            t[c as usize] = i as i8;
        }
        t
    };

    let mut out = Vec::with_capacity(chars.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < chars.len() {
        let a = table.get(chars[i] as usize).copied().unwrap_or(-1);
        let b = table.get(chars[i+1] as usize).copied().unwrap_or(-1);
        let c = table.get(chars[i+2] as usize).copied().unwrap_or(-1);
        let d = table.get(chars[i+3] as usize).copied().unwrap_or(-1);
        if a < 0 || b < 0 || c < 0 || d < 0 {
            return Err("invalid base64 character".to_string());
        }
        let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
        out.push(n as u8);
        i += 4;
    }
    Ok(out)
}

fn hex_decode(s: &str) -> std::result::Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
