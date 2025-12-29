//! Audit logging and analysis module
//!
//! Provides immutable audit logging with:
//! - Event logging
//! - Log analysis and alerting
//! - Tamper detection

pub mod logger;
pub mod events;
pub mod analyzer;

pub use logger::{AuditLogger, AuditLog};
pub use events::{AuditEvent, EventType, EventSeverity};
pub use analyzer::{AuditAnalyzer, AuditAlert};
