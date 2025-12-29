//! Preference value types and schema definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Preference value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum PreferenceValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<PreferenceValue>),
    /// Object/map of values
    Object(HashMap<String, PreferenceValue>),
    /// Null value
    Null,
}

impl PreferenceValue {
    /// Get the type of this value
    pub fn get_type(&self) -> PreferenceType {
        match self {
            PreferenceValue::String(_) => PreferenceType::String,
            PreferenceValue::Integer(_) => PreferenceType::Integer,
            PreferenceValue::Float(_) => PreferenceType::Float,
            PreferenceValue::Boolean(_) => PreferenceType::Boolean,
            PreferenceValue::Array(_) => PreferenceType::Array,
            PreferenceValue::Object(_) => PreferenceType::Object,
            PreferenceValue::Null => PreferenceType::Null,
        }
    }

    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PreferenceValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PreferenceValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to float if possible
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PreferenceValue::Float(f) => Some(*f),
            PreferenceValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Convert to boolean if possible
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            PreferenceValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Convert to array if possible
    pub fn as_array(&self) -> Option<&Vec<PreferenceValue>> {
        match self {
            PreferenceValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Convert to object if possible
    pub fn as_object(&self) -> Option<&HashMap<String, PreferenceValue>> {
        match self {
            PreferenceValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, PreferenceValue::Null)
    }
}

/// Preference type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PreferenceType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Null,
}

impl std::fmt::Display for PreferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreferenceType::String => write!(f, "string"),
            PreferenceType::Integer => write!(f, "integer"),
            PreferenceType::Float => write!(f, "float"),
            PreferenceType::Boolean => write!(f, "boolean"),
            PreferenceType::Array => write!(f, "array"),
            PreferenceType::Object => write!(f, "object"),
            PreferenceType::Null => write!(f, "null"),
        }
    }
}

/// Schema for a preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceSchema {
    /// Unique key for the preference
    pub key: String,

    /// Type of the preference value
    pub pref_type: PreferenceType,

    /// Default value
    pub default_value: Option<PreferenceValue>,

    /// Allowed values (for enums)
    pub allowed_values: Option<Vec<PreferenceValue>>,

    /// Description of the preference
    pub description: String,

    /// Category for grouping
    pub category: String,

    /// Whether changing this preference requires app restart
    pub requires_restart: bool,

    /// Whether this preference contains sensitive data
    pub sensitive: bool,
}

impl PreferenceSchema {
    /// Create a new schema builder
    pub fn builder(key: String) -> PreferenceSchemaBuilder {
        PreferenceSchemaBuilder {
            key,
            pref_type: PreferenceType::String,
            default_value: None,
            allowed_values: None,
            description: String::new(),
            category: "general".to_string(),
            requires_restart: false,
            sensitive: false,
        }
    }

    /// Validate a value against this schema
    pub fn validate(&self, value: &PreferenceValue) -> Result<(), String> {
        // Check type
        if value.get_type() != self.pref_type && !value.is_null() {
            return Err(format!(
                "Type mismatch: expected {}, got {}",
                self.pref_type,
                value.get_type()
            ));
        }

        // Check allowed values
        if let Some(ref allowed) = self.allowed_values {
            if !allowed.contains(value) && !value.is_null() {
                return Err(format!(
                    "Value not in allowed list: {:?}",
                    value
                ));
            }
        }

        Ok(())
    }
}

/// Builder for preference schemas
pub struct PreferenceSchemaBuilder {
    key: String,
    pref_type: PreferenceType,
    default_value: Option<PreferenceValue>,
    allowed_values: Option<Vec<PreferenceValue>>,
    description: String,
    category: String,
    requires_restart: bool,
    sensitive: bool,
}

impl PreferenceSchemaBuilder {
    pub fn pref_type(mut self, pref_type: PreferenceType) -> Self {
        self.pref_type = pref_type;
        self
    }

    pub fn default_value(mut self, value: PreferenceValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn allowed_values(mut self, values: Vec<PreferenceValue>) -> Self {
        self.allowed_values = Some(values);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn category(mut self, category: String) -> Self {
        self.category = category;
        self
    }

    pub fn requires_restart(mut self, requires_restart: bool) -> Self {
        self.requires_restart = requires_restart;
        self
    }

    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    pub fn build(self) -> PreferenceSchema {
        PreferenceSchema {
            key: self.key,
            pref_type: self.pref_type,
            default_value: self.default_value,
            allowed_values: self.allowed_values,
            description: self.description,
            category: self.category,
            requires_restart: self.requires_restart,
            sensitive: self.sensitive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_value_types() {
        let string_val = PreferenceValue::String("test".to_string());
        assert_eq!(string_val.get_type(), PreferenceType::String);
        assert_eq!(string_val.as_string(), Some("test"));

        let int_val = PreferenceValue::Integer(42);
        assert_eq!(int_val.get_type(), PreferenceType::Integer);
        assert_eq!(int_val.as_integer(), Some(42));

        let bool_val = PreferenceValue::Boolean(true);
        assert_eq!(bool_val.get_type(), PreferenceType::Boolean);
        assert_eq!(bool_val.as_boolean(), Some(true));
    }

    #[test]
    fn test_schema_validation() {
        let schema = PreferenceSchema::builder("test.key".to_string())
            .pref_type(PreferenceType::String)
            .allowed_values(vec![
                PreferenceValue::String("a".to_string()),
                PreferenceValue::String("b".to_string()),
            ])
            .build();

        assert!(schema.validate(&PreferenceValue::String("a".to_string())).is_ok());
        assert!(schema.validate(&PreferenceValue::String("c".to_string())).is_err());
        assert!(schema.validate(&PreferenceValue::Integer(1)).is_err());
    }

    #[test]
    fn test_schema_builder() {
        let schema = PreferenceSchema::builder("test.key".to_string())
            .pref_type(PreferenceType::Boolean)
            .default_value(PreferenceValue::Boolean(true))
            .description("Test preference".to_string())
            .category("test".to_string())
            .requires_restart(true)
            .build();

        assert_eq!(schema.key, "test.key");
        assert_eq!(schema.pref_type, PreferenceType::Boolean);
        assert_eq!(schema.category, "test");
        assert!(schema.requires_restart);
    }
}
