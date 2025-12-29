use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transfer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: u64,
    /// Chunk size for streaming (default: 1MB)
    pub chunk_size: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (0-9)
    pub compression_level: u32,
    /// CSV delimiter
    pub csv_delimiter: char,
    /// CSV has header row
    pub csv_has_header: bool,
    /// Excel sheet index (0-based)
    pub excel_sheet_index: usize,
    /// Excel sheet name
    pub excel_sheet_name: Option<String>,
    /// JSON pretty print
    pub json_pretty: bool,
    /// XML root element name
    pub xml_root_element: String,
    /// Include metadata
    pub include_metadata: bool,
    /// Custom field mappings
    pub field_mappings: HashMap<String, String>,
    /// Date format
    pub date_format: String,
    /// Timezone
    pub timezone: String,
    /// Skip validation
    pub skip_validation: bool,
    /// Continue on error
    pub continue_on_error: bool,
    /// Error threshold (max errors before stopping)
    pub error_threshold: usize,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            chunk_size: 1024 * 1024,          // 1MB
            enable_compression: true,
            compression_level: 6,
            csv_delimiter: ',',
            csv_has_header: true,
            excel_sheet_index: 0,
            excel_sheet_name: None,
            json_pretty: true,
            xml_root_element: "data".to_string(),
            include_metadata: true,
            field_mappings: HashMap::new(),
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            timezone: "UTC".to_string(),
            skip_validation: false,
            continue_on_error: false,
            error_threshold: 100,
        }
    }
}

impl TransferConfig {
    /// Create new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: Set max file size
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Builder: Set chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Builder: Enable/disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.enable_compression = enabled;
        self
    }

    /// Builder: Set CSV delimiter
    pub fn with_csv_delimiter(mut self, delimiter: char) -> Self {
        self.csv_delimiter = delimiter;
        self
    }

    /// Builder: Set CSV header flag
    pub fn with_csv_header(mut self, has_header: bool) -> Self {
        self.csv_has_header = has_header;
        self
    }

    /// Builder: Set Excel sheet
    pub fn with_excel_sheet(mut self, index: usize, name: Option<String>) -> Self {
        self.excel_sheet_index = index;
        self.excel_sheet_name = name;
        self
    }

    /// Builder: Set JSON pretty print
    pub fn with_json_pretty(mut self, pretty: bool) -> Self {
        self.json_pretty = pretty;
        self
    }

    /// Builder: Add field mapping
    pub fn with_field_mapping(mut self, from: String, to: String) -> Self {
        self.field_mappings.insert(from, to);
        self
    }

    /// Builder: Set date format
    pub fn with_date_format(mut self, format: String) -> Self {
        self.date_format = format;
        self
    }

    /// Builder: Set error handling
    pub fn with_error_handling(mut self, continue_on_error: bool, threshold: usize) -> Self {
        self.continue_on_error = continue_on_error;
        self.error_threshold = threshold;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_file_size == 0 {
            return Err("max_file_size must be greater than 0".to_string());
        }
        if self.chunk_size == 0 {
            return Err("chunk_size must be greater than 0".to_string());
        }
        if self.compression_level > 9 {
            return Err("compression_level must be between 0 and 9".to_string());
        }
        if self.xml_root_element.is_empty() {
            return Err("xml_root_element cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Export/Import format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferFormat {
    Csv,
    Excel,
    Json,
    Xml,
    Pdf,
    Archive,
}

impl TransferFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &str {
        match self {
            Self::Csv => "csv",
            Self::Excel => "xlsx",
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Pdf => "pdf",
            Self::Archive => "zip",
        }
    }

    /// Get MIME type for format
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Csv => "text/csv",
            Self::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Json => "application/json",
            Self::Xml => "application/xml",
            Self::Pdf => "application/pdf",
            Self::Archive => "application/zip",
        }
    }

    /// Check if format supports import
    pub fn supports_import(&self) -> bool {
        !matches!(self, Self::Pdf)
    }

    /// Check if format supports export
    pub fn supports_export(&self) -> bool {
        true
    }
}
