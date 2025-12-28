//! Structured audit logging
//!
//! Provides structured logging for audit events with async support.

use crate::audit::event::{AuditEvent, EventResult, EventSeverity, EventType, ResourceInfo};
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit logger
pub struct AuditLogger {
    handlers: Arc<RwLock<Vec<Box<dyn AuditHandler>>>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add an audit handler
    pub async fn add_handler(&self, handler: Box<dyn AuditHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler.handle(&event).await?;
        }
        Ok(())
    }

    /// Log authentication event
    pub async fn log_auth(
        &self,
        event_type: EventType,
        user_id: Option<String>,
        session_id: Option<String>,
        ip: Option<String>,
        result: EventResult,
        error: Option<String>,
    ) -> Result<()> {
        let mut event = AuditEvent::new(event_type, format!("{:?}", event_type))
            .with_result(result)
            .with_severity(EventSeverity::Info);

        if let Some(uid) = user_id {
            event = event.with_user(uid);
        }
        if let Some(sid) = session_id {
            event = event.with_session(sid);
        }
        if let Some(ip_addr) = ip {
            event = event.with_ip(ip_addr);
        }
        if let Some(err) = error {
            event = event.with_error(err);
        }

        self.log(event).await
    }

    /// Log authorization event
    pub async fn log_authz(
        &self,
        user_id: String,
        action: String,
        resource: Option<ResourceInfo>,
        granted: bool,
        reason: Option<String>,
    ) -> Result<()> {
        let event_type = if granted {
            EventType::AuthzAccessGranted
        } else {
            EventType::AuthzAccessDenied
        };

        let mut event = AuditEvent::new(event_type, action)
            .with_user(user_id)
            .with_result(if granted {
                EventResult::Success
            } else {
                EventResult::Failure
            })
            .with_severity(if granted {
                EventSeverity::Info
            } else {
                EventSeverity::Warning
            });

        if let Some(res) = resource {
            event = event.with_resource(res);
        }
        if let Some(r) = reason {
            event = event.add_metadata("reason".to_string(), r);
        }

        self.log(event).await
    }

    /// Log data access event
    pub async fn log_data_access(
        &self,
        user_id: String,
        resource: ResourceInfo,
        action: String,
        result: EventResult,
    ) -> Result<()> {
        let event = AuditEvent::new(EventType::DataRead, action)
            .with_user(user_id)
            .with_resource(resource)
            .with_result(result)
            .with_severity(EventSeverity::Info);

        self.log(event).await
    }

    /// Log security threat
    pub async fn log_security_threat(
        &self,
        threat_type: EventType,
        source: String,
        description: String,
    ) -> Result<()> {
        let event = AuditEvent::new(threat_type, description.clone())
            .with_severity(EventSeverity::Critical)
            .with_result(EventResult::Failure)
            .add_metadata("source".to_string(), source)
            .add_metadata("description".to_string(), description);

        self.log(event).await
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit event handler trait
#[async_trait::async_trait]
pub trait AuditHandler: Send + Sync {
    /// Handle an audit event
    async fn handle(&self, event: &AuditEvent) -> Result<()>;
}

/// Console audit handler (for development)
pub struct ConsoleHandler;

#[async_trait::async_trait]
impl AuditHandler for ConsoleHandler {
    async fn handle(&self, event: &AuditEvent) -> Result<()> {
        tracing::info!(
            event_id = %event.id,
            event_type = ?event.event_type,
            user_id = ?event.user_id,
            action = %event.action,
            result = ?event.result,
            "Audit event"
        );
        Ok(())
    }
}

/// File audit handler (writes to file)
pub struct FileHandler {
    path: std::path::PathBuf,
}

impl FileHandler {
    /// Create a new file handler
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait::async_trait]
impl AuditHandler for FileHandler {
    async fn handle(&self, event: &AuditEvent) -> Result<()> {
        let json = serde_json::to_string_pretty(event)
            .map_err(|e| crate::error::SecurityError::Internal(e.to_string()))?;

        tokio::fs::write(&self.path, format!("{}\n", json))
            .await
            .map_err(|e| crate::error::SecurityError::AuditLogFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let logger = AuditLogger::new();
        let event = AuditEvent::new(EventType::AuthLogin, "login".to_string());

        // Should not fail with no handlers
        assert!(logger.log(event).await.is_ok());
    }

    #[tokio::test]
    async fn test_console_handler() {
        let logger = AuditLogger::new();
        logger.add_handler(Box::new(ConsoleHandler)).await;

        let event = AuditEvent::new(EventType::AuthLogin, "login".to_string())
            .with_user("user123".to_string());

        assert!(logger.log(event).await.is_ok());
    }

    #[tokio::test]
    async fn test_log_auth_event() {
        let logger = AuditLogger::new();
        logger.add_handler(Box::new(ConsoleHandler)).await;

        assert!(logger
            .log_auth(
                EventType::AuthLogin,
                Some("user123".to_string()),
                Some("session123".to_string()),
                Some("192.168.1.1".to_string()),
                EventResult::Success,
                None,
            )
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_log_authz_event() {
        let logger = AuditLogger::new();
        logger.add_handler(Box::new(ConsoleHandler)).await;

        let resource = ResourceInfo::new("case", "case-123");

        assert!(logger
            .log_authz(
                "user123".to_string(),
                "read".to_string(),
                Some(resource),
                true,
                Some("User has permission".to_string()),
            )
            .await
            .is_ok());
    }
}
