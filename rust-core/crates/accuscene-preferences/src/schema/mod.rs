//! Preference schema and validation

pub mod defaults;
pub mod types;
pub mod validation;

pub use types::{PreferenceSchema, PreferenceType, PreferenceValue};
pub use validation::validate_preference;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema definition for all preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceSchemaRegistry {
    /// All registered schemas
    schemas: HashMap<String, PreferenceSchema>,
}

impl PreferenceSchemaRegistry {
    /// Create a new schema registry with default schemas
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: HashMap::new(),
        };

        // Register default schemas
        registry.register_default_schemas();

        registry
    }

    /// Register a schema
    pub fn register(&mut self, key: String, schema: PreferenceSchema) {
        self.schemas.insert(key, schema);
    }

    /// Get a schema by key
    pub fn get(&self, key: &str) -> Option<&PreferenceSchema> {
        self.schemas.get(key)
    }

    /// Register all default schemas
    fn register_default_schemas(&mut self) {
        // General settings
        self.register(
            "general.language".to_string(),
            PreferenceSchema {
                key: "general.language".to_string(),
                pref_type: PreferenceType::String,
                default_value: Some(PreferenceValue::String("en".to_string())),
                allowed_values: Some(vec![
                    PreferenceValue::String("en".to_string()),
                    PreferenceValue::String("es".to_string()),
                    PreferenceValue::String("fr".to_string()),
                    PreferenceValue::String("de".to_string()),
                    PreferenceValue::String("ja".to_string()),
                ]),
                description: "Application language".to_string(),
                category: "general".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "general.timezone".to_string(),
            PreferenceSchema {
                key: "general.timezone".to_string(),
                pref_type: PreferenceType::String,
                default_value: Some(PreferenceValue::String("UTC".to_string())),
                allowed_values: None,
                description: "User timezone".to_string(),
                category: "general".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        // Appearance settings
        self.register(
            "appearance.theme".to_string(),
            PreferenceSchema {
                key: "appearance.theme".to_string(),
                pref_type: PreferenceType::String,
                default_value: Some(PreferenceValue::String("light".to_string())),
                allowed_values: Some(vec![
                    PreferenceValue::String("light".to_string()),
                    PreferenceValue::String("dark".to_string()),
                    PreferenceValue::String("auto".to_string()),
                ]),
                description: "Application theme".to_string(),
                category: "appearance".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "appearance.font_size".to_string(),
            PreferenceSchema {
                key: "appearance.font_size".to_string(),
                pref_type: PreferenceType::Integer,
                default_value: Some(PreferenceValue::Integer(14)),
                allowed_values: None,
                description: "Base font size in pixels".to_string(),
                category: "appearance".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "appearance.compact_mode".to_string(),
            PreferenceSchema {
                key: "appearance.compact_mode".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(false)),
                allowed_values: None,
                description: "Enable compact UI mode".to_string(),
                category: "appearance".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        // Notification settings
        self.register(
            "notifications.enabled".to_string(),
            PreferenceSchema {
                key: "notifications.enabled".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(true)),
                allowed_values: None,
                description: "Enable notifications".to_string(),
                category: "notifications".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "notifications.sound".to_string(),
            PreferenceSchema {
                key: "notifications.sound".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(true)),
                allowed_values: None,
                description: "Play sound for notifications".to_string(),
                category: "notifications".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        // Privacy settings
        self.register(
            "privacy.analytics".to_string(),
            PreferenceSchema {
                key: "privacy.analytics".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(false)),
                allowed_values: None,
                description: "Enable analytics tracking".to_string(),
                category: "privacy".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "privacy.crash_reports".to_string(),
            PreferenceSchema {
                key: "privacy.crash_reports".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(true)),
                allowed_values: None,
                description: "Send crash reports".to_string(),
                category: "privacy".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        // Accessibility settings
        self.register(
            "accessibility.high_contrast".to_string(),
            PreferenceSchema {
                key: "accessibility.high_contrast".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(false)),
                allowed_values: None,
                description: "Enable high contrast mode".to_string(),
                category: "accessibility".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "accessibility.reduce_motion".to_string(),
            PreferenceSchema {
                key: "accessibility.reduce_motion".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(false)),
                allowed_values: None,
                description: "Reduce animations and motion".to_string(),
                category: "accessibility".to_string(),
                requires_restart: false,
                sensitive: false,
            },
        );

        self.register(
            "accessibility.screen_reader".to_string(),
            PreferenceSchema {
                key: "accessibility.screen_reader".to_string(),
                pref_type: PreferenceType::Boolean,
                default_value: Some(PreferenceValue::Boolean(false)),
                allowed_values: None,
                description: "Enable screen reader support".to_string(),
                category: "accessibility".to_string(),
                requires_restart: true,
                sensitive: false,
            },
        );
    }

    /// Get all schemas in a category
    pub fn get_category(&self, category: &str) -> Vec<&PreferenceSchema> {
        self.schemas
            .values()
            .filter(|schema| schema.category == category)
            .collect()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .schemas
            .values()
            .map(|schema| schema.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

impl Default for PreferenceSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_registry() {
        let registry = PreferenceSchemaRegistry::new();
        assert!(registry.get("appearance.theme").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_get_category() {
        let registry = PreferenceSchemaRegistry::new();
        let appearance = registry.get_category("appearance");
        assert!(!appearance.is_empty());
    }

    #[test]
    fn test_get_categories() {
        let registry = PreferenceSchemaRegistry::new();
        let categories = registry.get_categories();
        assert!(categories.contains(&"appearance".to_string()));
        assert!(categories.contains(&"general".to_string()));
    }
}
