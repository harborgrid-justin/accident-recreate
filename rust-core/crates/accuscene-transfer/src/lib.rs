/// AccuScene Enterprise Data Transfer Library
///
/// Provides comprehensive import/export capabilities with support for:
/// - CSV, Excel, JSON, XML, PDF, and Archive formats
/// - Streaming for large files
/// - Progress tracking
/// - Field mapping and transformation
/// - Schema detection and validation
/// - Error recovery

pub mod config;
pub mod error;
pub mod formats;
pub mod mapping;
pub mod progress;
pub mod validation;

pub use config::{TransferConfig, TransferFormat};
pub use error::{Result, TransferError};
pub use progress::{ProgressStatus, ProgressTracker, TransferProgress};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetadata {
    /// Source format
    pub source_format: Option<TransferFormat>,
    /// Target format
    pub target_format: TransferFormat,
    /// Record count
    pub record_count: usize,
    /// Field count
    pub field_count: usize,
    /// File size in bytes
    pub file_size: u64,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Creator/exporter info
    pub created_by: Option<String>,
    /// Schema version
    pub schema_version: String,
    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl TransferMetadata {
    /// Create new metadata
    pub fn new(target_format: TransferFormat) -> Self {
        Self {
            source_format: None,
            target_format,
            record_count: 0,
            field_count: 0,
            file_size: 0,
            created_at: chrono::Utc::now(),
            created_by: None,
            schema_version: env!("CARGO_PKG_VERSION").to_string(),
            custom: HashMap::new(),
        }
    }

    /// Set source format
    pub fn with_source_format(mut self, format: TransferFormat) -> Self {
        self.source_format = Some(format);
        self
    }

    /// Set record count
    pub fn with_record_count(mut self, count: usize) -> Self {
        self.record_count = count;
        self
    }

    /// Set field count
    pub fn with_field_count(mut self, count: usize) -> Self {
        self.field_count = count;
        self
    }

    /// Add custom metadata
    pub fn with_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// Generic data record for transfer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub fields: HashMap<String, serde_json::Value>,
}

impl DataRecord {
    /// Create new empty record
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Create from fields
    pub fn from_fields(fields: HashMap<String, serde_json::Value>) -> Self {
        Self { fields }
    }

    /// Get field value
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    /// Set field value
    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.fields.insert(key, value);
    }

    /// Get all field names
    pub fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }
}

impl Default for DataRecord {
    fn default() -> Self {
        Self::new()
    }
}
