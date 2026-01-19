use thiserror::Error;

/// Custom error type for Graph API related errors
#[derive(Error, Debug)]
pub enum CirroError {
    #[error("ARM API error: {0}")]
    ArmApiError(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Log setup error: {0}")]
    LogSetupError(#[from] log::SetLoggerError),

    #[error("Multiple errors: {0:?}")]
    MultipleErrors(Vec<CirroError>),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("OData error: {0}")]
    ODataError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Semaphore error: {0}")]
    SemaphoreError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("Token expired")]
    TokenExpired,

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Unsupported HTTP method: {0}")]
    UnsupportedHttpMethod(reqwest::Method),
}
