//! File-based streaming sink with rotation support.

use crate::error::{Result, StreamingError};
use crate::sink::Sink;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};

/// File sink that writes data to a file
pub struct FileSink {
    path: PathBuf,
    writer: Option<BufWriter<File>>,
    append: bool,
}

impl FileSink {
    /// Create a new file sink
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            writer: None,
            append: false,
        }
    }

    /// Create a file sink that appends to existing file
    pub fn new_append(path: PathBuf) -> Self {
        Self {
            path,
            writer: None,
            append: true,
        }
    }

    /// Initialize the writer
    async fn init_writer(&mut self) -> Result<()> {
        if self.writer.is_some() {
            return Ok(());
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(self.append)
            .truncate(!self.append)
            .open(&self.path)
            .await
            .map_err(|e| StreamingError::Sink(format!("Failed to open file: {}", e)))?;

        self.writer = Some(BufWriter::new(file));
        Ok(())
    }
}

#[async_trait]
impl Sink<String> for FileSink {
    async fn write(&mut self, item: String) -> Result<()> {
        self.init_writer().await?;

        let writer = self.writer.as_mut().unwrap();
        writer
            .write_all(item.as_bytes())
            .await
            .map_err(|e| StreamingError::Sink(format!("File write error: {}", e)))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| StreamingError::Sink(format!("File write error: {}", e)))?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::Sink(format!("File flush error: {}", e)))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await?;
        self.writer = None;
        Ok(())
    }
}

#[async_trait]
impl Sink<Vec<u8>> for FileSink {
    async fn write(&mut self, item: Vec<u8>) -> Result<()> {
        self.init_writer().await?;

        let writer = self.writer.as_mut().unwrap();
        writer
            .write_all(&item)
            .await
            .map_err(|e| StreamingError::Sink(format!("File write error: {}", e)))?;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::Sink(format!("File flush error: {}", e)))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await?;
        self.writer = None;
        Ok(())
    }
}

/// File sink with automatic rotation based on size or time
pub struct RotatingFileSink {
    base_path: PathBuf,
    max_size: u64,
    current_size: u64,
    rotation_count: usize,
    writer: Option<BufWriter<File>>,
}

impl RotatingFileSink {
    /// Create a new rotating file sink
    pub fn new(base_path: PathBuf, max_size: u64) -> Self {
        Self {
            base_path,
            max_size,
            current_size: 0,
            rotation_count: 0,
            writer: None,
        }
    }

    async fn rotate(&mut self) -> Result<()> {
        // Close current file
        if let Some(mut writer) = self.writer.take() {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::FileRotation(format!("Failed to flush: {}", e)))?;
        }

        // Open new file
        self.rotation_count += 1;
        let new_path = self
            .base_path
            .with_file_name(format!(
                "{}.{}",
                self.base_path.file_name().unwrap().to_string_lossy(),
                self.rotation_count
            ));

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&new_path)
            .await
            .map_err(|e| StreamingError::FileRotation(format!("Failed to open new file: {}", e)))?;

        self.writer = Some(BufWriter::new(file));
        self.current_size = 0;

        Ok(())
    }

    async fn ensure_writer(&mut self) -> Result<()> {
        if self.writer.is_none() {
            self.rotate().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Sink<String> for RotatingFileSink {
    async fn write(&mut self, item: String) -> Result<()> {
        let item_size = item.len() as u64 + 1; // +1 for newline

        if self.current_size + item_size > self.max_size {
            self.rotate().await?;
        }

        self.ensure_writer().await?;

        let writer = self.writer.as_mut().unwrap();
        writer
            .write_all(item.as_bytes())
            .await
            .map_err(|e| StreamingError::Sink(format!("File write error: {}", e)))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| StreamingError::Sink(format!("File write error: {}", e)))?;

        self.current_size += item_size;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.writer {
            writer
                .flush()
                .await
                .map_err(|e| StreamingError::Sink(format!("File flush error: {}", e)))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.flush().await?;
        self.writer = None;
        Ok(())
    }
}

/// JSON lines file sink
pub struct JsonLinesSink<T> {
    inner: FileSink,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonLinesSink<T>
where
    T: serde::Serialize + Send + 'static,
{
    /// Create a new JSON lines sink
    pub fn new(path: PathBuf) -> Self {
        Self {
            inner: FileSink::new(path),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a JSON lines sink that appends
    pub fn new_append(path: PathBuf) -> Self {
        Self {
            inner: FileSink::new_append(path),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> Sink<T> for JsonLinesSink<T>
where
    T: serde::Serialize + Send + 'static,
{
    async fn write(&mut self, item: T) -> Result<()> {
        let json = serde_json::to_string(&item)
            .map_err(|e| StreamingError::Sink(format!("JSON serialization error: {}", e)))?;
        self.inner.write(json).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.inner.flush().await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_sink() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let mut sink = FileSink::new(path.clone());

        sink.write("line1".to_string()).await.unwrap();
        sink.write("line2".to_string()).await.unwrap();
        sink.write("line3".to_string()).await.unwrap();

        sink.close().await.unwrap();

        let content = fs::read_to_string(path).await.unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
    }

    #[tokio::test]
    async fn test_rotating_file_sink() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let mut sink = RotatingFileSink::new(path.clone(), 20);

        sink.write("line1".to_string()).await.unwrap();
        sink.write("line2".to_string()).await.unwrap();
        sink.write("line3".to_string()).await.unwrap(); // Should trigger rotation

        sink.close().await.unwrap();
    }
}
