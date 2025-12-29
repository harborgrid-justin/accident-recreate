/// Format handlers for import/export operations
pub mod archive;
pub mod csv;
pub mod excel;
pub mod json;
pub mod pdf;
pub mod xml;

use crate::{error::Result, config::TransferConfig, DataRecord, progress::ProgressTracker};
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Streaming data reader
pub type DataStream = Pin<Box<dyn Stream<Item = Result<DataRecord>> + Send>>;

/// Format handler trait for imports
#[async_trait]
pub trait ImportHandler: Send + Sync {
    /// Import data from bytes with progress tracking
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>>;

    /// Import data as stream for large files
    async fn import_stream(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<DataStream>;

    /// Validate import data
    async fn validate(&self, data: Bytes, config: &TransferConfig) -> Result<()>;
}

/// Format handler trait for exports
#[async_trait]
pub trait ExportHandler: Send + Sync {
    /// Export data to bytes
    async fn export(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes>;

    /// Export data as stream for large datasets
    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes>;
}

/// Get import handler for format
pub fn get_import_handler(format: &str) -> Result<Box<dyn ImportHandler>> {
    match format.to_lowercase().as_str() {
        "csv" => Ok(Box::new(csv::CsvHandler)),
        "excel" | "xlsx" => Ok(Box::new(excel::ExcelHandler)),
        "json" => Ok(Box::new(json::JsonHandler)),
        "xml" => Ok(Box::new(xml::XmlHandler)),
        "zip" | "archive" => Ok(Box::new(archive::ArchiveHandler)),
        _ => Err(crate::error::TransferError::UnsupportedFormat(
            format.to_string(),
        )),
    }
}

/// Get export handler for format
pub fn get_export_handler(format: &str) -> Result<Box<dyn ExportHandler>> {
    match format.to_lowercase().as_str() {
        "csv" => Ok(Box::new(csv::CsvHandler)),
        "excel" | "xlsx" => Ok(Box::new(excel::ExcelHandler)),
        "json" => Ok(Box::new(json::JsonHandler)),
        "xml" => Ok(Box::new(xml::XmlHandler)),
        "pdf" => Ok(Box::new(pdf::PdfHandler)),
        "zip" | "archive" => Ok(Box::new(archive::ArchiveHandler)),
        _ => Err(crate::error::TransferError::UnsupportedFormat(
            format.to_string(),
        )),
    }
}
