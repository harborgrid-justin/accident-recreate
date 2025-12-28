//! Error types for the search engine

use std::fmt;

pub type SearchResult<T> = Result<T, SearchError>;

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query parsing error: {0}")]
    QueryParseError(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid field: {0}")]
    InvalidField(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Search timeout")]
    Timeout,

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Index corrupted: {0}")]
    CorruptedIndex(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

impl SearchError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SearchError::Timeout
                | SearchError::ResourceLimitExceeded(_)
                | SearchError::QueryParseError(_)
                | SearchError::InvalidQuery(_)
        )
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            SearchError::IndexError(_) => "INDEX_ERROR",
            SearchError::QueryParseError(_) => "QUERY_PARSE_ERROR",
            SearchError::SchemaError(_) => "SCHEMA_ERROR",
            SearchError::DocumentNotFound(_) => "DOCUMENT_NOT_FOUND",
            SearchError::InvalidField(_) => "INVALID_FIELD",
            SearchError::SerializationError(_) => "SERIALIZATION_ERROR",
            SearchError::IoError(_) => "IO_ERROR",
            SearchError::TantivyError(_) => "TANTIVY_ERROR",
            SearchError::JsonError(_) => "JSON_ERROR",
            SearchError::ConfigError(_) => "CONFIG_ERROR",
            SearchError::Timeout => "TIMEOUT",
            SearchError::InvalidQuery(_) => "INVALID_QUERY",
            SearchError::CorruptedIndex(_) => "CORRUPTED_INDEX",
            SearchError::ResourceLimitExceeded(_) => "RESOURCE_LIMIT_EXCEEDED",
        }
    }
}
