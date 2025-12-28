use crate::{error::{Result, TransferError}, DataRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Data transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Transform {
    /// Convert to uppercase
    ToUpperCase,
    /// Convert to lowercase
    ToLowerCase,
    /// Trim whitespace
    Trim,
    /// Replace text
    Replace { from: String, to: String },
    /// Format number
    FormatNumber { decimals: usize },
    /// Parse number
    ParseNumber,
    /// Format date
    FormatDate { from: String, to: String },
    /// Add prefix
    Prefix { prefix: String },
    /// Add suffix
    Suffix { suffix: String },
    /// Substring
    Substring { start: usize, end: Option<usize> },
    /// Concatenate fields
    Concat { fields: Vec<String>, separator: String },
    /// Split string
    Split { delimiter: String, index: usize },
    /// Default value if null
    DefaultIfNull { default: Value },
    /// Convert type
    ConvertType { target_type: String },
    /// Custom function (named)
    Custom { function: String },
}

impl Transform {
    /// Apply transformation to value
    pub fn apply(&self, value: Value) -> Result<Value> {
        match self {
            Transform::ToUpperCase => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(s.to_uppercase()))
                } else {
                    Ok(value)
                }
            }
            Transform::ToLowerCase => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(s.to_lowercase()))
                } else {
                    Ok(value)
                }
            }
            Transform::Trim => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(s.trim().to_string()))
                } else {
                    Ok(value)
                }
            }
            Transform::Replace { from, to } => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(s.replace(from, to)))
                } else {
                    Ok(value)
                }
            }
            Transform::FormatNumber { decimals } => {
                if let Some(num) = value.as_f64() {
                    Ok(Value::String(format!("{:.prec$}", num, prec = decimals)))
                } else {
                    Ok(value)
                }
            }
            Transform::ParseNumber => {
                if let Some(s) = value.as_str() {
                    if let Ok(num) = s.parse::<f64>() {
                        if let Some(num) = serde_json::Number::from_f64(num) {
                            return Ok(Value::Number(num));
                        }
                    }
                }
                Ok(value)
            }
            Transform::FormatDate { from, to } => {
                if let Some(s) = value.as_str() {
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, from) {
                        Ok(Value::String(dt.format(to).to_string()))
                    } else {
                        Ok(value)
                    }
                } else {
                    Ok(value)
                }
            }
            Transform::Prefix { prefix } => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(format!("{}{}", prefix, s)))
                } else {
                    Ok(value)
                }
            }
            Transform::Suffix { suffix } => {
                if let Some(s) = value.as_str() {
                    Ok(Value::String(format!("{}{}", s, suffix)))
                } else {
                    Ok(value)
                }
            }
            Transform::Substring { start, end } => {
                if let Some(s) = value.as_str() {
                    let chars: Vec<char> = s.chars().collect();
                    let end_idx = end.unwrap_or(chars.len());
                    let substr: String = chars
                        .iter()
                        .skip(*start)
                        .take(end_idx - start)
                        .collect();
                    Ok(Value::String(substr))
                } else {
                    Ok(value)
                }
            }
            Transform::Split { delimiter, index } => {
                if let Some(s) = value.as_str() {
                    let parts: Vec<&str> = s.split(delimiter).collect();
                    if *index < parts.len() {
                        Ok(Value::String(parts[*index].to_string()))
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    Ok(value)
                }
            }
            Transform::DefaultIfNull { default } => {
                if value.is_null() {
                    Ok(default.clone())
                } else {
                    Ok(value)
                }
            }
            Transform::ConvertType { target_type } => {
                convert_type(value, target_type)
            }
            Transform::Concat { .. } => {
                // Concat requires access to multiple fields, handled separately
                Ok(value)
            }
            Transform::Custom { function } => {
                // Custom functions would be implemented via plugins
                Err(TransferError::Transform(format!(
                    "Custom function '{}' not implemented",
                    function
                )))
            }
        }
    }

    /// Apply transformation to entire record
    pub fn apply_to_record(&self, mut record: DataRecord) -> Result<DataRecord> {
        match self {
            Transform::Concat { fields, separator } => {
                let parts: Vec<String> = fields
                    .iter()
                    .filter_map(|f| {
                        record.get(f).and_then(|v| {
                            if let Some(s) = v.as_str() {
                                Some(s.to_string())
                            } else if !v.is_null() {
                                Some(v.to_string())
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                let result = parts.join(separator);
                record.set("_concat".to_string(), Value::String(result));
                Ok(record)
            }
            _ => {
                // Apply to all fields
                let field_names = record.field_names();
                for field_name in field_names {
                    if let Some(value) = record.get(&field_name).cloned() {
                        let transformed = self.apply(value)?;
                        record.set(field_name, transformed);
                    }
                }
                Ok(record)
            }
        }
    }
}

/// Transform pipeline
pub struct TransformPipeline {
    transforms: Vec<Transform>,
}

impl TransformPipeline {
    /// Create new pipeline
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    /// Add transform
    pub fn add(mut self, transform: Transform) -> Self {
        self.transforms.push(transform);
        self
    }

    /// Apply all transforms
    pub fn apply(&self, value: Value) -> Result<Value> {
        let mut result = value;
        for transform in &self.transforms {
            result = transform.apply(result)?;
        }
        Ok(result)
    }

    /// Apply pipeline to record
    pub fn apply_to_record(&self, record: DataRecord) -> Result<DataRecord> {
        let mut result = record;
        for transform in &self.transforms {
            result = transform.apply_to_record(result)?;
        }
        Ok(result)
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert value to target type
fn convert_type(value: Value, target_type: &str) -> Result<Value> {
    match target_type.to_lowercase().as_str() {
        "string" => Ok(Value::String(value.to_string())),
        "number" | "float" => {
            if let Some(n) = value.as_f64() {
                Ok(Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into())))
            } else if let Some(s) = value.as_str() {
                if let Ok(n) = s.parse::<f64>() {
                    Ok(Value::Number(serde_json::Number::from_f64(n).unwrap_or(0.into())))
                } else {
                    Err(TransferError::Transform(format!("Cannot convert '{}' to number", s)))
                }
            } else {
                Err(TransferError::Transform("Cannot convert to number".to_string()))
            }
        }
        "integer" | "int" => {
            if let Some(n) = value.as_i64() {
                Ok(Value::Number(n.into()))
            } else if let Some(s) = value.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    Ok(Value::Number(n.into()))
                } else {
                    Err(TransferError::Transform(format!("Cannot convert '{}' to integer", s)))
                }
            } else {
                Err(TransferError::Transform("Cannot convert to integer".to_string()))
            }
        }
        "boolean" | "bool" => {
            if let Some(b) = value.as_bool() {
                Ok(Value::Bool(b))
            } else if let Some(s) = value.as_str() {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "1" => Ok(Value::Bool(true)),
                    "false" | "no" | "0" => Ok(Value::Bool(false)),
                    _ => Err(TransferError::Transform(format!("Cannot convert '{}' to boolean", s))),
                }
            } else {
                Err(TransferError::Transform("Cannot convert to boolean".to_string()))
            }
        }
        _ => Err(TransferError::Transform(format!("Unknown target type: {}", target_type))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uppercase_transform() {
        let transform = Transform::ToUpperCase;
        let value = Value::String("hello".to_string());
        let result = transform.apply(value).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_trim_transform() {
        let transform = Transform::Trim;
        let value = Value::String("  hello  ".to_string());
        let result = transform.apply(value).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_replace_transform() {
        let transform = Transform::Replace {
            from: "old".to_string(),
            to: "new".to_string(),
        };
        let value = Value::String("old value".to_string());
        let result = transform.apply(value).unwrap();
        assert_eq!(result, Value::String("new value".to_string()));
    }

    #[test]
    fn test_pipeline() {
        let pipeline = TransformPipeline::new()
            .add(Transform::Trim)
            .add(Transform::ToUpperCase);

        let value = Value::String("  hello  ".to_string());
        let result = pipeline.apply(value).unwrap();
        assert_eq!(result, Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_convert_type() {
        let result = convert_type(Value::String("42".to_string()), "integer").unwrap();
        assert_eq!(result, Value::Number(42.into()));

        let result = convert_type(Value::String("true".to_string()), "boolean").unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}
