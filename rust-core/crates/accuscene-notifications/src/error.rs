//! Error types for the notification system

use thiserror::Error;

/// Result type for notification operations
pub type Result<T> = std::result::Result<T, NotificationError>;

/// Comprehensive error types for the notification system
#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Dispatcher error: {0}")]
    Dispatcher(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Email delivery error: {0}")]
    EmailDelivery(String),

    #[error("SMS delivery error: {0}")]
    SmsDelivery(String),

    #[error("Push notification error: {0}")]
    PushNotification(String),

    #[error("Webhook delivery error: {0}")]
    WebhookDelivery(String),

    #[error("Invalid notification: {0}")]
    InvalidNotification(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Notification not found: {0}")]
    NotificationNotFound(String),

    #[error("Preference error: {0}")]
    Preference(String),

    #[error("Scheduling error: {0}")]
    Scheduling(String),

    #[error("Aggregation error: {0}")]
    Aggregation(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Queue full")]
    QueueFull,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl NotificationError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NotificationError::WebSocket(_)
                | NotificationError::EmailDelivery(_)
                | NotificationError::SmsDelivery(_)
                | NotificationError::PushNotification(_)
                | NotificationError::WebhookDelivery(_)
                | NotificationError::QueueFull
        )
    }

    /// Get retry delay in milliseconds
    pub fn retry_delay(&self) -> u64 {
        match self {
            NotificationError::RateLimitExceeded => 60000, // 1 minute
            NotificationError::QueueFull => 5000,          // 5 seconds
            _ => 1000,                                      // 1 second
        }
    }
}
