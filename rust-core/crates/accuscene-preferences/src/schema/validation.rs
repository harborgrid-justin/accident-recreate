//! Preference validation logic

use crate::error::{PreferencesError, Result};
use crate::schema::types::{PreferenceSchema, PreferenceType, PreferenceValue};
use crate::schema::PreferenceSchemaRegistry;
use std::sync::OnceLock;

/// Global schema registry
static SCHEMA_REGISTRY: OnceLock<PreferenceSchemaRegistry> = OnceLock::new();

/// Get the global schema registry
fn get_registry() -> &'static PreferenceSchemaRegistry {
    SCHEMA_REGISTRY.get_or_init(|| PreferenceSchemaRegistry::new())
}

/// Validate a preference value against its schema
pub fn validate_preference(key: &str, value: &PreferenceValue) -> Result<()> {
    let registry = get_registry();

    // Get schema for key
    let schema = registry.get(key).ok_or_else(|| {
        PreferencesError::ValidationError {
            key: key.to_string(),
            message: "No schema found for key".to_string(),
        }
    })?;

    // Validate against schema
    schema.validate(value).map_err(|msg| PreferencesError::ValidationError {
        key: key.to_string(),
        message: msg,
    })?;

    // Additional validation rules
    validate_custom_rules(key, value, schema)?;

    Ok(())
}

/// Custom validation rules for specific preferences
fn validate_custom_rules(
    key: &str,
    value: &PreferenceValue,
    schema: &PreferenceSchema,
) -> Result<()> {
    match key {
        // Font size validation
        "appearance.font_size" => {
            if let Some(size) = value.as_integer() {
                if size < 8 || size > 32 {
                    return Err(PreferencesError::ValidationError {
                        key: key.to_string(),
                        message: "Font size must be between 8 and 32 pixels".to_string(),
                    });
                }
            }
        }

        // Timezone validation
        "general.timezone" => {
            if let Some(tz) = value.as_string() {
                // Basic timezone validation
                if !is_valid_timezone(tz) {
                    return Err(PreferencesError::ValidationError {
                        key: key.to_string(),
                        message: format!("Invalid timezone: {}", tz),
                    });
                }
            }
        }

        // Language validation
        "general.language" => {
            if let Some(lang) = value.as_string() {
                if !is_valid_language_code(lang) {
                    return Err(PreferencesError::ValidationError {
                        key: key.to_string(),
                        message: format!("Invalid language code: {}", lang),
                    });
                }
            }
        }

        _ => {}
    }

    Ok(())
}

/// Validate timezone string
fn is_valid_timezone(tz: &str) -> bool {
    // Common timezone patterns
    const VALID_TZ: &[&str] = &[
        "UTC", "GMT", "EST", "CST", "MST", "PST",
        "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles",
        "Europe/London", "Europe/Paris", "Europe/Berlin",
        "Asia/Tokyo", "Asia/Shanghai", "Asia/Dubai",
        "Australia/Sydney", "Pacific/Auckland",
    ];

    // Check if it's in the list or follows standard pattern
    VALID_TZ.contains(&tz) || tz.contains('/') && tz.len() > 3
}

/// Validate language code (ISO 639-1)
fn is_valid_language_code(lang: &str) -> bool {
    const VALID_LANGS: &[&str] = &[
        "en", "es", "fr", "de", "it", "pt", "ru", "ja", "zh", "ko", "ar",
    ];

    VALID_LANGS.contains(&lang) || lang.len() == 2
}

/// Validate multiple preferences at once
pub fn validate_preferences(
    preferences: &std::collections::HashMap<String, PreferenceValue>,
) -> Result<()> {
    for (key, value) in preferences {
        validate_preference(key, value)?;
    }
    Ok(())
}

/// Check if a preference requires restart
pub fn requires_restart(key: &str) -> bool {
    let registry = get_registry();
    registry
        .get(key)
        .map(|schema| schema.requires_restart)
        .unwrap_or(false)
}

/// Check if a preference is sensitive
pub fn is_sensitive(key: &str) -> bool {
    let registry = get_registry();
    registry
        .get(key)
        .map(|schema| schema.sensitive)
        .unwrap_or(false)
}

/// Get the category for a preference
pub fn get_category(key: &str) -> Option<String> {
    let registry = get_registry();
    registry.get(key).map(|schema| schema.category.clone())
}

/// Get all preferences in a category
pub fn get_preferences_in_category(category: &str) -> Vec<String> {
    let registry = get_registry();
    registry
        .get_category(category)
        .iter()
        .map(|schema| schema.key.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_validate_preference() {
        let result = validate_preference(
            "appearance.theme",
            &PreferenceValue::String("dark".to_string()),
        );
        assert!(result.is_ok());

        let result = validate_preference(
            "appearance.theme",
            &PreferenceValue::String("invalid".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_font_size_validation() {
        let result = validate_preference(
            "appearance.font_size",
            &PreferenceValue::Integer(14),
        );
        assert!(result.is_ok());

        let result = validate_preference(
            "appearance.font_size",
            &PreferenceValue::Integer(5),
        );
        assert!(result.is_err());

        let result = validate_preference(
            "appearance.font_size",
            &PreferenceValue::Integer(50),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_timezone_validation() {
        assert!(is_valid_timezone("UTC"));
        assert!(is_valid_timezone("America/New_York"));
        assert!(!is_valid_timezone("XX"));
    }

    #[test]
    fn test_language_validation() {
        assert!(is_valid_language_code("en"));
        assert!(is_valid_language_code("es"));
        assert!(!is_valid_language_code("xxx"));
    }

    #[test]
    fn test_requires_restart() {
        assert!(!requires_restart("appearance.theme"));
        assert!(requires_restart("accessibility.screen_reader"));
    }

    #[test]
    fn test_get_category() {
        assert_eq!(get_category("appearance.theme"), Some("appearance".to_string()));
        assert_eq!(get_category("nonexistent"), None);
    }
}
