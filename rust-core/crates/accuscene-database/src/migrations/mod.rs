//! Database migration system for AccuScene Enterprise
//!
//! Provides automatic schema migrations with version tracking,
//! rollback support, and migration history.

pub mod runner;
pub mod v001_initial;

use crate::error::{DatabaseError, DbResult};
use rusqlite::Connection;
use std::collections::HashMap;
use tracing::{debug, info};

/// Represents a single migration
pub trait Migration: Send + Sync {
    /// Get the migration version
    fn version(&self) -> u32;

    /// Get the migration name
    fn name(&self) -> &str;

    /// Execute the migration (upgrade)
    fn up(&self, conn: &mut Connection) -> DbResult<()>;

    /// Rollback the migration (downgrade)
    fn down(&self, conn: &mut Connection) -> DbResult<()>;

    /// Get migration description
    fn description(&self) -> &str {
        ""
    }
}

/// Migration registry
pub struct MigrationRegistry {
    migrations: HashMap<u32, Box<dyn Migration>>,
}

impl MigrationRegistry {
    /// Create a new migration registry
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
        }
    }

    /// Register a migration
    pub fn register(&mut self, migration: Box<dyn Migration>) {
        let version = migration.version();
        debug!("Registering migration v{}: {}", version, migration.name());
        self.migrations.insert(version, migration);
    }

    /// Get all migrations in order
    pub fn all_migrations(&self) -> Vec<&dyn Migration> {
        let mut migrations: Vec<_> = self.migrations.values().map(|m| m.as_ref()).collect();
        migrations.sort_by_key(|m| m.version());
        migrations
    }

    /// Get a specific migration by version
    pub fn get_migration(&self, version: u32) -> Option<&dyn Migration> {
        self.migrations.get(&version).map(|m| m.as_ref())
    }

    /// Get migrations to apply (versions greater than current)
    pub fn pending_migrations(&self, current_version: u32) -> Vec<&dyn Migration> {
        let mut migrations: Vec<_> = self
            .migrations
            .values()
            .filter(|m| m.version() > current_version)
            .map(|m| m.as_ref())
            .collect();
        migrations.sort_by_key(|m| m.version());
        migrations
    }

    /// Get the latest migration version
    pub fn latest_version(&self) -> u32 {
        self.migrations
            .keys()
            .max()
            .copied()
            .unwrap_or(0)
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register all migrations
        registry.register(Box::new(v001_initial::InitialMigration));

        info!(
            "Registered {} migrations, latest version: {}",
            registry.migrations.len(),
            registry.latest_version()
        );

        registry
    }
}

/// Migration history entry
#[derive(Debug, Clone)]
pub struct MigrationHistory {
    pub version: u32,
    pub name: String,
    pub applied_at: String,
    pub execution_time_ms: u64,
}

/// Get the current database version
pub fn get_current_version(conn: &Connection) -> DbResult<u32> {
    // Check if migrations table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !table_exists {
        debug!("Migrations table does not exist, current version is 0");
        return Ok(0);
    }

    // Get the latest applied migration version
    let version = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get::<_, u32>(0),
        )
        .unwrap_or(0);

    debug!("Current database version: {}", version);

    Ok(version)
}

/// Get migration history
pub fn get_migration_history(conn: &Connection) -> DbResult<Vec<MigrationHistory>> {
    // Check if migrations table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !table_exists {
        return Ok(Vec::new());
    }

    let mut stmt = conn
        .prepare("SELECT version, name, applied_at, execution_time_ms FROM _migrations ORDER BY version")
        .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

    let history = stmt
        .query_map([], |row| {
            Ok(MigrationHistory {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: row.get(2)?,
                execution_time_ms: row.get(3)?,
            })
        })
        .map_err(|e| DatabaseError::MigrationError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMigration {
        version: u32,
        name: String,
    }

    impl Migration for TestMigration {
        fn version(&self) -> u32 {
            self.version
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn up(&self, _conn: &mut Connection) -> DbResult<()> {
            Ok(())
        }

        fn down(&self, _conn: &mut Connection) -> DbResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(TestMigration {
            version: 1,
            name: "test1".to_string(),
        }));
        registry.register(Box::new(TestMigration {
            version: 2,
            name: "test2".to_string(),
        }));

        assert_eq!(registry.latest_version(), 2);
        assert_eq!(registry.all_migrations().len(), 2);
    }

    #[test]
    fn test_pending_migrations() {
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(TestMigration {
            version: 1,
            name: "test1".to_string(),
        }));
        registry.register(Box::new(TestMigration {
            version: 2,
            name: "test2".to_string(),
        }));
        registry.register(Box::new(TestMigration {
            version: 3,
            name: "test3".to_string(),
        }));

        let pending = registry.pending_migrations(1);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].version(), 2);
        assert_eq!(pending[1].version(), 3);
    }
}
