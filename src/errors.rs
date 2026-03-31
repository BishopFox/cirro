use thiserror::Error;

/// Custom error type for Cirro
#[derive(Error, Debug)]
pub enum CirroError {
    #[error("ARM API error: {0}")]
    ArmApiError(String),

    #[error("Auth error: {0}")]
    AuthError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("DNS resolver error: {0}")]
    DnsResolverError(String),

    #[error("Domain check error: {0}")]
    DomainCheckError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Log setup error: {0}")]
    LogSetupError(String),

    #[error("Multiple errors: {0:?}")]
    MultipleErrors(Vec<CirroError>),

    #[cfg(feature = "graph")]
    #[error("Neo4j error: {0}")]
    Neo4jError(#[from] neo4rs::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("OData error: {0}")]
    ODataError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[cfg(feature = "collector")]
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Semaphore error: {0}")]
    SemaphoreError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Sqlite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[cfg(feature = "graph")]
    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),

    #[error("Token expired")]
    TokenExpired,

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Unsupported HTTP method: {0}")]
    UnsupportedHttpMethod(String),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("WHOIS error: {0}")]
    WhoisError(String),

    #[cfg(feature = "graph")]
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl From<fern::InitError> for CirroError {
    fn from(e: fern::InitError) -> Self {
        CirroError::LogSetupError(e.to_string())
    }
}

#[cfg(feature = "collector")]
impl From<reqwest::Method> for CirroError {
    fn from(method: reqwest::Method) -> Self {
        CirroError::UnsupportedHttpMethod(method.to_string())
    }
}
