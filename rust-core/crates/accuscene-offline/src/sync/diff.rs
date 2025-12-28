use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// Type of diff operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffOp {
    /// Add a value at path
    Add { path: String, value: Value },

    /// Remove a value at path
    Remove { path: String },

    /// Replace a value at path
    Replace { path: String, value: Value },

    /// Move a value from one path to another
    Move { from: String, to: String },

    /// Copy a value from one path to another
    Copy { from: String, to: String },

    /// Test if value at path matches
    Test { path: String, value: Value },
}

/// A diff between two JSON values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    /// List of operations to transform source to target
    pub operations: Vec<DiffOp>,

    /// Metadata about the diff
    pub metadata: DiffMetadata,
}

/// Metadata about a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffMetadata {
    /// Number of additions
    pub additions: usize,

    /// Number of deletions
    pub deletions: usize,

    /// Number of modifications
    pub modifications: usize,

    /// Estimated size of diff in bytes
    pub size_bytes: usize,

    /// Compression ratio if compressed
    pub compression_ratio: Option<f64>,
}

/// Differential sync algorithm implementation
pub struct DiffEngine;

impl DiffEngine {
    /// Compute diff between two JSON values
    pub fn compute_diff(source: &Value, target: &Value) -> Diff {
        let mut operations = Vec::new();
        Self::compute_diff_recursive("", source, target, &mut operations);

        let metadata = Self::calculate_metadata(&operations);

        Diff {
            operations,
            metadata,
        }
    }

    /// Recursively compute diff operations
    fn compute_diff_recursive(path: &str, source: &Value, target: &Value, ops: &mut Vec<DiffOp>) {
        match (source, target) {
            (Value::Object(src_obj), Value::Object(tgt_obj)) => {
                // Find all keys
                let src_keys: HashSet<_> = src_obj.keys().collect();
                let tgt_keys: HashSet<_> = tgt_obj.keys().collect();

                // Removed keys
                for key in src_keys.difference(&tgt_keys) {
                    let key_path = Self::join_path(path, key);
                    ops.push(DiffOp::Remove { path: key_path });
                }

                // Added keys
                for key in tgt_keys.difference(&src_keys) {
                    let key_path = Self::join_path(path, key);
                    ops.push(DiffOp::Add {
                        path: key_path,
                        value: tgt_obj[*key].clone(),
                    });
                }

                // Modified keys
                for key in src_keys.intersection(&tgt_keys) {
                    let key_path = Self::join_path(path, key);
                    let src_val = &src_obj[*key];
                    let tgt_val = &tgt_obj[*key];

                    if src_val != tgt_val {
                        Self::compute_diff_recursive(&key_path, src_val, tgt_val, ops);
                    }
                }
            }
            (Value::Array(src_arr), Value::Array(tgt_arr)) => {
                // Simple array diff using LCS (Longest Common Subsequence)
                let diff_ops = Self::array_diff(path, src_arr, tgt_arr);
                ops.extend(diff_ops);
            }
            _ => {
                // Different types or primitive values
                if source != target {
                    ops.push(DiffOp::Replace {
                        path: path.to_string(),
                        value: target.clone(),
                    });
                }
            }
        }
    }

    /// Compute array diff using simple LCS-based algorithm
    fn array_diff(path: &str, source: &[Value], target: &[Value]) -> Vec<DiffOp> {
        let mut ops = Vec::new();

        // Simple replacement for now (can be optimized with Myers diff algorithm)
        if source.len() == target.len() {
            for (i, (src_val, tgt_val)) in source.iter().zip(target.iter()).enumerate() {
                if src_val != tgt_val {
                    ops.push(DiffOp::Replace {
                        path: format!("{}/{}", path, i),
                        value: tgt_val.clone(),
                    });
                }
            }
        } else {
            // Full replacement for simplicity
            ops.push(DiffOp::Replace {
                path: path.to_string(),
                value: Value::Array(target.to_vec()),
            });
        }

        ops
    }

    /// Apply diff operations to a value
    pub fn apply_diff(source: &Value, diff: &Diff) -> Result<Value, String> {
        let mut result = source.clone();

        for op in &diff.operations {
            result = Self::apply_operation(&result, op)?;
        }

        Ok(result)
    }

    /// Apply a single diff operation
    fn apply_operation(value: &Value, op: &DiffOp) -> Result<Value, String> {
        match op {
            DiffOp::Add { path, value: new_val } => {
                Self::set_at_path(value, path, new_val.clone())
            }
            DiffOp::Remove { path } => {
                Self::remove_at_path(value, path)
            }
            DiffOp::Replace { path, value: new_val } => {
                Self::set_at_path(value, path, new_val.clone())
            }
            DiffOp::Move { from, to } => {
                let val = Self::get_at_path(value, from)?;
                let temp = Self::remove_at_path(value, from)?;
                Self::set_at_path(&temp, to, val)
            }
            DiffOp::Copy { from, to } => {
                let val = Self::get_at_path(value, from)?;
                Self::set_at_path(value, to, val)
            }
            DiffOp::Test { path, value: expected } => {
                let actual = Self::get_at_path(value, path)?;
                if &actual != expected {
                    return Err(format!("Test failed at path {}", path));
                }
                Ok(value.clone())
            }
        }
    }

    /// Get value at JSON pointer path
    fn get_at_path(value: &Value, path: &str) -> Result<Value, String> {
        if path.is_empty() || path == "/" {
            return Ok(value.clone());
        }

        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut current = value;

        for part in parts {
            match current {
                Value::Object(obj) => {
                    current = obj.get(part)
                        .ok_or_else(|| format!("Path not found: {}", path))?;
                }
                Value::Array(arr) => {
                    let idx: usize = part.parse()
                        .map_err(|_| format!("Invalid array index: {}", part))?;
                    current = arr.get(idx)
                        .ok_or_else(|| format!("Array index out of bounds: {}", idx))?;
                }
                _ => return Err(format!("Cannot navigate path in non-object/array: {}", path)),
            }
        }

        Ok(current.clone())
    }

    /// Set value at JSON pointer path
    fn set_at_path(value: &Value, path: &str, new_value: Value) -> Result<Value, String> {
        if path.is_empty() || path == "/" {
            return Ok(new_value);
        }

        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        Self::set_at_path_recursive(value, &parts, 0, new_value)
    }

    fn set_at_path_recursive(
        value: &Value,
        parts: &[&str],
        index: usize,
        new_value: Value,
    ) -> Result<Value, String> {
        if index >= parts.len() {
            return Ok(new_value);
        }

        let part = parts[index];

        match value {
            Value::Object(obj) => {
                let mut new_obj = obj.clone();
                let child = obj.get(part).unwrap_or(&Value::Null);
                let new_child = Self::set_at_path_recursive(child, parts, index + 1, new_value)?;
                new_obj.insert(part.to_string(), new_child);
                Ok(Value::Object(new_obj))
            }
            Value::Array(arr) => {
                let idx: usize = part.parse()
                    .map_err(|_| format!("Invalid array index: {}", part))?;
                let mut new_arr = arr.clone();
                if idx < new_arr.len() {
                    let new_child = Self::set_at_path_recursive(&arr[idx], parts, index + 1, new_value)?;
                    new_arr[idx] = new_child;
                } else {
                    return Err(format!("Array index out of bounds: {}", idx));
                }
                Ok(Value::Array(new_arr))
            }
            _ => Err(format!("Cannot set path in non-object/array")),
        }
    }

    /// Remove value at JSON pointer path
    fn remove_at_path(value: &Value, path: &str) -> Result<Value, String> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        Self::remove_at_path_recursive(value, &parts, 0)
    }

    fn remove_at_path_recursive(value: &Value, parts: &[&str], index: usize) -> Result<Value, String> {
        if parts.is_empty() || index >= parts.len() {
            return Err("Cannot remove root".to_string());
        }

        let part = parts[index];

        match value {
            Value::Object(obj) => {
                let mut new_obj = obj.clone();
                if index == parts.len() - 1 {
                    new_obj.remove(part);
                } else {
                    let child = obj.get(part).ok_or_else(|| format!("Path not found: {}", part))?;
                    let new_child = Self::remove_at_path_recursive(child, parts, index + 1)?;
                    new_obj.insert(part.to_string(), new_child);
                }
                Ok(Value::Object(new_obj))
            }
            Value::Array(arr) => {
                let idx: usize = part.parse()
                    .map_err(|_| format!("Invalid array index: {}", part))?;
                let mut new_arr = arr.clone();
                if index == parts.len() - 1 {
                    if idx < new_arr.len() {
                        new_arr.remove(idx);
                    }
                } else {
                    let new_child = Self::remove_at_path_recursive(&arr[idx], parts, index + 1)?;
                    new_arr[idx] = new_child;
                }
                Ok(Value::Array(new_arr))
            }
            _ => Err("Cannot remove from non-object/array".to_string()),
        }
    }

    /// Join path components
    fn join_path(base: &str, component: &str) -> String {
        if base.is_empty() {
            format!("/{}", component)
        } else {
            format!("{}/{}", base, component)
        }
    }

    /// Calculate diff metadata
    fn calculate_metadata(operations: &[DiffOp]) -> DiffMetadata {
        let mut additions = 0;
        let mut deletions = 0;
        let mut modifications = 0;

        for op in operations {
            match op {
                DiffOp::Add { .. } => additions += 1,
                DiffOp::Remove { .. } => deletions += 1,
                DiffOp::Replace { .. } => modifications += 1,
                DiffOp::Move { .. } => modifications += 1,
                DiffOp::Copy { .. } => additions += 1,
                DiffOp::Test { .. } => {}
            }
        }

        let size_bytes = bincode::serialize(operations)
            .map(|bytes| bytes.len())
            .unwrap_or(0);

        DiffMetadata {
            additions,
            deletions,
            modifications,
            size_bytes,
            compression_ratio: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_diff() {
        let source = serde_json::json!({
            "name": "Alice",
            "age": 30
        });

        let target = serde_json::json!({
            "name": "Alice",
            "age": 31,
            "city": "NYC"
        });

        let diff = DiffEngine::compute_diff(&source, &target);

        assert!(diff.operations.len() > 0);
        assert_eq!(diff.metadata.additions, 1); // city added
        assert_eq!(diff.metadata.modifications, 1); // age modified
    }

    #[test]
    fn test_apply_diff() {
        let source = serde_json::json!({
            "name": "Alice",
            "age": 30
        });

        let target = serde_json::json!({
            "name": "Alice",
            "age": 31,
            "city": "NYC"
        });

        let diff = DiffEngine::compute_diff(&source, &target);
        let result = DiffEngine::apply_diff(&source, &diff).unwrap();

        assert_eq!(result["age"], 31);
        assert_eq!(result["city"], "NYC");
    }
}
