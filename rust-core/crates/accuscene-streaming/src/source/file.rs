//! File-based streaming source with tailing support.

use crate::error::{Result, StreamingError};
use crate::source::Source;
use crate::stream::DataStream;
use async_trait::async_trait;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

/// File source that reads lines from a file
pub struct FileSource {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    running: bool,
    position: u64,
}

impl FileSource {
    /// Create a new file source
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            running: false,
            position: 0,
        }
    }
}

#[async_trait]
impl DataStream for FileSource {
    type Item = String;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        if self.reader.is_none() {
            return Ok(None);
        }

        let reader = self.reader.as_mut().unwrap();
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(n) => {
                self.position += n as u64;
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Some(line))
            }
            Err(e) => Err(StreamingError::Source(format!("File read error: {}", e))),
        }
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl Source for FileSource {
    async fn start(&mut self) -> Result<()> {
        let file = File::open(&self.path).await.map_err(|e| {
            StreamingError::Source(format!("Failed to open file: {}", e))
        })?;

        self.reader = Some(BufReader::new(file));
        self.running = true;

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.reader = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// File source with tail support (follows file as it grows)
pub struct TailFileSource {
    path: PathBuf,
    reader: Option<BufReader<File>>,
    running: bool,
    position: u64,
    poll_interval: std::time::Duration,
}

impl TailFileSource {
    /// Create a new tail file source
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            running: false,
            position: 0,
            poll_interval: std::time::Duration::from_millis(100),
        }
    }

    /// Set the poll interval for checking file changes
    pub fn with_poll_interval(mut self, interval: std::time::Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

#[async_trait]
impl DataStream for TailFileSource {
    type Item = String;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        if self.reader.is_none() {
            return Ok(None);
        }

        let reader = self.reader.as_mut().unwrap();
        let mut line = String::new();

        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF - wait and retry
                    tokio::time::sleep(self.poll_interval).await;
                    line.clear();

                    // Check if file was truncated
                    if let Ok(metadata) = tokio::fs::metadata(&self.path).await {
                        if metadata.len() < self.position {
                            // File was truncated, reopen
                            let file = File::open(&self.path).await.map_err(|e| {
                                StreamingError::Source(format!("Failed to reopen file: {}", e))
                            })?;
                            self.reader = Some(BufReader::new(file));
                            self.position = 0;
                        }
                    }

                    if !self.running {
                        return Ok(None);
                    }
                }
                Ok(n) => {
                    self.position += n as u64;
                    // Remove trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    return Ok(Some(line));
                }
                Err(e) => {
                    return Err(StreamingError::Source(format!("File read error: {}", e)))
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl Source for TailFileSource {
    async fn start(&mut self) -> Result<()> {
        let file = File::open(&self.path).await.map_err(|e| {
            StreamingError::Source(format!("Failed to open file: {}", e))
        })?;

        self.reader = Some(BufReader::new(file));
        self.running = true;

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.reader = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

/// File watcher source that monitors a directory for new files
pub struct FileWatcherSource {
    directory: PathBuf,
    pattern: Option<String>,
    receiver: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<PathBuf>>>,
    _watcher: Option<RecommendedWatcher>,
    running: bool,
}

impl FileWatcherSource {
    /// Create a new file watcher source
    pub fn new(directory: PathBuf, pattern: Option<String>) -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let tx = Arc::new(Mutex::new(tx));

        Ok(Self {
            directory,
            pattern,
            receiver: Arc::new(Mutex::new(rx)),
            _watcher: None,
            running: false,
        })
    }

    fn matches_pattern(&self, path: &PathBuf) -> bool {
        if let Some(pattern) = &self.pattern {
            if let Some(filename) = path.file_name() {
                return filename.to_string_lossy().contains(pattern);
            }
            false
        } else {
            true
        }
    }
}

#[async_trait]
impl DataStream for FileWatcherSource {
    type Item = PathBuf;

    async fn next(&mut self) -> Result<Option<Self::Item>> {
        if !self.running {
            return Ok(None);
        }

        let mut receiver = self.receiver.lock().await;
        Ok(receiver.recv().await)
    }

    fn is_complete(&self) -> bool {
        !self.running
    }
}

#[async_trait]
impl Source for FileWatcherSource {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_file_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();
        writeln!(temp_file, "line3").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let mut source = FileSource::new(path);

        source.start().await.unwrap();
        assert!(source.is_running());

        assert_eq!(source.next().await.unwrap(), Some("line1".to_string()));
        assert_eq!(source.next().await.unwrap(), Some("line2".to_string()));
        assert_eq!(source.next().await.unwrap(), Some("line3".to_string()));
        assert_eq!(source.next().await.unwrap(), None);

        source.stop().await.unwrap();
    }
}
