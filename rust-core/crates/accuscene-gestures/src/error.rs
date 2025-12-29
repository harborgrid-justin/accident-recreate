use thiserror::Error;

/// Gesture recognition errors
#[derive(Error, Debug, Clone)]
pub enum GestureError {
    #[error("Invalid touch point: {0}")]
    InvalidTouchPoint(String),

    #[error("Insufficient touch points: expected {expected}, got {actual}")]
    InsufficientTouchPoints { expected: usize, actual: usize },

    #[error("Gesture timeout: {gesture_type}")]
    GestureTimeout { gesture_type: String },

    #[error("Conflicting gestures detected: {0}")]
    ConflictingGestures(String),

    #[error("Invalid gesture state transition: from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Gesture recognition failed: {0}")]
    RecognitionFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Touch tracking error: {0}")]
    TouchTrackingError(String),

    #[error("Velocity calculation error: {0}")]
    VelocityCalculationError(String),

    #[error("Custom gesture pattern not found: {0}")]
    CustomPatternNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type GestureResult<T> = Result<T, GestureError>;
