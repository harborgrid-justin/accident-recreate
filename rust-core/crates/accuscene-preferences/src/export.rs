//! Preference export and import functionality

use crate::error::{PreferencesError, Result};
use crate::schema::types::PreferenceValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// TOML format
    Toml,
    /// Binary format (compressed)
    Binary,
}

/// Preference export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceExport {
    /// Export format version
    pub version: String,

    /// Application version
    pub app_version: String,

    /// Export timestamp
    pub exported_at: chrono::DateTime<chrono::Utc>,

    /// Device ID that created the export
    pub device_id: String,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Export format
    pub format: ExportFormat,

    /// Exported preferences
    pub preferences: HashMap<String, PreferenceValue>,

    /// Metadata about the export
    pub metadata: ExportMetadata,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Total number of preferences
    pub total_count: usize,

    /// Categories included
    pub categories: Vec<String>,

    /// Whether sensitive data is included
    pub includes_sensitive: bool,

    /// Export notes
    pub notes: Option<String>,
}

impl PreferenceExport {
    /// Create a new export
    pub fn new(
        app_version: String,
        device_id: String,
        user_id: Option<String>,
        preferences: HashMap<String, PreferenceValue>,
        format: ExportFormat,
    ) -> Self {
        let categories = Self::extract_categories(&preferences);
        let includes_sensitive = Self::check_sensitive(&preferences);

        Self {
            version: "1.0".to_string(),
            app_version,
            exported_at: chrono::Utc::now(),
            device_id,
            user_id,
            format,
            preferences: preferences.clone(),
            metadata: ExportMetadata {
                total_count: preferences.len(),
                categories,
                includes_sensitive,
                notes: None,
            },
        }
    }

    /// Extract categories from preferences
    fn extract_categories(preferences: &HashMap<String, PreferenceValue>) -> Vec<String> {
        let mut categories: Vec<String> = preferences
            .keys()
            .filter_map(|key| {
                let parts: Vec<&str> = key.split('.').collect();
                if parts.len() >= 2 {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }

    /// Check if export includes sensitive data
    fn check_sensitive(_preferences: &HashMap<String, PreferenceValue>) -> bool {
        // In production, this would check against schema
        false
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PreferencesError::ExportError(e.to_string()))
    }

    /// Convert to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| PreferencesError::ExportError(e.to_string()))
    }

    /// Convert to binary format
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| PreferencesError::ExportError(e.to_string()))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| PreferencesError::ImportError(e.to_string()))
    }

    /// Parse from TOML string
    pub fn from_toml(toml: &str) -> Result<Self> {
        toml::from_str(toml).map_err(|e| PreferencesError::ImportError(e.to_string()))
    }

    /// Parse from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| PreferencesError::ImportError(e.to_string()))
    }
}

/// Export all preferences
pub fn export_preferences(
    preferences: &HashMap<String, PreferenceValue>,
) -> Result<PreferenceExport> {
    let export = PreferenceExport::new(
        env!("CARGO_PKG_VERSION").to_string(),
        uuid::Uuid::new_v4().to_string(),
        None,
        preferences.clone(),
        ExportFormat::Json,
    );

    Ok(export)
}

/// Export preferences with options
pub fn export_preferences_with_options(
    preferences: &HashMap<String, PreferenceValue>,
    options: ExportOptions,
) -> Result<PreferenceExport> {
    let mut filtered_prefs = preferences.clone();

    // Filter by categories if specified
    if let Some(ref categories) = options.categories {
        filtered_prefs.retain(|key, _| {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() >= 2 {
                categories.contains(&parts[0].to_string())
            } else {
                false
            }
        });
    }

    // Exclude sensitive data if requested
    if !options.include_sensitive {
        // In production, this would check against schema
        // filtered_prefs.retain(|key, _| !is_sensitive(key));
    }

    let mut export = PreferenceExport::new(
        env!("CARGO_PKG_VERSION").to_string(),
        options.device_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        options.user_id,
        filtered_prefs,
        options.format,
    );

    export.metadata.notes = options.notes;

    Ok(export)
}

/// Import preferences from export
pub fn import_preferences(export: PreferenceExport) -> Result<HashMap<String, PreferenceValue>> {
    // Validate export version
    if export.version != "1.0" {
        return Err(PreferencesError::ImportError(format!(
            "Unsupported export version: {}",
            export.version
        )));
    }

    // In production, you might want to validate or migrate preferences here

    Ok(export.preferences)
}

/// Import preferences with options
pub fn import_preferences_with_options(
    export: PreferenceExport,
    options: ImportOptions,
) -> Result<HashMap<String, PreferenceValue>> {
    let mut preferences = export.preferences;

    // Filter by categories if specified
    if let Some(ref categories) = options.categories {
        preferences.retain(|key, _| {
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() >= 2 {
                categories.contains(&parts[0].to_string())
            } else {
                false
            }
        });
    }

    // Apply merge strategy
    match options.merge_strategy {
        MergeStrategy::Replace => {
            // Return all imported preferences
        }
        MergeStrategy::Merge => {
            // Would merge with existing preferences (handled by caller)
        }
        MergeStrategy::KeepExisting => {
            // Would only import missing preferences (handled by caller)
        }
    }

    Ok(preferences)
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format
    pub format: ExportFormat,

    /// Include only specific categories
    pub categories: Option<Vec<String>>,

    /// Include sensitive data
    pub include_sensitive: bool,

    /// Device ID
    pub device_id: Option<String>,

    /// User ID
    pub user_id: Option<String>,

    /// Export notes
    pub notes: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            categories: None,
            include_sensitive: false,
            device_id: None,
            user_id: None,
            notes: None,
        }
    }
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Import only specific categories
    pub categories: Option<Vec<String>>,

    /// Merge strategy
    pub merge_strategy: MergeStrategy,

    /// Validate before import
    pub validate: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            categories: None,
            merge_strategy: MergeStrategy::Merge,
            validate: true,
        }
    }
}

/// Merge strategy for imports
#[derive(Debug, Clone, PartialEq)]
pub enum MergeStrategy {
    /// Replace all existing preferences
    Replace,

    /// Merge with existing preferences (imported takes precedence)
    Merge,

    /// Keep existing preferences (only import missing)
    KeepExisting,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_creation() {
        let mut prefs = HashMap::new();
        prefs.insert(
            "appearance.theme".to_string(),
            PreferenceValue::String("dark".to_string()),
        );
        prefs.insert(
            "general.language".to_string(),
            PreferenceValue::String("en".to_string()),
        );

        let export = export_preferences(&prefs).unwrap();

        assert_eq!(export.preferences.len(), 2);
        assert_eq!(export.metadata.total_count, 2);
        assert!(export.metadata.categories.contains(&"appearance".to_string()));
        assert!(export.metadata.categories.contains(&"general".to_string()));
    }

    #[test]
    fn test_json_serialization() {
        let mut prefs = HashMap::new();
        prefs.insert(
            "test.key".to_string(),
            PreferenceValue::String("value".to_string()),
        );

        let export = export_preferences(&prefs).unwrap();
        let json = export.to_json().unwrap();

        let imported = PreferenceExport::from_json(&json).unwrap();
        assert_eq!(imported.preferences, export.preferences);
    }

    #[test]
    fn test_export_with_options() {
        let mut prefs = HashMap::new();
        prefs.insert(
            "appearance.theme".to_string(),
            PreferenceValue::String("dark".to_string()),
        );
        prefs.insert(
            "general.language".to_string(),
            PreferenceValue::String("en".to_string()),
        );

        let options = ExportOptions {
            categories: Some(vec!["appearance".to_string()]),
            ..Default::default()
        };

        let export = export_preferences_with_options(&prefs, options).unwrap();

        assert_eq!(export.preferences.len(), 1);
        assert!(export.preferences.contains_key("appearance.theme"));
    }

    #[test]
    fn test_import() {
        let mut prefs = HashMap::new();
        prefs.insert(
            "test.key".to_string(),
            PreferenceValue::String("value".to_string()),
        );

        let export = export_preferences(&prefs).unwrap();
        let imported = import_preferences(export).unwrap();

        assert_eq!(imported, prefs);
    }
}
