//! Job-specific error types for the AccuScene job processing system.

use thiserror::Error;

/// Result type alias for job operations
pub type Result<T> = std::result::Result<T, JobError>;

/// Comprehensive error types for job processing
#[derive(Error, Debug)]
pub enum JobError {
    /// Job execution failed
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    /// Job was cancelled
    #[error("Job was cancelled: {0}")]
    Cancelled(String),

    /// Job timeout
    #[error("Job timed out after {duration_secs} seconds")]
    Timeout { duration_secs: u64 },

    /// Job not found
    #[error("Job not found: {job_id}")]
    NotFound { job_id: String },

    /// Queue is full
    #[error("Queue is full (capacity: {capacity})")]
    QueueFull { capacity: usize },

    /// Queue operation failed
    #[error("Queue operation failed: {0}")]
    QueueError(String),

    /// Worker pool error
    #[error("Worker pool error: {0}")]
    WorkerPoolError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Database error (for persistent queue)
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Retry limit exceeded
    #[error("Retry limit exceeded (attempts: {attempts})")]
    RetryLimitExceeded { attempts: u32 },

    /// Scheduling error
    #[error("Scheduling error: {0}")]
    SchedulingError(String),

    /// Invalid cron expression
    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    /// Job already exists
    #[error("Job already exists: {job_id}")]
    AlreadyExists { job_id: String },

    /// Invalid job state transition
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Worker not available
    #[error("No workers available")]
    NoWorkersAvailable,

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit} jobs per {window_secs} seconds")]
    RateLimitExceeded { limit: u32, window_secs: u64 },

    /// Job dependency error
    #[error("Job dependency error: {0}")]
    DependencyError(String),

    /// Pipeline error
    #[error("Pipeline error: {0}")]
    PipelineError(String),

    /// Batch processing error
    #[error("Batch processing error: {0}")]
    BatchError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl JobError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            JobError::ExecutionFailed(_)
                | JobError::Timeout { .. }
                | JobError::WorkerPoolError(_)
                | JobError::NoWorkersAvailable
                | JobError::DatabaseError(_)
        )
    }

    /// Check if the error is permanent (not retryable)
    pub fn is_permanent(&self) -> bool {
        !self.is_retryable()
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            JobError::ExecutionFailed(_) => ErrorSeverity::High,
            JobError::Cancelled(_) => ErrorSeverity::Low,
            JobError::Timeout { .. } => ErrorSeverity::Medium,
            JobError::NotFound { .. } => ErrorSeverity::Low,
            JobError::QueueFull { .. } => ErrorSeverity::High,
            JobError::QueueError(_) => ErrorSeverity::High,
            JobError::WorkerPoolError(_) => ErrorSeverity::High,
            JobError::SerializationError(_) => ErrorSeverity::Medium,
            JobError::DatabaseError(_) => ErrorSeverity::Critical,
            JobError::InvalidConfiguration(_) => ErrorSeverity::Critical,
            JobError::RetryLimitExceeded { .. } => ErrorSeverity::High,
            JobError::SchedulingError(_) => ErrorSeverity::Medium,
            JobError::InvalidCronExpression(_) => ErrorSeverity::Medium,
            JobError::AlreadyExists { .. } => ErrorSeverity::Low,
            JobError::InvalidStateTransition { .. } => ErrorSeverity::Medium,
            JobError::NoWorkersAvailable => ErrorSeverity::High,
            JobError::RateLimitExceeded { .. } => ErrorSeverity::Low,
            JobError::DependencyError(_) => ErrorSeverity::High,
            JobError::PipelineError(_) => ErrorSeverity::High,
            JobError::BatchError(_) => ErrorSeverity::High,
            JobError::IoError(_) => ErrorSeverity::Medium,
            JobError::Internal(_) => ErrorSeverity::Critical,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryability() {
        let retryable = JobError::ExecutionFailed("test".to_string());
        assert!(retryable.is_retryable());

        let not_retryable = JobError::Cancelled("test".to_string());
        assert!(!not_retryable.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let critical = JobError::DatabaseError(rusqlite::Error::InvalidParameterCount(0, 1));
        assert_eq!(critical.severity(), ErrorSeverity::Critical);

        let low = JobError::Cancelled("test".to_string());
        assert_eq!(low.severity(), ErrorSeverity::Low);
    }
}
