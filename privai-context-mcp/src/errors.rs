use thiserror::Error;

pub type Result<T> = std::result::Result<T, McpError>;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("guardrail violation: {0}")]
    GuardrailViolation(String),

    #[error("unsupported operation in scaffold: {0}")]
    Unsupported(String),
}
