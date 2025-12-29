//! Preference migration for version updates

use crate::error::{PreferencesError, Result};
use crate::schema::types::PreferenceValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Migration version
pub type MigrationVersion = String;

/// Preference migration
pub trait PreferenceMigration: Send + Sync {
    /// Get the source version for this migration
    fn from_version(&self) -> MigrationVersion;

    /// Get the target version for this migration
    fn to_version(&self) -> MigrationVersion;

    /// Apply the migration to preferences
    fn migrate(&self, preferences: &mut HashMap<String, PreferenceValue>) -> Result<()>;

    /// Get migration description
    fn description(&self) -> String;
}

/// Migration manager
pub struct MigrationManager {
    migrations: Vec<Box<dyn PreferenceMigration>>,
}

impl MigrationManager {
    /// Create a new migration manager with default migrations
    pub fn new() -> Self {
        let mut manager = Self {
            migrations: Vec::new(),
        };

        // Register default migrations
        manager.register_default_migrations();

        manager
    }

    /// Register a migration
    pub fn register(&mut self, migration: Box<dyn PreferenceMigration>) {
        self.migrations.push(migration);
    }

    /// Register default migrations
    fn register_default_migrations(&mut self) {
        // Example: Migration from 0.1.0 to 0.2.0
        self.register(Box::new(Migration_0_1_to_0_2));

        // Example: Migration from 0.2.0 to 0.2.5
        self.register(Box::new(Migration_0_2_to_0_2_5));
    }

    /// Migrate preferences from one version to another
    pub fn migrate(
        &self,
        from_version: &str,
        to_version: &str,
        preferences: &mut HashMap<String, PreferenceValue>,
    ) -> Result<()> {
        let mut current_version = from_version.to_string();

        while current_version != to_version {
            // Find migration for current version
            let migration = self
                .migrations
                .iter()
                .find(|m| m.from_version() == current_version)
                .ok_or_else(|| {
                    PreferencesError::MigrationError(format!(
                        "No migration found from version {}",
                        current_version
                    ))
                })?;

            tracing::info!(
                "Applying migration: {} -> {} ({})",
                migration.from_version(),
                migration.to_version(),
                migration.description()
            );

            // Apply migration
            migration.migrate(preferences)?;

            // Update current version
            current_version = migration.to_version();

            // Safety check to prevent infinite loops
            if current_version == from_version {
                return Err(PreferencesError::MigrationError(
                    "Migration loop detected".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get all migrations for a version range
    pub fn get_migrations(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Vec<&dyn PreferenceMigration> {
        let mut migrations = Vec::new();
        let mut current_version = from_version.to_string();

        while current_version != to_version {
            if let Some(migration) = self
                .migrations
                .iter()
                .find(|m| m.from_version() == current_version)
            {
                migrations.push(migration.as_ref());
                current_version = migration.to_version();
            } else {
                break;
            }
        }

        migrations
    }

    /// Check if migration is needed
    pub fn needs_migration(&self, from_version: &str, to_version: &str) -> bool {
        from_version != to_version
            && self
                .migrations
                .iter()
                .any(|m| m.from_version() == from_version)
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

// Example migrations

/// Migration from 0.1.0 to 0.2.0
struct Migration_0_1_to_0_2;

impl PreferenceMigration for Migration_0_1_to_0_2 {
    fn from_version(&self) -> MigrationVersion {
        "0.1.0".to_string()
    }

    fn to_version(&self) -> MigrationVersion {
        "0.2.0".to_string()
    }

    fn migrate(&self, preferences: &mut HashMap<String, PreferenceValue>) -> Result<()> {
        // Example: Rename old key to new key
        if let Some(value) = preferences.remove("old.theme") {
            preferences.insert("appearance.theme".to_string(), value);
        }

        // Example: Convert old boolean to new string enum
        if let Some(PreferenceValue::Boolean(true)) = preferences.get("notifications.enabled") {
            preferences.insert(
                "notifications.mode".to_string(),
                PreferenceValue::String("all".to_string()),
            );
        }

        Ok(())
    }

    fn description(&self) -> String {
        "Migrate theme preferences to new structure".to_string()
    }
}

/// Migration from 0.2.0 to 0.2.5
struct Migration_0_2_to_0_2_5;

impl PreferenceMigration for Migration_0_2_to_0_2_5 {
    fn from_version(&self) -> MigrationVersion {
        "0.2.0".to_string()
    }

    fn to_version(&self) -> MigrationVersion {
        "0.2.5".to_string()
    }

    fn migrate(&self, preferences: &mut HashMap<String, PreferenceValue>) -> Result<()> {
        // Example: Add new default preferences
        if !preferences.contains_key("accessibility.high_contrast") {
            preferences.insert(
                "accessibility.high_contrast".to_string(),
                PreferenceValue::Boolean(false),
            );
        }

        if !preferences.contains_key("accessibility.reduce_motion") {
            preferences.insert(
                "accessibility.reduce_motion".to_string(),
                PreferenceValue::Boolean(false),
            );
        }

        // Example: Convert old integer font size to new scale
        if let Some(PreferenceValue::Integer(size)) = preferences.get("appearance.font_size") {
            // Convert old pixel size to new scale (12-16 -> 0.8-1.2)
            let scale = (*size as f64 - 12.0) / 4.0 + 0.8;
            preferences.insert(
                "appearance.font_scale".to_string(),
                PreferenceValue::Float(scale),
            );
        }

        Ok(())
    }

    fn description(&self) -> String {
        "Add accessibility preferences and update font scaling".to_string()
    }
}

/// Migration metadata stored with preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    pub original_version: String,
    pub current_version: String,
    pub migration_history: Vec<MigrationRecord>,
    pub last_migration: Option<chrono::DateTime<chrono::Utc>>,
}

/// Record of a single migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub from_version: String,
    pub to_version: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

impl MigrationMetadata {
    /// Create new metadata
    pub fn new(version: String) -> Self {
        Self {
            original_version: version.clone(),
            current_version: version,
            migration_history: Vec::new(),
            last_migration: None,
        }
    }

    /// Record a migration
    pub fn record_migration(&mut self, migration: &dyn PreferenceMigration) {
        self.migration_history.push(MigrationRecord {
            from_version: migration.from_version(),
            to_version: migration.to_version(),
            applied_at: chrono::Utc::now(),
            description: migration.description(),
        });
        self.current_version = migration.to_version();
        self.last_migration = Some(chrono::Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_manager() {
        let manager = MigrationManager::new();
        assert!(manager.needs_migration("0.1.0", "0.2.0"));
        assert!(!manager.needs_migration("0.2.5", "0.2.5"));
    }

    #[test]
    fn test_migration_0_1_to_0_2() {
        let mut prefs = HashMap::new();
        prefs.insert(
            "old.theme".to_string(),
            PreferenceValue::String("dark".to_string()),
        );

        let migration = Migration_0_1_to_0_2;
        migration.migrate(&mut prefs).unwrap();

        assert!(!prefs.contains_key("old.theme"));
        assert_eq!(
            prefs.get("appearance.theme"),
            Some(&PreferenceValue::String("dark".to_string()))
        );
    }

    #[test]
    fn test_migration_chain() {
        let manager = MigrationManager::new();
        let mut prefs = HashMap::new();
        prefs.insert(
            "old.theme".to_string(),
            PreferenceValue::String("dark".to_string()),
        );

        manager.migrate("0.1.0", "0.2.5", &mut prefs).unwrap();

        // Check that all migrations were applied
        assert!(prefs.contains_key("appearance.theme"));
        assert!(prefs.contains_key("accessibility.high_contrast"));
    }

    #[test]
    fn test_migration_metadata() {
        let mut metadata = MigrationMetadata::new("0.1.0".to_string());
        assert_eq!(metadata.current_version, "0.1.0");

        let migration = Migration_0_1_to_0_2;
        metadata.record_migration(&migration);

        assert_eq!(metadata.current_version, "0.2.0");
        assert_eq!(metadata.migration_history.len(), 1);
    }
}
