//! Contextual logging with spans

use std::collections::HashMap;
use tracing::Span;

/// Log context for structured logging
#[derive(Debug, Clone)]
pub struct LogContext {
    fields: HashMap<String, serde_json::Value>,
}

impl LogContext {
    /// Create a new log context
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Add a field to the context
    pub fn add<V: serde::Serialize>(&mut self, key: impl Into<String>, value: V) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.fields.insert(key.into(), json_value);
        }
    }

    /// Remove a field from the context
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.fields.remove(key)
    }

    /// Get a field from the context
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        self.fields.clear();
    }

    /// Get all fields
    pub fn fields(&self) -> &HashMap<String, serde_json::Value> {
        &self.fields
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: &LogContext) {
        for (key, value) in &other.fields {
            self.fields.insert(key.clone(), value.clone());
        }
    }
}

impl Default for LogContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new span with the given name
pub fn span(name: &'static str) -> Span {
    tracing::info_span!(name)
}

/// Create a new span with fields
#[macro_export]
macro_rules! span_with_fields {
    ($name:expr, $($key:tt = $value:expr),* $(,)?) => {
        tracing::info_span!($name, $($key = $value),*)
    };
}

/// Log with context
#[macro_export]
macro_rules! log_with_context {
    ($level:ident, $context:expr, $($arg:tt)*) => {
        {
            let ctx = $context;
            tracing::$level!(
                fields = ?ctx.fields(),
                $($arg)*
            );
        }
    };
}

/// Enter a span and execute a closure
pub fn with_span<F, R>(span: Span, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = span.enter();
    f()
}

/// Async version of with_span
pub async fn with_span_async<F, R>(span: Span, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    async move {
        let _guard = span.enter();
        f.await
    }
    .await
}

/// Span builder for creating complex spans
pub struct SpanBuilder {
    name: String,
    fields: HashMap<String, String>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: HashMap::new(),
        }
    }

    /// Add a field to the span
    pub fn field(mut self, key: impl Into<String>, value: impl std::fmt::Display) -> Self {
        self.fields.insert(key.into(), value.to_string());
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        let span = tracing::info_span!(&self.name);
        for (key, value) in self.fields {
            span.record(&key, &value.as_str());
        }
        span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context() {
        let mut ctx = LogContext::new();
        ctx.add("user_id", 123);
        ctx.add("request_id", "abc-123");

        assert!(ctx.get("user_id").is_some());
        assert!(ctx.get("request_id").is_some());

        ctx.remove("user_id");
        assert!(ctx.get("user_id").is_none());
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = LogContext::new();
        ctx1.add("key1", "value1");

        let mut ctx2 = LogContext::new();
        ctx2.add("key2", "value2");

        ctx1.merge(&ctx2);

        assert!(ctx1.get("key1").is_some());
        assert!(ctx1.get("key2").is_some());
    }

    #[test]
    fn test_span_builder() {
        let _span = SpanBuilder::new("test_span")
            .field("user_id", 123)
            .field("operation", "test")
            .build();
    }
}
