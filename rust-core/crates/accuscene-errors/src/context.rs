//! Error context and chaining for rich error information

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error context provides additional information about where and why an error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Contextual message
    message: String,

    /// Parent context (for chaining)
    parent: Option<Box<ErrorContext>>,

    /// Additional key-value data
    data: std::collections::HashMap<String, serde_json::Value>,

    /// Timestamp when context was added
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    /// Creates a new error context
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            parent: None,
            data: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Adds a parent context (for chaining)
    pub fn with_parent(mut self, parent: ErrorContext) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    /// Adds data to the context
    pub fn with_data(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.data.insert(key.into(), json_value);
        }
        self
    }

    /// Returns the context message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the parent context
    pub fn parent(&self) -> Option<&ErrorContext> {
        self.parent.as_deref()
    }

    /// Returns the context data
    pub fn data(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.data
    }

    /// Returns the timestamp
    pub fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    /// Returns the full context chain as a vector
    pub fn chain(&self) -> Vec<&ErrorContext> {
        let mut chain = vec![self];
        let mut current = self;
        while let Some(parent) = &current.parent {
            chain.push(parent);
            current = parent;
        }
        chain
    }

    /// Returns the depth of the context chain
    pub fn depth(&self) -> usize {
        self.chain().len()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if !self.data.is_empty() {
            write!(f, " (")?;
            let mut first = true;
            for (key, value) in &self.data {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}={}", key, value)?;
                first = false;
            }
            write!(f, ")")?;
        }

        if let Some(parent) = &self.parent {
            write!(f, " <- {}", parent)?;
        }

        Ok(())
    }
}

/// Extension trait for adding context to Results
pub trait ErrorContextExt<T, E> {
    /// Adds context to an error
    fn context(self, message: impl Into<String>) -> Result<T, crate::AccuSceneError>;

    /// Adds context with a closure (lazy evaluation)
    fn with_context<F>(self, f: F) -> Result<T, crate::AccuSceneError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContextExt<T, E> for Result<T, E>
where
    E: Into<crate::AccuSceneError>,
{
    fn context(self, message: impl Into<String>) -> Result<T, crate::AccuSceneError> {
        self.map_err(|e| {
            let mut error: crate::AccuSceneError = e.into();
            let ctx = ErrorContext::new(message);
            error = error.with_context(ctx.message);
            error
        })
    }

    fn with_context<F>(self, f: F) -> Result<T, crate::AccuSceneError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let mut error: crate::AccuSceneError = e.into();
            let ctx = ErrorContext::new(f());
            error = error.with_context(ctx.message);
            error
        })
    }
}

/// Builder for constructing error contexts
pub struct ErrorContextBuilder {
    context: ErrorContext,
}

impl ErrorContextBuilder {
    /// Creates a new context builder
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            context: ErrorContext::new(message),
        }
    }

    /// Adds data to the context
    pub fn with_data(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Self {
        self.context = self.context.with_data(key, value);
        self
    }

    /// Adds a parent context
    pub fn with_parent(mut self, parent: ErrorContext) -> Self {
        self.context = self.context.with_parent(parent);
        self
    }

    /// Builds the context
    pub fn build(self) -> ErrorContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ErrorContext::new("Test context");
        assert_eq!(ctx.message(), "Test context");
        assert!(ctx.parent().is_none());
    }

    #[test]
    fn test_context_chaining() {
        let parent = ErrorContext::new("Parent context");
        let child = ErrorContext::new("Child context").with_parent(parent);

        assert_eq!(child.depth(), 2);
        assert!(child.parent().is_some());
    }

    #[test]
    fn test_context_with_data() {
        let ctx = ErrorContext::new("Test")
            .with_data("key1", "value1")
            .with_data("key2", 42);

        assert_eq!(ctx.data().len(), 2);
    }

    #[test]
    fn test_context_builder() {
        let ctx = ErrorContextBuilder::new("Test message")
            .with_data("user_id", "12345")
            .with_data("action", "delete")
            .build();

        assert_eq!(ctx.message(), "Test message");
        assert_eq!(ctx.data().len(), 2);
    }
}
