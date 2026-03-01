use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub license: String,
    pub category: SkillCategory,
    pub tags: Vec<String>,
    pub permissions_required: Vec<String>,
    pub download_count: u64,
    pub rating_avg: f32,
    pub rating_count: u32,
    pub price_cents: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub review_status: ReviewStatus,
}

impl Skill {
    pub fn is_free(&self) -> bool {
        self.price_cents == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Productivity,
    Development,
    Security,
    DataAnalysis,
    Communication,
    Automation,
    Integration,
    Utilities,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Productivity => "productivity",
            Self::Development => "development",
            Self::Security => "security",
            Self::DataAnalysis => "data_analysis",
            Self::Communication => "communication",
            Self::Automation => "automation",
            Self::Integration => "integration",
            Self::Utilities => "utilities",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for SkillCategory {
    type Err = crate::error::MarketplaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "productivity" => Ok(Self::Productivity),
            "development" => Ok(Self::Development),
            "security" => Ok(Self::Security),
            "data_analysis" => Ok(Self::DataAnalysis),
            "communication" => Ok(Self::Communication),
            "automation" => Ok(Self::Automation),
            "integration" => Ok(Self::Integration),
            "utilities" => Ok(Self::Utilities),
            other => Err(crate::error::MarketplaceError::InvalidCategory(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    AutoScanPassed,
    AutoScanFailed,
    CommunityReview,
    CommunityApproved,
    StaffReview,
    Approved,
    Rejected { reason: String },
}

impl ReviewStatus {
    pub fn as_db_str(&self) -> String {
        match self {
            Self::Pending => "pending".to_string(),
            Self::AutoScanPassed => "auto_scan_passed".to_string(),
            Self::AutoScanFailed => "auto_scan_failed".to_string(),
            Self::CommunityReview => "community_review".to_string(),
            Self::CommunityApproved => "community_approved".to_string(),
            Self::StaffReview => "staff_review".to_string(),
            Self::Approved => "approved".to_string(),
            Self::Rejected { reason } => format!("rejected:{reason}"),
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        if let Some(reason) = s.strip_prefix("rejected:") {
            return Self::Rejected { reason: reason.to_string() };
        }
        match s {
            "pending" => Self::Pending,
            "auto_scan_passed" => Self::AutoScanPassed,
            "auto_scan_failed" => Self::AutoScanFailed,
            "community_review" => Self::CommunityReview,
            "community_approved" => Self::CommunityApproved,
            "staff_review" => Self::StaffReview,
            "approved" => Self::Approved,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPackage {
    pub metadata: Skill,
    pub wasm_bytes: Vec<u8>,
    pub signature: Vec<u8>,
    pub signer_public_key: Vec<u8>,
    pub manifest_hash: String,
}

impl SkillPackage {
    pub fn wasm_size_bytes(&self) -> usize {
        self.wasm_bytes.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub reviewer_id: Uuid,
    pub rating: u8,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub is_community_review: bool,
}

impl Review {
    pub fn validate_rating(&self) -> bool {
        (1..=5).contains(&self.rating)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRating {
    pub skill_id: Uuid,
    pub avg_rating: f32,
    pub total_reviews: u32,
    pub distribution: HashMap<u8, u32>,
}

impl SkillRating {
    pub fn compute(skill_id: Uuid, reviews: &[Review]) -> Self {
        let mut distribution: HashMap<u8, u32> = HashMap::new();
        let mut total = 0f32;

        for review in reviews {
            *distribution.entry(review.rating).or_insert(0) += 1;
            total += review.rating as f32;
        }

        let total_reviews = reviews.len() as u32;
        let avg_rating = if total_reviews > 0 {
            total / total_reviews as f32
        } else {
            0.0
        };

        Self {
            skill_id,
            avg_rating,
            total_reviews,
            distribution,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub skills_published: u32,
    pub total_downloads: u64,
    pub total_revenue_cents: u64,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Platform fee in basis points (1500 = 15%)
    pub platform_fee_bps: u32,
    pub min_payout_cents: u32,
    pub stripe_api_key_encrypted: Option<String>,
    pub cdn_base_url: String,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            platform_fee_bps: 1500,
            min_payout_cents: 1000,
            stripe_api_key_encrypted: None,
            cdn_base_url: "https://cdn.openzax.dev".to_string(),
        }
    }
}

impl MarketplaceConfig {
    pub fn platform_fee_fraction(&self) -> f64 {
        self.platform_fee_bps as f64 / 10_000.0
    }

    pub fn developer_payout_cents(&self, price_cents: u32) -> u32 {
        let fee = (price_cents as f64 * self.platform_fee_fraction()) as u32;
        price_cents.saturating_sub(fee)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchQuery {
    pub query: Option<String>,
    pub category: Option<SkillCategory>,
    pub tags: Vec<String>,
    pub max_price_cents: Option<u32>,
    pub free_only: bool,
    pub sort: SkillSortOrder,
    pub page: u32,
    pub per_page: u32,
}

impl Default for SkillSearchQuery {
    fn default() -> Self {
        Self {
            query: None,
            category: None,
            tags: vec![],
            max_price_cents: None,
            free_only: false,
            sort: SkillSortOrder::Trending,
            page: 1,
            per_page: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillSortOrder {
    #[default]
    Trending,
    Newest,
    Rating,
    Downloads,
    PriceLow,
    PriceHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let total_pages = ((total as f64) / per_page as f64).ceil() as u32;
        Self { items, total, page, per_page, total_pages }
    }
}
