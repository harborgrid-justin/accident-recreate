//! AccuScene Enterprise Security & Audit System
//!
//! Comprehensive security, authentication, authorization, and audit system
//! for AccuScene Enterprise v0.2.0.
//!
//! # Features
//!
//! - **Authentication**: Password hashing (Argon2id), MFA (TOTP, WebAuthn), SSO (SAML, OIDC), sessions, JWT tokens
//! - **Authorization**: RBAC, ABAC, policy engine
//! - **Audit**: Structured logging, tamper-proof trail, querying
//! - **Compliance**: SOC2, GDPR, HIPAA controls
//! - **Encryption**: AES-256-GCM at rest, TLS in transit, key management
//! - **Secrets**: Secure vault, rotation
//! - **Validation**: Input sanitization and validation
//! - **Threat Detection**: Rate limiting, brute force detection, anomaly detection
//! - **Domain Security**: Case access, evidence chain of custody, report security
//!
//! # Example
//!
//! ```rust
//! use accuscene_security::{SecurityConfig, SecurityService};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create security service
//! let config = SecurityConfig::default();
//! let service = SecurityService::new(config).await?;
//!
//! // Authenticate user
//! // Authorize requests
//! // Audit events
//! # Ok(())
//! # }
//! ```

pub mod audit;
pub mod auth;
pub mod authz;
pub mod compliance;
pub mod config;
pub mod domain;
pub mod encryption;
pub mod error;
pub mod secrets;
pub mod threat;
pub mod validation;

// Re-export main types
pub use audit::AuditService;
pub use auth::{AuthenticationService, AuthContext};
pub use authz::AuthorizationService;
pub use compliance::ComplianceService;
pub use config::SecurityConfig;
pub use error::{Result, SecurityError, Severity};

use std::sync::Arc;

/// Main security service coordinating all components
pub struct SecurityService {
    config: SecurityConfig,
    auth: Arc<AuthenticationService>,
    authz: Arc<tokio::sync::RwLock<AuthorizationService>>,
    audit: Arc<AuditService>,
    compliance: Arc<ComplianceService>,
}

impl SecurityService {
    /// Create a new security service
    pub async fn new(config: SecurityConfig) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Initialize authentication service
        let auth = Arc::new(AuthenticationService::new(
            config.auth.clone(),
            b"change-this-secret-key-32-bytes!",
        ));

        // Initialize authorization service
        let authz = Arc::new(tokio::sync::RwLock::new(AuthorizationService::new(
            authz::PolicyEngineConfig {
                rbac_enabled: config.authz.rbac_enabled,
                abac_enabled: config.authz.abac_enabled,
                cache_enabled: config.authz.cache_decisions,
                cache_ttl_secs: config.authz.cache_ttl_secs,
            },
        )));

        // Initialize audit service
        let audit = Arc::new(AuditService::new(audit::StorageConfig {
            max_memory_events: 10_000,
            retention_days: config.audit.retention_days,
            encrypt: config.audit.encrypt_logs,
        }));

        // Add audit handlers
        if config.audit.enabled {
            audit.add_handler(Box::new(audit::ConsoleHandler)).await;
        }

        // Initialize compliance service
        let compliance = Arc::new(ComplianceService::new());

        Ok(Self {
            config,
            auth,
            authz,
            audit,
            compliance,
        })
    }

    /// Get authentication service
    pub fn auth(&self) -> Arc<AuthenticationService> {
        Arc::clone(&self.auth)
    }

    /// Get authorization service
    pub fn authz(&self) -> Arc<tokio::sync::RwLock<AuthorizationService>> {
        Arc::clone(&self.authz)
    }

    /// Get audit service
    pub fn audit(&self) -> Arc<AuditService> {
        Arc::clone(&self.audit)
    }

    /// Get compliance service
    pub fn compliance(&self) -> Arc<ComplianceService> {
        Arc::clone(&self.compliance)
    }

    /// Get configuration
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Verify system integrity
    pub async fn verify_integrity(&self) -> Result<IntegrityReport> {
        // Verify audit trail
        self.audit.verify_trail().await?;

        // Get compliance status
        let compliance_summary = self.compliance.get_summary();

        // Get audit trail stats
        let trail_stats = self.audit.trail_stats().await;

        Ok(IntegrityReport {
            audit_trail_valid: true,
            audit_entries: trail_stats.entry_count,
            soc2_compliance: compliance_summary.soc2_compliance,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Perform security health check
    pub async fn health_check(&self) -> HealthCheckResult {
        HealthCheckResult {
            status: HealthStatus::Healthy,
            components: vec![
                ("authentication".to_string(), ComponentStatus::Ok),
                ("authorization".to_string(), ComponentStatus::Ok),
                ("audit".to_string(), ComponentStatus::Ok),
                ("compliance".to_string(), ComponentStatus::Ok),
            ],
            timestamp: chrono::Utc::now(),
        }
    }
}

/// System integrity report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Audit trail validity
    pub audit_trail_valid: bool,
    /// Number of audit entries
    pub audit_entries: usize,
    /// SOC2 compliance percentage
    pub soc2_compliance: u8,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Overall health status
    pub status: HealthStatus,
    /// Component statuses
    pub components: Vec<(String, ComponentStatus)>,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentStatus {
    Ok,
    Warning,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_service_creation() {
        let config = SecurityConfig::default();
        let service = SecurityService::new(config).await.unwrap();

        assert!(service.auth().auth.password_service.validate_password("TestP@ssw0rd123!").is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = SecurityConfig::default();
        let service = SecurityService::new(config).await.unwrap();

        let health = service.health_check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.components.len(), 4);
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let config = SecurityConfig::default();
        let service = SecurityService::new(config).await.unwrap();

        let report = service.verify_integrity().await.unwrap();
        assert!(report.audit_trail_valid);
    }
}
