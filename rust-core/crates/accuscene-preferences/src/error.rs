//! Error types for the preferences system

use thiserror::Error;

/// Result type for preferences operations
pub type Result<T> = std::result::Result<T, PreferencesError>;

/// Errors that can occur in the preferences system
#[derive(Error, Debug)]
pub enum PreferencesError {
    /// Key not found
    #[error("Preference key not found: {0}")]
    KeyNotFound(String),

    /// Invalid value type
    #[error("Invalid value type for key {key}: expected {expected}, got {actual}")]
    InvalidValueType {
        key: String,
        expected: String,
        actual: String,
    },

    /// Validation error
    #[error("Validation error for key {key}: {message}")]
    ValidationError { key: String, message: String },

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Sync error
    #[error("Sync error: {0}")]
    SyncError(String),

    /// Sync disabled
    #[error("Sync is disabled in configuration")]
    SyncDisabled,

    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Export error
    #[error("Export error: {0}")]
    ExportError(String),

    /// Import error
    #[error("Import error: {0}")]
    ImportError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Conflict error (for sync conflicts)
    #[error("Conflict detected for key {key}: local version {local_version}, remote version {remote_version}")]
    ConflictError {
        key: String,
        local_version: u64,
        remote_version: u64,
    },

    /// Schema error
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for PreferencesError {
    fn from(err: serde_json::Error) -> Self {
        PreferencesError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for PreferencesError {
    fn from(err: toml::de::Error) -> Self {
        PreferencesError::SerializationError(err.to_string())
    }
}

impl From<toml::ser::Error> for PreferencesError {
    fn from(err: toml::ser::Error) -> Self {
        PreferencesError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for PreferencesError {
    fn from(err: reqwest::Error) -> Self {
        PreferencesError::NetworkError(err.to_string())
    }
}

impl PreferencesError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PreferencesError::NetworkError(_)
                | PreferencesError::SyncError(_)
                | PreferencesError::ConflictError { .. }
        )
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            PreferencesError::KeyNotFound(_) => "KEY_NOT_FOUND",
            PreferencesError::InvalidValueType { .. } => "INVALID_VALUE_TYPE",
            PreferencesError::ValidationError { .. } => "VALIDATION_ERROR",
            PreferencesError::StorageError(_) => "STORAGE_ERROR",
            PreferencesError::SyncError(_) => "SYNC_ERROR",
            PreferencesError::SyncDisabled => "SYNC_DISABLED",
            PreferencesError::EncryptionError(_) => "ENCRYPTION_ERROR",
            PreferencesError::DecryptionError(_) => "DECRYPTION_ERROR",
            PreferencesError::MigrationError(_) => "MIGRATION_ERROR",
            PreferencesError::ExportError(_) => "EXPORT_ERROR",
            PreferencesError::ImportError(_) => "IMPORT_ERROR",
            PreferencesError::IoError(_) => "IO_ERROR",
            PreferencesError::SerializationError(_) => "SERIALIZATION_ERROR",
            PreferencesError::NetworkError(_) => "NETWORK_ERROR",
            PreferencesError::ConfigError(_) => "CONFIG_ERROR",
            PreferencesError::PermissionDenied(_) => "PERMISSION_DENIED",
            PreferencesError::ConflictError { .. } => "CONFLICT_ERROR",
            PreferencesError::SchemaError(_) => "SCHEMA_ERROR",
            PreferencesError::Unknown(_) => "UNKNOWN_ERROR",
        }
    }
}
