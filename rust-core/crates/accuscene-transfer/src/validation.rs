use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    DataRecord,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field name
    pub field: String,
    /// Rule type
    pub rule_type: RuleType,
    /// Error message
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleType {
    Required,
    MinLength { min: usize },
    MaxLength { max: usize },
    Pattern { regex: String },
    MinValue { min: f64 },
    MaxValue { max: f64 },
    Email,
    Url,
    Date { format: String },
    Enum { values: Vec<String> },
    Custom { validator: String },
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Warnings
    pub warnings: Vec<ValidationWarning>,
    /// Total records validated
    pub total_records: usize,
    /// Valid records
    pub valid_records: usize,
    /// Invalid records
    pub invalid_records: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            total_records: 0,
            valid_records: 0,
            invalid_records: 0,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Record index (if applicable)
    pub record_index: Option<usize>,
    /// Field name (if applicable)
    pub field: Option<String>,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Record index (if applicable)
    pub record_index: Option<usize>,
    /// Field name (if applicable)
    pub field: Option<String>,
    /// Warning message
    pub message: String,
}

/// Data validator
pub struct DataValidator {
    rules: Vec<ValidationRule>,
    config: TransferConfig,
}

impl DataValidator {
    /// Create new validator
    pub fn new(rules: Vec<ValidationRule>, config: TransferConfig) -> Self {
        Self { rules, config }
    }

    /// Validate records
    pub fn validate(&self, records: &[DataRecord]) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.total_records = records.len();

        for (idx, record) in records.iter().enumerate() {
            match self.validate_record(record, idx) {
                Ok(record_valid) => {
                    if record_valid {
                        result.valid_records += 1;
                    } else {
                        result.invalid_records += 1;
                    }
                }
                Err(e) => {
                    result.add_error(ValidationError {
                        record_index: Some(idx),
                        field: None,
                        message: e.to_string(),
                        code: "VALIDATION_ERROR".to_string(),
                    });
                    result.invalid_records += 1;

                    if !self.config.continue_on_error {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Validate single record
    fn validate_record(&self, record: &DataRecord, record_idx: usize) -> Result<bool> {
        let mut is_valid = true;

        for rule in &self.rules {
            if let Err(e) = self.validate_field(record, &rule.field, &rule.rule_type, record_idx) {
                is_valid = false;
                if !self.config.continue_on_error {
                    return Err(e);
                }
            }
        }

        Ok(is_valid)
    }

    /// Validate field
    fn validate_field(
        &self,
        record: &DataRecord,
        field: &str,
        rule_type: &RuleType,
        record_idx: usize,
    ) -> Result<()> {
        let value = record.get(field);

        match rule_type {
            RuleType::Required => {
                if value.is_none() || value == Some(&serde_json::Value::Null) {
                    return Err(TransferError::Validation(format!(
                        "Record {}: Field '{}' is required",
                        record_idx, field
                    )));
                }
            }
            RuleType::MinLength { min } => {
                if let Some(serde_json::Value::String(s)) = value {
                    if s.len() < *min {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' must be at least {} characters",
                            record_idx, field, min
                        )));
                    }
                }
            }
            RuleType::MaxLength { max } => {
                if let Some(serde_json::Value::String(s)) = value {
                    if s.len() > *max {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' must be at most {} characters",
                            record_idx, field, max
                        )));
                    }
                }
            }
            RuleType::Pattern { regex } => {
                if let Some(serde_json::Value::String(s)) = value {
                    let re = regex::Regex::new(regex)
                        .map_err(|e| TransferError::Validation(format!("Invalid regex: {}", e)))?;
                    if !re.is_match(s) {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' does not match pattern",
                            record_idx, field
                        )));
                    }
                }
            }
            RuleType::MinValue { min } => {
                if let Some(num) = value.and_then(|v| v.as_f64()) {
                    if num < *min {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' must be at least {}",
                            record_idx, field, min
                        )));
                    }
                }
            }
            RuleType::MaxValue { max } => {
                if let Some(num) = value.and_then(|v| v.as_f64()) {
                    if num > *max {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' must be at most {}",
                            record_idx, field, max
                        )));
                    }
                }
            }
            RuleType::Email => {
                if let Some(serde_json::Value::String(s)) = value {
                    if !is_valid_email(s) {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' is not a valid email",
                            record_idx, field
                        )));
                    }
                }
            }
            RuleType::Url => {
                if let Some(serde_json::Value::String(s)) = value {
                    if !is_valid_url(s) {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' is not a valid URL",
                            record_idx, field
                        )));
                    }
                }
            }
            RuleType::Date { format } => {
                if let Some(serde_json::Value::String(s)) = value {
                    if chrono::NaiveDateTime::parse_from_str(s, format).is_err() {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' is not a valid date (expected format: {})",
                            record_idx, field, format
                        )));
                    }
                }
            }
            RuleType::Enum { values } => {
                if let Some(serde_json::Value::String(s)) = value {
                    if !values.contains(s) {
                        return Err(TransferError::Validation(format!(
                            "Record {}: Field '{}' must be one of: {:?}",
                            record_idx, field, values
                        )));
                    }
                }
            }
            RuleType::Custom { validator: _ } => {
                // Custom validators would be implemented here
                // For now, just pass
            }
        }

        Ok(())
    }

    /// Quick validation (schema only)
    pub fn quick_validate(&self, records: &[DataRecord]) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        result.total_records = records.len();

        if records.is_empty() {
            result.add_warning(ValidationWarning {
                record_index: None,
                field: None,
                message: "No records to validate".to_string(),
            });
            return Ok(result);
        }

        // Check field consistency
        let first_fields: Vec<String> = records[0].field_names();
        for (idx, record) in records.iter().enumerate().skip(1) {
            let record_fields = record.field_names();
            if record_fields != first_fields {
                result.add_warning(ValidationWarning {
                    record_index: Some(idx),
                    field: None,
                    message: "Field set differs from first record".to_string(),
                });
            }
        }

        result.valid_records = records.len();
        Ok(result)
    }
}

/// Simple email validation
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Simple URL validation
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_required_validation() {
        let rules = vec![ValidationRule {
            field: "name".to_string(),
            rule_type: RuleType::Required,
            message: None,
        }];

        let validator = DataValidator::new(rules, TransferConfig::default());

        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John".to_string()));

        let result = validator.validate(&[record]).unwrap();
        assert_eq!(result.valid_records, 1);
    }

    #[test]
    fn test_min_length_validation() {
        let rules = vec![ValidationRule {
            field: "name".to_string(),
            rule_type: RuleType::MinLength { min: 3 },
            message: None,
        }];

        let validator = DataValidator::new(rules, TransferConfig::default());

        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("Jo".to_string()));

        let result = validator.validate(&[record]).unwrap();
        assert_eq!(result.invalid_records, 1);
    }

    #[test]
    fn test_email_validation() {
        let rules = vec![ValidationRule {
            field: "email".to_string(),
            rule_type: RuleType::Email,
            message: None,
        }];

        let validator = DataValidator::new(rules, TransferConfig::default());

        let mut record = DataRecord::new();
        record.set("email".to_string(), Value::String("test@example.com".to_string()));

        let result = validator.validate(&[record]).unwrap();
        assert_eq!(result.valid_records, 1);
    }
}
