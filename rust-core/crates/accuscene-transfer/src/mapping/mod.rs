/// Field mapping and transformation
pub mod schema;
pub mod transform;

use crate::{error::Result, DataRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use schema::{FieldSchema, SchemaDetector};
pub use transform::{Transform, TransformPipeline};

/// Field mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field name
    pub source: String,
    /// Target field name
    pub target: String,
    /// Optional transformation
    pub transform: Option<Transform>,
    /// Default value if source is missing
    pub default: Option<serde_json::Value>,
    /// Is this field required?
    pub required: bool,
}

impl FieldMapping {
    /// Create new field mapping
    pub fn new(source: String, target: String) -> Self {
        Self {
            source,
            target,
            transform: None,
            default: None,
            required: false,
        }
    }

    /// Set transformation
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Mapping profile for data transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProfile {
    /// Profile name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Field mappings
    pub mappings: Vec<FieldMapping>,
    /// Global transformations
    pub global_transforms: Vec<Transform>,
}

impl MappingProfile {
    /// Create new profile
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            mappings: Vec::new(),
            global_transforms: Vec::new(),
        }
    }

    /// Add field mapping
    pub fn add_mapping(mut self, mapping: FieldMapping) -> Self {
        self.mappings.push(mapping);
        self
    }

    /// Add global transform
    pub fn add_global_transform(mut self, transform: Transform) -> Self {
        self.global_transforms.push(transform);
        self
    }

    /// Apply profile to records
    pub fn apply(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>> {
        let mut transformed = Vec::new();

        for record in records {
            let new_record = self.apply_to_record(record)?;
            transformed.push(new_record);
        }

        Ok(transformed)
    }

    /// Apply profile to single record
    pub fn apply_to_record(&self, record: DataRecord) -> Result<DataRecord> {
        let mut new_record = DataRecord::new();

        // Apply field mappings
        for mapping in &self.mappings {
            let value = record.get(&mapping.source)
                .cloned()
                .or_else(|| mapping.default.clone());

            if let Some(mut val) = value {
                // Apply field-specific transform
                if let Some(ref transform) = mapping.transform {
                    val = transform.apply(val)?;
                }

                new_record.set(mapping.target.clone(), val);
            } else if mapping.required {
                return Err(crate::error::TransferError::FieldMapping(
                    format!("Required field '{}' not found", mapping.source),
                ));
            }
        }

        // Apply global transforms
        for transform in &self.global_transforms {
            new_record = transform.apply_to_record(new_record)?;
        }

        Ok(new_record)
    }
}

/// Auto-generate mapping profile from two schemas
pub fn auto_map(
    source_fields: Vec<String>,
    target_fields: Vec<String>,
) -> MappingProfile {
    let mut profile = MappingProfile::new("auto_generated".to_string());

    // Create case-insensitive lookup
    let target_map: HashMap<String, String> = target_fields
        .iter()
        .map(|f| (f.to_lowercase(), f.clone()))
        .collect();

    for source_field in source_fields {
        let source_lower = source_field.to_lowercase();

        // Try exact match (case-insensitive)
        if let Some(target_field) = target_map.get(&source_lower) {
            profile = profile.add_mapping(
                FieldMapping::new(source_field.clone(), target_field.clone())
            );
            continue;
        }

        // Try fuzzy match
        let mut best_match = None;
        let mut best_score = 0.0;

        for target_field in &target_fields {
            let score = similarity(&source_lower, &target_field.to_lowercase());
            if score > best_score && score > 0.6 {
                best_score = score;
                best_match = Some(target_field.clone());
            }
        }

        if let Some(target_field) = best_match {
            profile = profile.add_mapping(
                FieldMapping::new(source_field.clone(), target_field)
            );
        }
    }

    profile
}

/// Calculate string similarity (simple implementation)
fn similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let max_len = a_chars.len().max(b_chars.len());
    if max_len == 0 {
        return 1.0;
    }

    let mut matches = 0;
    let len = a_chars.len().min(b_chars.len());

    for i in 0..len {
        if a_chars[i] == b_chars[i] {
            matches += 1;
        }
    }

    matches as f64 / max_len as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_field_mapping() {
        let mapping = FieldMapping::new("old_name".to_string(), "new_name".to_string());

        let mut record = DataRecord::new();
        record.set("old_name".to_string(), Value::String("test".to_string()));

        let profile = MappingProfile::new("test".to_string())
            .add_mapping(mapping);

        let result = profile.apply_to_record(record).unwrap();
        assert_eq!(result.get("new_name"), Some(&Value::String("test".to_string())));
    }

    #[test]
    fn test_auto_map() {
        let source = vec!["firstName".to_string(), "lastName".to_string()];
        let target = vec!["first_name".to_string(), "last_name".to_string()];

        let profile = auto_map(source, target);
        assert!(!profile.mappings.is_empty());
    }

    #[test]
    fn test_similarity() {
        assert_eq!(similarity("test", "test"), 1.0);
        assert!(similarity("firstName", "first_name") > 0.5);
        assert!(similarity("abc", "xyz") < 0.5);
    }
}
