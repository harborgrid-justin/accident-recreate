//! Migration runner for executing database migrations
//!
//! Handles applying and rolling back migrations with transaction support,
//! error handling, and migration history tracking.

use super::{get_current_version, Migration, MigrationRegistry};
use crate::error::{DatabaseError, DbResult};
use rusqlite::Connection;
use std::time::Instant;
use tracing::{error, info, warn};

/// Migration runner
pub struct MigrationRunner {
    registry: MigrationRegistry,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new() -> Self {
        Self {
            registry: MigrationRegistry::default(),
        }
    }

    /// Create a migration runner with a custom registry
    pub fn with_registry(registry: MigrationRegistry) -> Self {
        Self { registry }
    }

    /// Initialize the migrations table
    fn initialize_migrations_table(conn: &mut Connection) -> DbResult<()> {
        info!("Initializing migrations table");

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now')),
                execution_time_ms INTEGER NOT NULL,
                checksum TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_migrations_applied_at
            ON _migrations(applied_at);
            "#,
        )
        .map_err(|e| DatabaseError::MigrationError(format!("Failed to initialize migrations table: {}", e)))?;

        Ok(())
    }

    /// Record a migration in the history
    fn record_migration(
        conn: &mut Connection,
        version: u32,
        name: &str,
        execution_time_ms: u64,
    ) -> DbResult<()> {
        conn.execute(
            "INSERT INTO _migrations (version, name, execution_time_ms) VALUES (?, ?, ?)",
            [&version.to_string(), name, &execution_time_ms.to_string()],
        )
        .map_err(|e| DatabaseError::MigrationError(format!("Failed to record migration: {}", e)))?;

        Ok(())
    }

    /// Remove a migration from the history
    fn remove_migration(conn: &mut Connection, version: u32) -> DbResult<()> {
        conn.execute("DELETE FROM _migrations WHERE version = ?", [version])
            .map_err(|e| DatabaseError::MigrationError(format!("Failed to remove migration: {}", e)))?;

        Ok(())
    }

    /// Run all pending migrations
    pub fn migrate(&self, conn: &mut Connection) -> DbResult<()> {
        info!("Starting database migration");

        // Initialize migrations table
        Self::initialize_migrations_table(conn)?;

        // Get current version
        let current_version = get_current_version(conn)?;
        info!("Current database version: {}", current_version);

        // Get pending migrations
        let pending = self.registry.pending_migrations(current_version);

        if pending.is_empty() {
            info!("Database is up to date, no migrations to apply");
            return Ok(());
        }

        info!("Found {} pending migrations", pending.len());

        // Apply each migration in a transaction
        for migration in pending {
            self.apply_migration(conn, migration)?;
        }

        let new_version = get_current_version(conn)?;
        info!(
            "Migration complete. Database upgraded from v{} to v{}",
            current_version, new_version
        );

        Ok(())
    }

    /// Apply a single migration
    fn apply_migration(&self, conn: &mut Connection, migration: &dyn Migration) -> DbResult<()> {
        let version = migration.version();
        let name = migration.name();

        info!("Applying migration v{}: {}", version, name);

        let start = Instant::now();

        // Use a transaction for safety
        let tx = conn
            .transaction()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // Execute the migration
        if let Err(e) = migration.up(&mut tx.into()) {
            error!("Migration v{} failed: {}", version, e);
            return Err(DatabaseError::MigrationError(format!(
                "Migration v{} failed: {}",
                version, e
            )));
        }

        // Record the migration
        let execution_time = start.elapsed().as_millis() as u64;
        Self::record_migration(&mut tx.into(), version, name, execution_time)?;

        // Commit the transaction
        tx.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        info!(
            "Migration v{} applied successfully in {}ms",
            version, execution_time
        );

        Ok(())
    }

    /// Migrate to a specific version
    pub fn migrate_to(&self, conn: &mut Connection, target_version: u32) -> DbResult<()> {
        info!("Migrating to version {}", target_version);

        Self::initialize_migrations_table(conn)?;

        let current_version = get_current_version(conn)?;

        if current_version == target_version {
            info!("Already at target version {}", target_version);
            return Ok(());
        }

        if current_version < target_version {
            // Upgrade
            let migrations = self.registry.pending_migrations(current_version);
            for migration in migrations {
                if migration.version() <= target_version {
                    self.apply_migration(conn, migration)?;
                }
            }
        } else {
            // Downgrade
            let mut migrations = self.registry.all_migrations();
            migrations.retain(|m| m.version() > target_version && m.version() <= current_version);
            migrations.sort_by_key(|m| std::cmp::Reverse(m.version()));

            for migration in migrations {
                self.rollback_migration(conn, migration)?;
            }
        }

        Ok(())
    }

    /// Rollback a single migration
    fn rollback_migration(&self, conn: &mut Connection, migration: &dyn Migration) -> DbResult<()> {
        let version = migration.version();
        let name = migration.name();

        warn!("Rolling back migration v{}: {}", version, name);

        let start = Instant::now();

        // Use a transaction for safety
        let tx = conn
            .transaction()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // Execute the rollback
        if let Err(e) = migration.down(&mut tx.into()) {
            error!("Rollback of v{} failed: {}", version, e);
            return Err(DatabaseError::MigrationError(format!(
                "Rollback of v{} failed: {}",
                version, e
            )));
        }

        // Remove from migration history
        Self::remove_migration(&mut tx.into(), version)?;

        // Commit the transaction
        tx.commit()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        let execution_time = start.elapsed().as_millis();
        info!(
            "Migration v{} rolled back successfully in {}ms",
            version, execution_time
        );

        Ok(())
    }

    /// Rollback the last migration
    pub fn rollback(&self, conn: &mut Connection) -> DbResult<()> {
        info!("Rolling back last migration");

        Self::initialize_migrations_table(conn)?;

        let current_version = get_current_version(conn)?;

        if current_version == 0 {
            info!("No migrations to rollback");
            return Ok(());
        }

        if let Some(migration) = self.registry.get_migration(current_version) {
            self.rollback_migration(conn, migration)?;
        } else {
            return Err(DatabaseError::MigrationError(format!(
                "Migration v{} not found in registry",
                current_version
            )));
        }

        Ok(())
    }

    /// Rollback all migrations
    pub fn rollback_all(&self, conn: &mut Connection) -> DbResult<()> {
        info!("Rolling back all migrations");

        while get_current_version(conn)? > 0 {
            self.rollback(conn)?;
        }

        info!("All migrations rolled back");

        Ok(())
    }

    /// Get the migration registry
    pub fn registry(&self) -> &MigrationRegistry {
        &self.registry
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMigration;

    impl Migration for TestMigration {
        fn version(&self) -> u32 {
            1
        }

        fn name(&self) -> &str {
            "test_migration"
        }

        fn up(&self, conn: &mut Connection) -> DbResult<()> {
            conn.execute_batch("CREATE TABLE test (id INTEGER PRIMARY KEY)")
                .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
            Ok(())
        }

        fn down(&self, conn: &mut Connection) -> DbResult<()> {
            conn.execute_batch("DROP TABLE IF EXISTS test")
                .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
            Ok(())
        }
    }

    #[test]
    fn test_migration_runner() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(TestMigration));

        let runner = MigrationRunner::with_registry(registry);
        runner.migrate(&mut conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_rollback() {
        let mut conn = Connection::open_in_memory().unwrap();
        let mut registry = MigrationRegistry::new();
        registry.register(Box::new(TestMigration));

        let runner = MigrationRunner::with_registry(registry);
        runner.migrate(&mut conn).unwrap();
        runner.rollback(&mut conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 0);
    }
}
