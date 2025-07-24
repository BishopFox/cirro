use thiserror::Error;

/// Custom error type for Graph API related errors
#[derive(Error, Debug)]
pub enum CirroIngestError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("Log setup error: {0}")]
    LogSetupError(#[from] fern::InitError),

    #[error("Multiple errors: {0:?}")]
    MultipleErrors(Vec<CirroIngestError>),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Neo4j error: {0}")]
    Neo4jError(#[from] neo4rs::Error),
}
