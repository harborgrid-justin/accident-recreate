//! Tracing subscriber setup

use crate::{LogFormat, Result};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

/// Builder for creating tracing subscribers
pub struct SubscriberBuilder {
    level: String,
    format: LogFormat,
    console: bool,
    ansi: bool,
}

impl SubscriberBuilder {
    /// Create a new subscriber builder
    pub fn new() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            console: true,
            ansi: true,
        }
    }

    /// Set the log level
    pub fn with_level(mut self, level: &str) -> Self {
        self.level = level.to_string();
        self
    }

    /// Set the log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable or disable console output
    pub fn with_console(mut self, console: bool) -> Self {
        self.console = console;
        self
    }

    /// Enable or disable ANSI colors
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }

    /// Build the subscriber
    pub fn build(self) -> Result<impl tracing::Subscriber + Send + Sync> {
        // Create environment filter
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&self.level));

        // Build the subscriber based on format
        let subscriber = tracing_subscriber::registry().with(env_filter);

        match self.format {
            LogFormat::Json => {
                let layer = fmt::layer()
                    .json()
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_current_span(true)
                    .with_span_list(true)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true);

                Ok(subscriber.with(layer))
            }
            LogFormat::Text => {
                let layer = fmt::layer()
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(self.ansi);

                Ok(subscriber.with(layer))
            }
        }
    }
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}
