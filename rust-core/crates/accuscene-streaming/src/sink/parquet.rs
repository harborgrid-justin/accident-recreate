//! Parquet file sink for columnar data storage.

use crate::error::{Result, StreamingError};
use crate::sink::Sink;
use arrow::array::{ArrayRef, RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use async_trait::async_trait;
use parquet::arrow::AsyncArrowWriter;
use parquet::file::properties::WriterProperties;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;

/// Parquet sink that writes Arrow RecordBatches to Parquet files
pub struct ParquetSink {
    path: PathBuf,
    schema: Arc<Schema>,
    writer: Option<AsyncArrowWriter<File>>,
    batch_size: usize,
    buffer: Vec<RecordBatch>,
    writer_properties: Option<WriterProperties>,
}

impl ParquetSink {
    /// Create a new Parquet sink
    pub fn new(path: PathBuf, schema: Arc<Schema>, batch_size: usize) -> Self {
        Self {
            path,
            schema,
            writer: None,
            batch_size,
            buffer: Vec::new(),
            writer_properties: None,
        }
    }

    /// Set writer properties
    pub fn with_properties(mut self, properties: WriterProperties) -> Self {
        self.writer_properties = Some(properties);
        self
    }

    async fn init_writer(&mut self) -> Result<()> {
        if self.writer.is_some() {
            return Ok(());
        }

        let file = File::create(&self.path).await.map_err(|e| {
            StreamingError::Parquet(format!("Failed to create Parquet file: {}", e))
        })?;

        let props = self.writer_properties.clone();
        let writer = if let Some(props) = props {
            AsyncArrowWriter::try_new(file, self.schema.clone(), Some(props))
        } else {
            AsyncArrowWriter::try_new(file, self.schema.clone(), None)
        }
        .map_err(|e| {
            StreamingError::Parquet(format!("Failed to create Parquet writer: {}", e))
        })?;

        self.writer = Some(writer);
        Ok(())
    }

    async fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        self.init_writer().await?;

        let writer = self.writer.as_mut().unwrap();

        for batch in self.buffer.drain(..) {
            writer.write(&batch).await.map_err(|e| {
                StreamingError::Parquet(format!("Failed to write RecordBatch: {}", e))
            })?;
        }

        Ok(())
    }
}

#[async_trait]
impl Sink<RecordBatch> for ParquetSink {
    async fn write(&mut self, item: RecordBatch) -> Result<()> {
        self.buffer.push(item);

        if self.buffer.len() >= self.batch_size {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.flush_buffer().await?;

        if let Some(writer) = &mut self.writer {
            writer.flush().await.map_err(|e| {
                StreamingError::Parquet(format!("Failed to flush Parquet writer: {}", e))
            })?;
        }

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.flush_buffer().await?;

        if let Some(writer) = self.writer.take() {
            writer.close().await.map_err(|e| {
                StreamingError::Parquet(format!("Failed to close Parquet writer: {}", e))
            })?;
        }

        Ok(())
    }
}

/// Helper to create a simple string RecordBatch
pub fn create_string_batch(
    field_name: &str,
    values: Vec<String>,
) -> Result<RecordBatch> {
    let mut builder = StringBuilder::new();
    for value in values {
        builder.append_value(value);
    }
    let array: ArrayRef = Arc::new(builder.finish());

    let schema = Arc::new(Schema::new(vec![Field::new(
        field_name,
        DataType::Utf8,
        false,
    )]));

    RecordBatch::try_new(schema, vec![array])
        .map_err(|e| StreamingError::Arrow(format!("Failed to create RecordBatch: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_parquet_sink() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let mut sink = ParquetSink::new(path.clone(), schema.clone(), 2);

        // Create sample data
        let ids: ArrayRef = Arc::new(Int32Array::from(vec![1, 2]));
        let names: ArrayRef = Arc::new(arrow::array::StringArray::from(vec!["Alice", "Bob"]));

        let batch = RecordBatch::try_new(schema.clone(), vec![ids, names]).unwrap();

        sink.write(batch).await.unwrap();
        sink.close().await.unwrap();

        // Verify file exists
        assert!(path.exists());
    }
}
