use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("Skill not found: {0}")]
    SkillNotFound(Uuid),

    #[error("Developer not found: {0}")]
    DeveloperNotFound(Uuid),

    #[error("Review not found: {0}")]
    ReviewNotFound(Uuid),

    #[error("Invalid category: {0}")]
    InvalidCategory(String),

    #[error("Invalid rating: must be between 1 and 5")]
    InvalidRating,

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Signature verification failed: {0}")]
    SignatureError(String),

    #[error("Scan failed: {0}")]
    ScanError(String),

    #[error("Package too large: {size} bytes exceeds limit of {limit} bytes")]
    PackageTooLarge { size: usize, limit: usize },

    #[error("Invalid WASM: {0}")]
    InvalidWasm(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, MarketplaceError>;

impl axum::response::IntoResponse for MarketplaceError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        let (status, message) = match &self {
            Self::SkillNotFound(_) | Self::DeveloperNotFound(_) | Self::ReviewNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::AuthenticationRequired => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),
            Self::InvalidCategory(_) | Self::InvalidRating | Self::InvalidWasm(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            Self::PackageTooLarge { .. } => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = serde_json::json!({
            "error": {
                "message": message,
                "code": status.as_u16(),
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
