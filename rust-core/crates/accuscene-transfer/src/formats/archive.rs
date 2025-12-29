use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    formats::{DataStream, ExportHandler, ImportHandler},
    progress::ProgressTracker,
    DataRecord,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream;
use std::io::{Cursor, Read, Write};
use zip::{ZipArchive, ZipWriter, write::FileOptions, CompressionMethod};

pub struct ArchiveHandler;

#[async_trait]
impl ImportHandler for ArchiveHandler {
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;
        let mut all_records = Vec::new();

        for i in 0..archive.len() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(i as u64, Some("Extracting archive files".to_string()))
                    .await;
            }

            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_string();

            // Skip directories
            if file.is_dir() {
                continue;
            }

            // Read file contents
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            let file_bytes = Bytes::from(contents);

            // Determine format from extension
            let extension = std::path::Path::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            // Import based on file type
            let handler = crate::formats::get_import_handler(extension)?;
            let records = handler.import(file_bytes, config, None).await?;
            all_records.extend(records);
        }

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(all_records)
    }

    async fn import_stream(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<DataStream> {
        let records = self.import(data, config, tracker).await?;
        Ok(Box::pin(stream::iter(records.into_iter().map(Ok))))
    }

    async fn validate(&self, data: Bytes, _config: &TransferConfig) -> Result<()> {
        let cursor = Cursor::new(data);
        let archive = ZipArchive::new(cursor)?;

        if archive.len() == 0 {
            return Err(TransferError::Validation("Archive is empty".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ExportHandler for ArchiveHandler {
    async fn export(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let cursor = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(cursor);

        let compression = if config.enable_compression {
            CompressionMethod::Deflated
        } else {
            CompressionMethod::Stored
        };

        let options = FileOptions::default()
            .compression_method(compression)
            .unix_permissions(0o644);

        // Export as multiple formats
        let formats = vec![
            ("data.json", "json"),
            ("data.csv", "csv"),
            ("data.xml", "xml"),
        ];

        for (idx, (filename, format)) in formats.iter().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(
                    idx as u64,
                    Some(format!("Adding {} to archive", filename)),
                )
                .await;
            }

            // Get handler and export
            let handler = crate::formats::get_export_handler(format)?;
            let data = handler.export(records.clone(), config, None).await?;

            // Add to archive
            zip.start_file(*filename, options)?;
            zip.write_all(&data)?;
        }

        // Add metadata file
        let metadata = create_metadata(&records, config);
        zip.start_file("metadata.json", options)?;
        zip.write_all(metadata.as_bytes())?;

        // Finish archive
        let result = zip.finish()?;
        let archive_bytes = result.into_inner();

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(Bytes::from(archive_bytes))
    }

    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        self.export(records, config, tracker).await
    }
}

/// Create metadata JSON for archive
fn create_metadata(records: &[DataRecord], config: &TransferConfig) -> String {
    let metadata = crate::TransferMetadata::new(crate::TransferFormat::Archive)
        .with_record_count(records.len())
        .with_field_count(
            records
                .first()
                .map(|r| r.field_names().len())
                .unwrap_or(0),
        );

    serde_json::to_string_pretty(&metadata).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[tokio::test]
    async fn test_archive_export_import() {
        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John Doe".to_string()));
        record.set("age".to_string(), Value::Number(30.into()));

        let records = vec![record];
        let config = TransferConfig::default();

        let handler = ArchiveHandler;
        let exported = handler.export(records.clone(), &config, None).await.unwrap();

        // Verify it's a valid ZIP
        assert!(!exported.is_empty());

        // Verify we can read it back
        let validation = handler.validate(exported.clone(), &config).await;
        assert!(validation.is_ok());
    }
}
