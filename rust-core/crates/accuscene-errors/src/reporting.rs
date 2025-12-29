//! Error reporting and formatting utilities

use crate::{AccuSceneError, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// Error report containing formatted error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Error ID for tracking
    pub id: String,

    /// Error code
    pub code: String,

    /// Error severity
    pub severity: String,

    /// Main error message
    pub message: String,

    /// Detailed description
    pub details: Option<String>,

    /// Context chain
    pub context: Vec<String>,

    /// Timestamp
    pub timestamp: String,

    /// Source location
    pub location: Option<String>,

    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Suggested actions for resolution
    pub suggested_actions: Vec<String>,

    /// Whether error is recoverable
    pub recoverable: bool,
}

/// Error reporter for formatting and presenting errors
pub struct ErrorReporter;

impl ErrorReporter {
    /// Formats an error as a human-readable string
    pub fn format_error(error: &AccuSceneError) -> String {
        let mut output = String::new();

        // Header
        writeln!(
            &mut output,
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        )
        .ok();
        writeln!(
            &mut output,
            "Error [{}] - {} - {}",
            error.id(),
            error.code(),
            error.severity()
        )
        .ok();
        writeln!(
            &mut output,
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        )
        .ok();

        // Message
        writeln!(&mut output).ok();
        writeln!(&mut output, "Message: {}", error.message()).ok();

        // Details
        if let Some(details) = error.details() {
            writeln!(&mut output).ok();
            writeln!(&mut output, "Details:").ok();
            writeln!(&mut output, "  {}", details).ok();
        }

        // Context chain
        if let Some(ctx) = error.context() {
            writeln!(&mut output).ok();
            writeln!(&mut output, "Context:").ok();
            for (i, context) in ctx.chain().iter().enumerate() {
                writeln!(&mut output, "  {}. {}", i + 1, context.message()).ok();
            }
        }

        // Location
        if let Some(location) = error.location() {
            writeln!(&mut output).ok();
            writeln!(&mut output, "Location: {}", location).ok();
        }

        // Metadata
        if !error.metadata().is_empty() {
            writeln!(&mut output).ok();
            writeln!(&mut output, "Metadata:").ok();
            for (key, value) in error.metadata() {
                writeln!(&mut output, "  {}: {}", key, value).ok();
            }
        }

        // Timestamp
        writeln!(&mut output).ok();
        writeln!(
            &mut output,
            "Timestamp: {}",
            error.timestamp().to_rfc3339()
        )
        .ok();

        // Suggested actions
        let actions = Self::suggest_actions(error);
        if !actions.is_empty() {
            writeln!(&mut output).ok();
            writeln!(&mut output, "Suggested Actions:").ok();
            for (i, action) in actions.iter().enumerate() {
                writeln!(&mut output, "  {}. {}", i + 1, action).ok();
            }
        }

        // Recoverable status
        writeln!(&mut output).ok();
        writeln!(
            &mut output,
            "Recoverable: {}",
            if error.is_recoverable() { "Yes" } else { "No" }
        )
        .ok();

        writeln!(
            &mut output,
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        )
        .ok();

        output
    }

    /// Creates an error report from an error
    pub fn create_report(error: &AccuSceneError) -> ErrorReport {
        let context_chain = error
            .context()
            .map(|ctx| {
                ctx.chain()
                    .iter()
                    .map(|c| c.message().to_string())
                    .collect()
            })
            .unwrap_or_default();

        ErrorReport {
            id: error.id().to_string(),
            code: error.code().to_string(),
            severity: error.severity().to_string(),
            message: error.message().to_string(),
            details: error.details().map(|s| s.to_string()),
            context: context_chain,
            timestamp: error.timestamp().to_rfc3339(),
            location: error.location().map(|s| s.to_string()),
            metadata: error.metadata().clone(),
            suggested_actions: Self::suggest_actions(error),
            recoverable: error.is_recoverable(),
        }
    }

    /// Suggests actions for resolving the error
    pub fn suggest_actions(error: &AccuSceneError) -> Vec<String> {
        use crate::ErrorCode;

        let mut actions = Vec::new();

        match error.code() {
            ErrorCode::Validation => {
                actions.push("Check input parameters and ensure they meet validation requirements".to_string());
                actions.push("Review API documentation for correct request format".to_string());
            }
            ErrorCode::Authentication => {
                actions.push("Verify authentication credentials are correct".to_string());
                actions.push("Check if authentication token has expired".to_string());
                actions.push("Ensure proper authentication headers are included".to_string());
            }
            ErrorCode::Authorization => {
                actions.push("Verify you have the necessary permissions for this operation".to_string());
                actions.push("Contact your administrator to request access".to_string());
            }
            ErrorCode::NotFound => {
                actions.push("Verify the resource ID or path is correct".to_string());
                actions.push("Check if the resource has been deleted".to_string());
            }
            ErrorCode::Network => {
                actions.push("Check your network connection".to_string());
                actions.push("Verify the service endpoint is accessible".to_string());
                actions.push("Retry the operation".to_string());
            }
            ErrorCode::Database => {
                actions.push("Check database connection settings".to_string());
                actions.push("Verify database is running and accessible".to_string());
                actions.push("Review database logs for more details".to_string());
            }
            ErrorCode::Timeout => {
                actions.push("Retry the operation".to_string());
                actions.push("Increase timeout duration if possible".to_string());
                actions.push("Check system load and performance".to_string());
            }
            ErrorCode::RateLimit => {
                actions.push("Wait before retrying the operation".to_string());
                actions.push("Implement exponential backoff".to_string());
                actions.push("Review rate limit quotas".to_string());
            }
            ErrorCode::Internal => {
                actions.push("Report this error to the development team".to_string());
                actions.push("Include the error ID in your report".to_string());
                actions.push("Check system logs for more details".to_string());
            }
            _ => {
                actions.push("Review error details and context".to_string());
                actions.push("Check application logs for more information".to_string());
            }
        }

        actions
    }

    /// Formats error for logging
    pub fn format_for_log(error: &AccuSceneError) -> String {
        format!(
            "[{}] {} - {}: {} (severity: {}, recoverable: {})",
            error.id(),
            error.code(),
            error.message(),
            error
                .details()
                .unwrap_or("no details"),
            error.severity(),
            error.is_recoverable()
        )
    }

    /// Formats error for JSON API response
    pub fn format_for_api(error: &AccuSceneError) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "id": error.id(),
                "code": error.code().to_string(),
                "message": error.message(),
                "details": error.details(),
                "severity": error.severity().to_string(),
                "timestamp": error.timestamp().to_rfc3339(),
                "recoverable": error.is_recoverable(),
            }
        })
    }

    /// Returns a user-friendly message based on severity
    pub fn user_friendly_message(error: &AccuSceneError) -> String {
        match error.severity() {
            ErrorSeverity::Critical => {
                format!(
                    "A critical error occurred: {}. Please contact support immediately.",
                    error.message()
                )
            }
            ErrorSeverity::High => {
                format!(
                    "An error occurred: {}. Please try again or contact support if the problem persists.",
                    error.message()
                )
            }
            ErrorSeverity::Medium => {
                format!(
                    "{}. Please try again.",
                    error.message()
                )
            }
            ErrorSeverity::Low | ErrorSeverity::Warning => {
                error.message().to_string()
            }
            ErrorSeverity::Info => {
                format!("Note: {}", error.message())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_error() {
        let error = AccuSceneError::validation("Test error")
            .with_details("Additional details");

        let formatted = ErrorReporter::format_error(&error);
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("Additional details"));
    }

    #[test]
    fn test_create_report() {
        let error = AccuSceneError::database("Connection failed");
        let report = ErrorReporter::create_report(&error);

        assert_eq!(report.message, "Connection failed");
        assert!(!report.suggested_actions.is_empty());
    }

    #[test]
    fn test_suggest_actions() {
        let error = AccuSceneError::authentication("Invalid credentials");
        let actions = ErrorReporter::suggest_actions(&error);

        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.contains("credentials")));
    }
}
