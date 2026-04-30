use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpiceError {
    #[error("Agent error: {0}")]
    AgentError(String),

    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
