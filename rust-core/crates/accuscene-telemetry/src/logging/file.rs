//! File logging with rotation

use crate::{LoggingConfig, Result, TelemetryError};
use chrono::Utc;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// File logger with rotation support
pub struct FileLogger {
    config: LoggingConfig,
    current_file: Option<File>,
    current_size: u64,
}

impl FileLogger {
    /// Create a new file logger
    pub fn new(config: &LoggingConfig) -> Result<Self> {
        // Create log directory if it doesn't exist
        if !config.log_dir.exists() {
            fs::create_dir_all(&config.log_dir)?;
        }

        let mut logger = Self {
            config: config.clone(),
            current_file: None,
            current_size: 0,
        };

        // Open initial log file
        logger.rotate()?;

        Ok(logger)
    }

    /// Rotate the log file
    pub fn rotate(&mut self) -> Result<()> {
        // Close current file
        if let Some(mut file) = self.current_file.take() {
            file.flush()?;
        }

        // Generate new file name
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.log", self.config.file_prefix, timestamp);
        let filepath = self.config.log_dir.join(&filename);

        // Open new file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)
            .map_err(|e| TelemetryError::FileRotation(e.to_string()))?;

        self.current_file = Some(file);
        self.current_size = 0;

        // Clean up old files
        self.cleanup_old_files()?;

        tracing::info!("Rotated log file to: {:?}", filepath);
        Ok(())
    }

    /// Write a log entry
    pub fn write(&mut self, entry: &str) -> Result<()> {
        if let Some(file) = &mut self.current_file {
            let bytes = entry.as_bytes();
            file.write_all(bytes)?;
            file.write_all(b"\n")?;

            self.current_size += bytes.len() as u64 + 1;

            // Check if rotation is needed
            if self.config.rotation && self.current_size >= self.config.max_file_size_mb * 1024 * 1024 {
                self.rotate()?;
            }
        }

        Ok(())
    }

    /// Flush the current file
    pub fn flush(&mut self) {
        if let Some(file) = &mut self.current_file {
            let _ = file.flush();
        }
    }

    /// Clean up old log files
    fn cleanup_old_files(&self) -> Result<()> {
        let mut log_files = self.get_log_files()?;

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| {
            let a_meta = fs::metadata(a).ok();
            let b_meta = fs::metadata(b).ok();

            match (a_meta, b_meta) {
                (Some(a_m), Some(b_m)) => {
                    let a_time = a_m.modified().ok();
                    let b_time = b_m.modified().ok();
                    b_time.cmp(&a_time)
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Remove old files beyond the limit
        for file in log_files.iter().skip(self.config.max_files) {
            if let Err(e) = fs::remove_file(file) {
                tracing::warn!("Failed to remove old log file {:?}: {}", file, e);
            } else {
                tracing::debug!("Removed old log file: {:?}", file);
            }
        }

        Ok(())
    }

    /// Get all log files
    fn get_log_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !self.config.log_dir.exists() {
            return Ok(files);
        }

        for entry in fs::read_dir(&self.config.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.starts_with(&self.config.file_prefix) && filename_str.ends_with(".log") {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get current log file path
    pub fn current_file_path(&self) -> Option<PathBuf> {
        self.get_log_files().ok()?.into_iter().next()
    }

    /// Get total size of all log files
    pub fn total_size(&self) -> Result<u64> {
        let files = self.get_log_files()?;
        let mut total = 0;

        for file in files {
            if let Ok(metadata) = fs::metadata(&file) {
                total += metadata.len();
            }
        }

        Ok(total)
    }
}

impl Drop for FileLogger {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = LoggingConfig {
            log_dir: temp_dir.path().to_path_buf(),
            file_prefix: "test".to_string(),
            ..Default::default()
        };

        let logger = FileLogger::new(&config);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_write_and_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = LoggingConfig {
            log_dir: temp_dir.path().to_path_buf(),
            file_prefix: "test".to_string(),
            max_file_size_mb: 1, // 1 MB
            ..Default::default()
        };
        config.rotation = true;

        let mut logger = FileLogger::new(&config).unwrap();

        // Write some data
        logger.write("Test log entry").unwrap();
        logger.flush();

        // Check that file was created
        let files = logger.get_log_files().unwrap();
        assert!(!files.is_empty());
    }
}
