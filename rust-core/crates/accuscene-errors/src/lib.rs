//! AccuScene Enterprise Error Handling Infrastructure
//!
//! This crate provides comprehensive error handling, reporting, and recovery
//! capabilities for the entire AccuScene Enterprise system.
//!
//! # Features
//!
//! - **Unified Error Types**: Consistent error types across all crates
//! - **Error Codes**: Standardized error codes for API responses
//! - **Context Chaining**: Rich error context with cause chains
//! - **Error Reporting**: Formatted error messages with actionable guidance
//! - **Recovery Strategies**: Automatic and manual error recovery mechanisms
//! - **Diagnostics**: Detailed error diagnostics for debugging
//!
//! # Example
//!
//! ```rust
//! use accuscene_errors::{AccuSceneError, ErrorCode, ErrorContext, Result};
//!
//! fn process_data() -> Result<()> {
//!     // Operation that might fail
//!     let data = load_data()
//!         .context("Failed to load scene data")?;
//!
//!     Ok(())
//! }
//! ```

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]

pub mod codes;
pub mod context;
pub mod macros;
pub mod recovery;
pub mod reporting;
pub mod types;

// Re-export commonly used types and traits
pub use codes::{ErrorCode, ErrorSeverity};
pub use context::{ErrorContext, ErrorContextExt};
pub use recovery::{RecoveryAction, RecoveryStrategy, RetryPolicy};
pub use reporting::{ErrorReport, ErrorReporter};
pub use types::{AccuSceneError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::codes::{ErrorCode, ErrorSeverity};
    pub use crate::context::{ErrorContext, ErrorContextExt};
    pub use crate::recovery::{RecoveryAction, RecoveryStrategy, RetryPolicy};
    pub use crate::reporting::{ErrorReport, ErrorReporter};
    pub use crate::types::{AccuSceneError, Result};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AccuSceneError::validation("Invalid input");
        assert!(matches!(error.code(), ErrorCode::Validation));
    }

    #[test]
    fn test_error_context() {
        let error = AccuSceneError::internal("Database connection failed")
            .with_context("Failed to initialize database pool");

        assert!(error.context().is_some());
    }
}
