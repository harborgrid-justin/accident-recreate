//! Report generation framework

pub mod builder;
pub mod export;

pub use builder::{Report, ReportBuilder, ReportSection};
pub use export::{CsvExporter, JsonExporter, ReportExporter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
    Pdf,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub title: String,
    pub description: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub version: String,
    pub tags: HashMap<String, String>,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self {
            title: "Analytics Report".to_string(),
            description: String::new(),
            generated_at: Utc::now(),
            generated_by: "AccuScene Analytics".to_string(),
            version: "0.2.0".to_string(),
            tags: HashMap::new(),
        }
    }
}
