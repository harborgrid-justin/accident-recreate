//! Audit system
//!
//! Comprehensive audit logging with tamper detection and querying.

pub mod event;
pub mod logger;
pub mod query;
pub mod storage;
pub mod trail;

pub use event::{
    AuditEvent, EventResult, EventSeverity, EventType, ResourceInfo,
};
pub use logger::{AuditHandler, AuditLogger, ConsoleHandler, FileHandler};
pub use query::{AuditQuery, QueryResult, SortField, SortOrder};
pub use storage::{AuditStorage, FileStorage, StorageConfig};
pub use trail::{AuditTrail, MerkleTree, TrailEntry};

use crate::error::Result;
use std::sync::Arc;

/// Complete audit service combining all components
pub struct AuditService {
    logger: Arc<AuditLogger>,
    storage: Arc<AuditStorage>,
    trail: Arc<tokio::sync::RwLock<AuditTrail>>,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(storage_config: StorageConfig) -> Self {
        Self {
            logger: Arc::new(AuditLogger::new()),
            storage: Arc::new(AuditStorage::new(storage_config)),
            trail: Arc::new(tokio::sync::RwLock::new(AuditTrail::new())),
        }
    }

    /// Get the logger
    pub fn logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.logger)
    }

    /// Get the storage
    pub fn storage(&self) -> Arc<AuditStorage> {
        Arc::clone(&self.storage)
    }

    /// Add an audit handler
    pub async fn add_handler(&self, handler: Box<dyn AuditHandler>) {
        self.logger.add_handler(handler).await;
    }

    /// Log and store an audit event
    pub async fn audit(&self, event: AuditEvent) -> Result<()> {
        // Log the event
        self.logger.log(event.clone()).await?;

        // Store the event
        self.storage.store(event.clone()).await?;

        // Add to tamper-proof trail
        let mut trail = self.trail.write().await;
        trail.add_event(event)?;

        Ok(())
    }

    /// Query audit logs
    pub async fn query(&self, query: AuditQuery) -> Result<QueryResult> {
        let events = self.storage.get_all().await?;
        Ok(QueryResult::from_query(&query, &events))
    }

    /// Verify audit trail integrity
    pub async fn verify_trail(&self) -> Result<()> {
        let trail = self.trail.read().await;
        trail.verify_integrity()
    }

    /// Get audit trail statistics
    pub async fn trail_stats(&self) -> TrailStats {
        let trail = self.trail.read().await;
        TrailStats {
            entry_count: trail.len(),
            last_hash: trail.last_hash().map(|s| s.to_string()),
        }
    }

    /// Export audit trail
    pub async fn export_trail(&self) -> Result<Vec<u8>> {
        let trail = self.trail.read().await;
        trail.export()
    }

    /// Clean up expired audit logs
    pub async fn cleanup_expired(&self) -> Result<usize> {
        self.storage.cleanup_expired().await
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

/// Audit trail statistics
#[derive(Debug, Clone)]
pub struct TrailStats {
    /// Number of entries in trail
    pub entry_count: usize,
    /// Last hash in the chain
    pub last_hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_service_creation() {
        let service = AuditService::default();
        let stats = service.trail_stats().await;
        assert_eq!(stats.entry_count, 0);
    }

    #[tokio::test]
    async fn test_audit_event() {
        let service = AuditService::default();
        service.add_handler(Box::new(ConsoleHandler)).await;

        let event = AuditEvent::new(EventType::AuthLogin, "login".to_string())
            .with_user("user123".to_string());

        service.audit(event).await.unwrap();

        let stats = service.trail_stats().await;
        assert_eq!(stats.entry_count, 1);
    }

    #[tokio::test]
    async fn test_audit_query() {
        let service = AuditService::default();

        // Add multiple events
        for i in 0..5 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            )
            .with_user("user123".to_string());
            service.audit(event).await.unwrap();
        }

        // Query events
        let query = AuditQuery::new().user_id("user123").limit(3);
        let result = service.query(query).await.unwrap();

        assert_eq!(result.total_count, 5);
        assert_eq!(result.events.len(), 3);
    }

    #[tokio::test]
    async fn test_trail_verification() {
        let service = AuditService::default();

        // Add events
        for i in 0..3 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            service.audit(event).await.unwrap();
        }

        // Verify trail
        assert!(service.verify_trail().await.is_ok());
    }

    #[tokio::test]
    async fn test_trail_export() {
        let service = AuditService::default();

        // Add events
        for i in 0..3 {
            let event = AuditEvent::new(
                EventType::AuthLogin,
                format!("login-{}", i),
            );
            service.audit(event).await.unwrap();
        }

        // Export trail
        let exported = service.export_trail().await.unwrap();
        assert!(!exported.is_empty());
    }
}
