//! Error types for the AccuScene dashboard system
//!
//! Provides comprehensive error handling with context and traceability

use std::fmt;
use thiserror::Error;

/// Main error type for dashboard operations
#[derive(Error, Debug)]
pub enum DashboardError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Layout error
    #[error("Layout error: {0}")]
    Layout(String),

    /// Widget error
    #[error("Widget error: {context}: {source}")]
    Widget {
        context: String,
        source: WidgetError,
    },

    /// State management error
    #[error("State error: {0}")]
    State(String),

    /// Persistence error
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Concurrent modification error
    #[error("Concurrent modification detected")]
    ConcurrentModification,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Widget-specific errors
#[derive(Error, Debug)]
pub enum WidgetError {
    /// Invalid widget type
    #[error("Invalid widget type: {0}")]
    InvalidType(String),

    /// Invalid widget configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Widget data error
    #[error("Data error: {0}")]
    DataError(String),

    /// Widget rendering error
    #[error("Rendering error: {0}")]
    RenderError(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Persistence layer errors
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Version conflict
    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict {
        expected: u64,
        actual: u64,
    },

    /// Storage quota exceeded
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Corruption detected
    #[error("Data corruption detected: {0}")]
    Corruption(String),
}

/// Layout errors
#[derive(Error, Debug)]
pub enum LayoutError {
    /// Invalid breakpoint
    #[error("Invalid breakpoint: {0}")]
    InvalidBreakpoint(String),

    /// Invalid grid configuration
    #[error("Invalid grid configuration: {0}")]
    InvalidGrid(String),

    /// Widget position conflict
    #[error("Widget position conflict at ({x}, {y})")]
    PositionConflict { x: u32, y: u32 },

    /// Out of bounds
    #[error("Position out of bounds: ({x}, {y}) exceeds grid ({max_x}, {max_y})")]
    OutOfBounds {
        x: u32,
        y: u32,
        max_x: u32,
        max_y: u32,
    },
}

/// Result type alias for dashboard operations
pub type DashboardResult<T> = Result<T, DashboardError>;

/// Result type alias for widget operations
pub type WidgetResult<T> = Result<T, WidgetError>;

/// Result type alias for layout operations
pub type LayoutResult<T> = Result<T, LayoutError>;

/// Result type alias for persistence operations
pub type PersistenceResult<T> = Result<T, PersistenceError>;

impl DashboardError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        DashboardError::Configuration(msg.into())
    }

    /// Create a layout error
    pub fn layout<S: Into<String>>(msg: S) -> Self {
        DashboardError::Layout(msg.into())
    }

    /// Create a state error
    pub fn state<S: Into<String>>(msg: S) -> Self {
        DashboardError::State(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        DashboardError::Validation(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        DashboardError::NotFound(resource.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        DashboardError::Internal(msg.into())
    }

    /// Wrap a widget error with context
    pub fn widget_error<S: Into<String>>(context: S, source: WidgetError) -> Self {
        DashboardError::Widget {
            context: context.into(),
            source,
        }
    }
}

impl From<LayoutError> for DashboardError {
    fn from(err: LayoutError) -> Self {
        DashboardError::Layout(err.to_string())
    }
}

impl From<WidgetError> for DashboardError {
    fn from(err: WidgetError) -> Self {
        DashboardError::Widget {
            context: "Widget operation failed".to_string(),
            source: err,
        }
    }
}

impl WidgetError {
    /// Create an invalid type error
    pub fn invalid_type<S: Into<String>>(type_name: S) -> Self {
        WidgetError::InvalidType(type_name.into())
    }

    /// Create an invalid configuration error
    pub fn invalid_config<S: Into<String>>(msg: S) -> Self {
        WidgetError::InvalidConfiguration(msg.into())
    }

    /// Create a data error
    pub fn data_error<S: Into<String>>(msg: S) -> Self {
        WidgetError::DataError(msg.into())
    }

    /// Create a render error
    pub fn render_error<S: Into<String>>(msg: S) -> Self {
        WidgetError::RenderError(msg.into())
    }

    /// Create a missing field error
    pub fn missing_field<S: Into<String>>(field: S) -> Self {
        WidgetError::MissingField(field.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = DashboardError::config("test config error");
        assert!(matches!(err, DashboardError::Configuration(_)));

        let err = WidgetError::invalid_type("UnknownWidget");
        assert!(matches!(err, WidgetError::InvalidType(_)));
    }

    #[test]
    fn test_error_display() {
        let err = DashboardError::not_found("widget-123");
        assert_eq!(err.to_string(), "Not found: widget-123");

        let widget_err = WidgetError::data_error("Invalid metric value");
        let err = DashboardError::widget_error("Metrics widget", widget_err);
        assert!(err.to_string().contains("Metrics widget"));
    }
}
