//! Default preference values

use crate::schema::types::PreferenceValue;
use crate::schema::PreferenceSchemaRegistry;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global schema registry
static SCHEMA_REGISTRY: OnceLock<PreferenceSchemaRegistry> = OnceLock::new();

/// Get the global schema registry
fn get_registry() -> &'static PreferenceSchemaRegistry {
    SCHEMA_REGISTRY.get_or_init(|| PreferenceSchemaRegistry::new())
}

/// Get default value for a preference
pub fn get_default(key: &str) -> Option<PreferenceValue> {
    let registry = get_registry();
    registry.get(key).and_then(|schema| schema.default_value.clone())
}

/// Get all default preferences
pub fn get_all_defaults() -> HashMap<String, PreferenceValue> {
    let registry = get_registry();
    let mut defaults = HashMap::new();

    for category in registry.get_categories() {
        for schema in registry.get_category(&category) {
            if let Some(default) = &schema.default_value {
                defaults.insert(schema.key.clone(), default.clone());
            }
        }
    }

    defaults
}

/// Get defaults for a specific category
pub fn get_category_defaults(category: &str) -> HashMap<String, PreferenceValue> {
    let registry = get_registry();
    let mut defaults = HashMap::new();

    for schema in registry.get_category(category) {
        if let Some(default) = &schema.default_value {
            defaults.insert(schema.key.clone(), default.clone());
        }
    }

    defaults
}

/// Check if a value is the default for a preference
pub fn is_default(key: &str, value: &PreferenceValue) -> bool {
    if let Some(default) = get_default(key) {
        &default == value
    } else {
        false
    }
}

/// Reset all preferences to defaults (returns the default map)
pub fn reset_to_defaults() -> HashMap<String, PreferenceValue> {
    get_all_defaults()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default() {
        let default = get_default("appearance.theme");
        assert!(default.is_some());
        assert_eq!(
            default.unwrap(),
            PreferenceValue::String("light".to_string())
        );
    }

    #[test]
    fn test_get_all_defaults() {
        let defaults = get_all_defaults();
        assert!(!defaults.is_empty());
        assert!(defaults.contains_key("appearance.theme"));
        assert!(defaults.contains_key("general.language"));
    }

    #[test]
    fn test_get_category_defaults() {
        let defaults = get_category_defaults("appearance");
        assert!(!defaults.is_empty());
        assert!(defaults.contains_key("appearance.theme"));
        assert!(!defaults.contains_key("general.language"));
    }

    #[test]
    fn test_is_default() {
        assert!(is_default(
            "appearance.theme",
            &PreferenceValue::String("light".to_string())
        ));
        assert!(!is_default(
            "appearance.theme",
            &PreferenceValue::String("dark".to_string())
        ));
    }
}
