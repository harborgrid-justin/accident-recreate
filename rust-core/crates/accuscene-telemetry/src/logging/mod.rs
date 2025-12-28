//! Logging module for structured logging

pub mod subscriber;
pub mod format;
pub mod filter;
pub mod file;
pub mod context;

use crate::{LoggingConfig, Result};
use std::sync::Arc;
use parking_lot::RwLock;

pub use subscriber::SubscriberBuilder;
pub use format::{JsonFormatter, TextFormatter};
pub use filter::LevelFilter;
pub use file::FileLogger;
pub use context::{LogContext, span};

/// Logging system
pub struct LoggingSystem {
    config: LoggingConfig,
    file_logger: Option<Arc<RwLock<FileLogger>>>,
}

impl LoggingSystem {
    /// Create a new logging system
    pub fn new(config: &LoggingConfig) -> Result<Self> {
        let file_logger = if config.file_logging {
            Some(Arc::new(RwLock::new(FileLogger::new(config)?)))
        } else {
            None
        };

        Ok(Self {
            config: config.clone(),
            file_logger,
        })
    }

    /// Initialize the logging system
    pub fn init(&self) -> Result<()> {
        let subscriber = SubscriberBuilder::new()
            .with_level(&self.config.level)
            .with_format(self.config.format)
            .with_console(self.config.console)
            .with_ansi(self.config.ansi)
            .build()?;

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| crate::error::TelemetryError::logging_init(e.to_string()))?;

        tracing::info!("Logging system initialized");
        Ok(())
    }

    /// Get the file logger
    pub fn file_logger(&self) -> Option<Arc<RwLock<FileLogger>>> {
        self.file_logger.clone()
    }

    /// Rotate log files manually
    pub fn rotate(&self) -> Result<()> {
        if let Some(logger) = &self.file_logger {
            logger.write().rotate()?;
        }
        Ok(())
    }

    /// Flush all log buffers
    pub fn flush(&self) {
        if let Some(logger) = &self.file_logger {
            logger.write().flush();
        }
    }
}
