//! AccuScene Enterprise Database Layer
//!
//! High-performance database layer with connection pooling, migrations, repositories,
//! and enterprise features for the AccuScene accident recreation platform.
//!
//! # Features
//!
//! - **Connection Pooling**: Efficient connection management with r2d2
//! - **Migrations**: Automatic schema migrations with version tracking
//! - **Repositories**: Repository pattern for type-safe data access
//! - **Query Builder**: Type-safe query construction with filtering and pagination
//! - **Transactions**: Transaction support with automatic rollback and retry logic
//! - **Audit Logging**: Comprehensive audit trail for compliance
//! - **Full-Text Search**: FTS5-powered search with ranking and snippets
//! - **Backup/Restore**: Database backup and restore functionality
//!
//! # Example
//!
//! ```rust,no_run
//! use accuscene_database::{DatabaseConfig, DatabasePool, migrations::MigrationRunner};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create database configuration
//! let config = DatabaseConfig::new("accuscene.db");
//!
//! // Create connection pool
//! let pool = DatabasePool::new(config)?;
//!
//! // Run migrations
//! let runner = MigrationRunner::new();
//! let mut conn = pool.get()?;
//! runner.migrate(&mut conn)?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Core modules
pub mod error;
pub mod config;
pub mod pool;
pub mod connection;

// Database schema and migrations
pub mod migrations;
pub mod schema;

// Data access layer
pub mod repositories;
pub mod query;

// Enterprise features
pub mod transaction;
pub mod backup;
pub mod audit;
pub mod search;

// Re-export commonly used types
pub use error::{DatabaseError, DbResult};
pub use config::{
    DatabaseConfig, PoolConfig, PerformanceConfig, FeatureConfig, BackupConfig,
    SynchronousMode, AutoVacuum, JournalMode,
};
pub use pool::{DatabasePool, PoolState, PoolStats};
pub use connection::{DbConnection, DbTransaction, IsolationLevel};

// Re-export migration types
pub use migrations::{Migration, MigrationRegistry, MigrationHistory};
pub use migrations::runner::MigrationRunner;

// Re-export repository types
pub use repositories::{
    Repository,
    CaseRepository, AccidentRepository, VehicleRepository,
    EvidenceRepository, UserRepository,
};

// Re-export repository entity types
pub use repositories::case::Case;
pub use repositories::accident::Accident;
pub use repositories::vehicle::Vehicle;
pub use repositories::evidence::Evidence;
pub use repositories::user::User;

// Re-export query types
pub use query::{
    QueryBuilder, Filter, FilterOperator, FilterCondition, FilterValue,
    Pagination, PaginationResult, CursorPagination, CursorPaginationResult,
};

// Re-export transaction types
pub use transaction::{TransactionOptions, TransactionManager, execute_transaction, with_transaction};

// Re-export backup types
pub use backup::{BackupManager, BackupInfo};

// Re-export audit types
pub use audit::{AuditLogger, AuditEntry, AuditAction, ChangeValue};

// Re-export search types
pub use search::{SearchManager, SearchResult, SearchOptions, SearchQuery};

/// Database version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the database with default configuration
///
/// This is a convenience function that creates a database pool,
/// runs migrations, and returns the pool ready for use.
///
/// # Example
///
/// ```rust,no_run
/// use accuscene_database::initialize_database;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = initialize_database("accuscene.db")?;
/// # Ok(())
/// # }
/// ```
pub fn initialize_database(path: impl Into<String>) -> DbResult<DatabasePool> {
    let config = DatabaseConfig::new(path);
    initialize_database_with_config(config)
}

/// Initialize the database with custom configuration
///
/// # Example
///
/// ```rust,no_run
/// use accuscene_database::{initialize_database_with_config, DatabaseConfig};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = DatabaseConfig::high_performance("accuscene.db");
/// let pool = initialize_database_with_config(config)?;
/// # Ok(())
/// # }
/// ```
pub fn initialize_database_with_config(config: DatabaseConfig) -> DbResult<DatabasePool> {
    use tracing::info;

    info!("Initializing AccuScene database v{}", VERSION);

    // Validate configuration
    config.validate()?;

    // Create connection pool
    let pool = DatabasePool::new(config.clone())?;

    // Run migrations if enabled
    if config.features.auto_migrate {
        info!("Running database migrations");
        let runner = MigrationRunner::new();
        let mut conn = pool.get()?;
        runner.migrate(&mut *conn)?;
    }

    info!("Database initialized successfully");

    Ok(pool)
}

/// Database manager for high-level database operations
///
/// Provides a unified interface for all database operations including
/// repositories, migrations, backups, and search.
pub struct DatabaseManager {
    pool: DatabasePool,
    audit_logger: AuditLogger,
    search_manager: SearchManager,
    backup_manager: BackupManager,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(config: DatabaseConfig) -> DbResult<Self> {
        let backup_dir = config.backup.directory.clone();
        let pool = initialize_database_with_config(config.clone())?;

        Ok(Self {
            pool,
            audit_logger: if config.features.audit_logging {
                AuditLogger::new()
            } else {
                AuditLogger::disabled()
            },
            search_manager: SearchManager::new(),
            backup_manager: BackupManager::new(backup_dir),
        })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    /// Get the audit logger
    pub fn audit_logger(&self) -> &AuditLogger {
        &self.audit_logger
    }

    /// Get the search manager
    pub fn search_manager(&self) -> &SearchManager {
        &self.search_manager
    }

    /// Get the backup manager
    pub fn backup_manager(&self) -> &BackupManager {
        &self.backup_manager
    }

    /// Get a connection from the pool
    pub fn get_connection(&self) -> DbResult<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
        self.pool.get()
    }

    /// Perform a health check
    pub fn health_check(&self) -> DbResult<()> {
        self.pool.health_check()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.pool.stats()
    }

    /// Create a backup
    pub fn create_backup(&self, name: Option<&str>) -> DbResult<std::path::PathBuf> {
        let conn = self.get_connection()?;
        self.backup_manager.backup(&conn, name)
    }

    /// Get case repository
    pub fn cases(&self) -> CaseRepository {
        CaseRepository::new()
    }

    /// Get accident repository
    pub fn accidents(&self) -> AccidentRepository {
        AccidentRepository::new()
    }

    /// Get vehicle repository
    pub fn vehicles(&self) -> VehicleRepository {
        VehicleRepository::new()
    }

    /// Get evidence repository
    pub fn evidence(&self) -> EvidenceRepository {
        EvidenceRepository::new()
    }

    /// Get user repository
    pub fn users(&self) -> UserRepository {
        UserRepository::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::test();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_in_memory_database() {
        let config = DatabaseConfig::in_memory();
        let pool = DatabasePool::new(config).unwrap();
        assert!(pool.health_check().is_ok());
    }

    #[test]
    fn test_database_manager() {
        let config = DatabaseConfig::test();
        let manager = DatabaseManager::new(config).unwrap();
        assert!(manager.health_check().is_ok());
    }
}
