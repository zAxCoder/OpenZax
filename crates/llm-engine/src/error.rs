use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Model loading error: {0}")]
    ModelLoad(String),

    #[error("Inference error: {0}")]
    Inference(String),

    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Context window exceeded: {current} > {max}")]
    ContextWindowExceeded { current: usize, max: usize },

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Invalid model format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type LlmResult<T> = std::result::Result<T, LlmError>;
