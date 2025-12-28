//! Log formatters for JSON and text output

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON log formatter
pub struct JsonFormatter;

impl JsonFormatter {
    /// Format a log record as JSON
    pub fn format(record: &LogRecord) -> String {
        serde_json::to_string(record).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format a log record as pretty JSON
    pub fn format_pretty(record: &LogRecord) -> String {
        serde_json::to_string_pretty(record).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Text log formatter
pub struct TextFormatter;

impl TextFormatter {
    /// Format a log record as text
    pub fn format(record: &LogRecord) -> String {
        let timestamp = record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level = &record.level;
        let target = &record.target;
        let message = &record.message;

        let mut output = format!("[{timestamp}] {level:5} {target}: {message}");

        if !record.fields.is_empty() {
            let fields_str = record
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            output.push_str(&format!(" | {}", fields_str));
        }

        if let Some(span) = &record.span {
            output.push_str(&format!(" [span: {}]", span));
        }

        output
    }
}

/// Log record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Log level
    pub level: String,

    /// Target (module path)
    pub target: String,

    /// Log message
    pub message: String,

    /// Additional fields
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,

    /// Span name (if in a span)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<String>,

    /// Thread name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<String>,

    /// File location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

impl LogRecord {
    /// Create a new log record
    pub fn new(level: impl Into<String>, target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level: level.into(),
            target: target.into(),
            message: message.into(),
            fields: HashMap::new(),
            span: None,
            thread: None,
            file: None,
            line: None,
        }
    }

    /// Add a field to the log record
    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// Set the span name
    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// Set the thread name
    pub fn with_thread(mut self, thread: impl Into<String>) -> Self {
        self.thread = Some(thread.into());
        self
    }

    /// Set the file location
    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_formatter() {
        let record = LogRecord::new("INFO", "test::module", "Test message")
            .with_field("key", serde_json::json!("value"));

        let json = JsonFormatter::format(&record);
        assert!(json.contains("Test message"));
        assert!(json.contains("INFO"));
    }

    #[test]
    fn test_text_formatter() {
        let record = LogRecord::new("INFO", "test::module", "Test message");
        let text = TextFormatter::format(&record);
        assert!(text.contains("Test message"));
        assert!(text.contains("INFO"));
        assert!(text.contains("test::module"));
    }
}
