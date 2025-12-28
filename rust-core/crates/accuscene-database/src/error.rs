//! Database-specific error types for AccuScene Enterprise
//!
//! Comprehensive error handling for all database operations including
//! connection errors, query errors, transaction errors, and migration errors.

use thiserror::Error;

/// Result type alias for database operations
pub type DbResult<T> = Result<T, DatabaseError>;

/// Comprehensive database error enumeration
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Connection pool errors
    #[error("Connection pool error: {0}")]
    PoolError(String),

    /// Connection errors
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// Query execution errors
    #[error("Query execution failed: {0}")]
    QueryError(String),

    /// Transaction errors
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Migration errors
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Schema errors
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// Record not found
    #[error("Record not found: {entity} with {field}={value}")]
    NotFound {
        entity: String,
        field: String,
        value: String,
    },

    /// Duplicate record
    #[error("Duplicate record: {entity} with {field}={value}")]
    DuplicateRecord {
        entity: String,
        field: String,
        value: String,
    },

    /// Constraint violation
    #[error("Constraint violation: {constraint} on {table}")]
    ConstraintViolation { constraint: String, table: String },

    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Backup/Restore errors
    #[error("Backup error: {0}")]
    BackupError(String),

    /// Audit logging errors
    #[error("Audit logging error: {0}")]
    AuditError(String),

    /// Full-text search errors
    #[error("Search error: {0}")]
    SearchError(String),

    /// Lock timeout
    #[error("Lock timeout: {0}")]
    LockTimeout(String),

    /// Deadlock detected
    #[error("Deadlock detected: {0}")]
    Deadlock(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic database error
    #[error("Database error: {0}")]
    Other(String),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound {
                entity: "Record".to_string(),
                field: "query".to_string(),
                value: "N/A".to_string(),
            },
            rusqlite::Error::SqliteFailure(err, msg) => {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    DatabaseError::ConstraintViolation {
                        constraint: msg.unwrap_or_default(),
                        table: "Unknown".to_string(),
                    }
                } else {
                    DatabaseError::QueryError(format!("{}: {:?}", err.code, msg))
                }
            }
            _ => DatabaseError::QueryError(err.to_string()),
        }
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(err: r2d2::Error) -> Self {
        DatabaseError::PoolError(err.to_string())
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        DatabaseError::SerializationError(err.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound {
                entity: "Record".to_string(),
                field: "query".to_string(),
                value: "N/A".to_string(),
            },
            sqlx::Error::Database(db_err) => {
                DatabaseError::QueryError(db_err.to_string())
            }
            _ => DatabaseError::QueryError(err.to_string()),
        }
    }
}

impl DatabaseError {
    /// Check if error is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, DatabaseError::NotFound { .. })
    }

    /// Check if error is a duplicate record error
    pub fn is_duplicate(&self) -> bool {
        matches!(self, DatabaseError::DuplicateRecord { .. })
    }

    /// Check if error is a constraint violation
    pub fn is_constraint_violation(&self) -> bool {
        matches!(self, DatabaseError::ConstraintViolation { .. })
    }

    /// Check if error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            DatabaseError::LockTimeout(_)
                | DatabaseError::Deadlock(_)
                | DatabaseError::ConnectionError(_)
        )
    }

    /// Create a not found error
    pub fn not_found(entity: impl Into<String>, field: impl Into<String>, value: impl Into<String>) -> Self {
        DatabaseError::NotFound {
            entity: entity.into(),
            field: field.into(),
            value: value.into(),
        }
    }

    /// Create a duplicate record error
    pub fn duplicate(entity: impl Into<String>, field: impl Into<String>, value: impl Into<String>) -> Self {
        DatabaseError::DuplicateRecord {
            entity: entity.into(),
            field: field.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_not_found() {
        let err = DatabaseError::not_found("User", "id", "123");
        assert!(err.is_not_found());
        assert!(!err.is_duplicate());
    }

    #[test]
    fn test_error_is_duplicate() {
        let err = DatabaseError::duplicate("User", "email", "test@example.com");
        assert!(err.is_duplicate());
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_error_is_transient() {
        let err = DatabaseError::LockTimeout("Timeout after 5s".to_string());
        assert!(err.is_transient());

        let err = DatabaseError::QueryError("Some error".to_string());
        assert!(!err.is_transient());
    }
}
