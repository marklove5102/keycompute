use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyComputeError {
    #[error("authentication failed: {0}")]
    AuthError(String),

    #[error("rate limit exceeded")]
    RateLimitExceeded,

    #[error("routing failed: no available provider")]
    RoutingFailed,

    #[error("upstream provider error: {0}")]
    ProviderError(String),

    #[error("database error: {0}")]
    DbError(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, KeyComputeError>;

impl From<serde_json::Error> for KeyComputeError {
    fn from(err: serde_json::Error) -> Self {
        KeyComputeError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for KeyComputeError {
    fn from(err: std::io::Error) -> Self {
        KeyComputeError::Internal(err.to_string())
    }
}
