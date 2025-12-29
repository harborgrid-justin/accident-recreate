use crate::{error::Result, DataRecord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Field schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field name
    pub name: String,
    /// Detected data type
    pub data_type: DataType,
    /// Is this field nullable?
    pub nullable: bool,
    /// Sample values
    pub samples: Vec<String>,
    /// Unique value count
    pub unique_count: usize,
    /// Null/missing count
    pub null_count: usize,
    /// Min value (for numbers)
    pub min_value: Option<f64>,
    /// Max value (for numbers)
    pub max_value: Option<f64>,
    /// Min length (for strings)
    pub min_length: Option<usize>,
    /// Max length (for strings)
    pub max_length: Option<usize>,
    /// Pattern (for strings)
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
    Time,
    Array,
    Object,
    Unknown,
}

impl FieldSchema {
    /// Create new field schema
    pub fn new(name: String) -> Self {
        Self {
            name,
            data_type: DataType::Unknown,
            nullable: false,
            samples: Vec::new(),
            unique_count: 0,
            null_count: 0,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }
}

/// Schema detector
pub struct SchemaDetector {
    /// Maximum samples to collect per field
    max_samples: usize,
}

impl SchemaDetector {
    /// Create new schema detector
    pub fn new() -> Self {
        Self { max_samples: 10 }
    }

    /// Detect schema from records
    pub fn detect(&self, records: &[DataRecord]) -> Result<Vec<FieldSchema>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        // Collect all field names
        let mut field_names = std::collections::HashSet::new();
        for record in records {
            for name in record.field_names() {
                field_names.insert(name);
            }
        }

        // Analyze each field
        let mut schemas = Vec::new();
        for field_name in field_names {
            let schema = self.analyze_field(&field_name, records)?;
            schemas.push(schema);
        }

        schemas.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(schemas)
    }

    /// Analyze single field
    fn analyze_field(&self, field_name: &str, records: &[DataRecord]) -> Result<FieldSchema> {
        let mut schema = FieldSchema::new(field_name.to_string());
        let mut values: Vec<serde_json::Value> = Vec::new();
        let mut unique_values = std::collections::HashSet::new();
        let mut type_counts: HashMap<DataType, usize> = HashMap::new();

        for record in records {
            if let Some(value) = record.get(field_name) {
                if value.is_null() {
                    schema.null_count += 1;
                } else {
                    values.push(value.clone());
                    unique_values.insert(value.to_string());

                    // Detect type
                    let detected_type = detect_type(value);
                    *type_counts.entry(detected_type).or_insert(0) += 1;

                    // Collect samples
                    if schema.samples.len() < self.max_samples {
                        schema.samples.push(format_sample(value));
                    }

                    // Analyze numeric values
                    if let Some(num) = value.as_f64() {
                        schema.min_value = Some(schema.min_value.map_or(num, |min| min.min(num)));
                        schema.max_value = Some(schema.max_value.map_or(num, |max| max.max(num)));
                    }

                    // Analyze string values
                    if let Some(s) = value.as_str() {
                        let len = s.len();
                        schema.min_length = Some(schema.min_length.map_or(len, |min| min.min(len)));
                        schema.max_length = Some(schema.max_length.map_or(len, |max| max.max(len)));
                    }
                }
            } else {
                schema.null_count += 1;
            }
        }

        schema.unique_count = unique_values.len();
        schema.nullable = schema.null_count > 0;

        // Determine most common type
        schema.data_type = type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(dtype, _)| dtype)
            .unwrap_or(DataType::Unknown);

        Ok(schema)
    }

    /// Quick schema detection (first N records only)
    pub fn quick_detect(&self, records: &[DataRecord], sample_size: usize) -> Result<Vec<FieldSchema>> {
        let sample = &records[..sample_size.min(records.len())];
        self.detect(sample)
    }
}

impl Default for SchemaDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect data type from JSON value
fn detect_type(value: &serde_json::Value) -> DataType {
    match value {
        serde_json::Value::Null => DataType::Unknown,
        serde_json::Value::Bool(_) => DataType::Boolean,
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                DataType::Integer
            } else {
                DataType::Float
            }
        }
        serde_json::Value::String(s) => {
            // Try to detect special string types
            if is_date(s) {
                DataType::Date
            } else if is_datetime(s) {
                DataType::DateTime
            } else if is_time(s) {
                DataType::Time
            } else {
                DataType::String
            }
        }
        serde_json::Value::Array(_) => DataType::Array,
        serde_json::Value::Object(_) => DataType::Object,
    }
}

/// Check if string is a date
fn is_date(s: &str) -> bool {
    // Simple date patterns
    let date_patterns = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%m/%d/%Y",
        "%d/%m/%Y",
    ];

    for pattern in &date_patterns {
        if chrono::NaiveDate::parse_from_str(s, pattern).is_ok() {
            return true;
        }
    }

    false
}

/// Check if string is a datetime
fn is_datetime(s: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(s).is_ok()
        || chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").is_ok()
}

/// Check if string is a time
fn is_time(s: &str) -> bool {
    chrono::NaiveTime::parse_from_str(s, "%H:%M:%S").is_ok()
        || chrono::NaiveTime::parse_from_str(s, "%H:%M").is_ok()
}

/// Format value as sample string
fn format_sample(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..47])
            } else {
                s.clone()
            }
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_detect_type() {
        assert_eq!(detect_type(&Value::Bool(true)), DataType::Boolean);
        assert_eq!(detect_type(&Value::Number(42.into())), DataType::Integer);
        assert_eq!(detect_type(&Value::String("test".to_string())), DataType::String);
    }

    #[test]
    fn test_schema_detection() {
        let mut record1 = DataRecord::new();
        record1.set("name".to_string(), Value::String("John".to_string()));
        record1.set("age".to_string(), Value::Number(30.into()));

        let mut record2 = DataRecord::new();
        record2.set("name".to_string(), Value::String("Jane".to_string()));
        record2.set("age".to_string(), Value::Number(25.into()));

        let detector = SchemaDetector::new();
        let schema = detector.detect(&[record1, record2]).unwrap();

        assert_eq!(schema.len(), 2);

        let age_schema = schema.iter().find(|s| s.name == "age").unwrap();
        assert_eq!(age_schema.data_type, DataType::Integer);
        assert_eq!(age_schema.min_value, Some(25.0));
        assert_eq!(age_schema.max_value, Some(30.0));
    }

    #[test]
    fn test_nullable_detection() {
        let mut record1 = DataRecord::new();
        record1.set("name".to_string(), Value::String("John".to_string()));

        let mut record2 = DataRecord::new();
        record2.set("name".to_string(), Value::Null);

        let detector = SchemaDetector::new();
        let schema = detector.detect(&[record1, record2]).unwrap();

        let name_schema = schema.iter().find(|s| s.name == "name").unwrap();
        assert!(name_schema.nullable);
        assert_eq!(name_schema.null_count, 1);
    }
}
