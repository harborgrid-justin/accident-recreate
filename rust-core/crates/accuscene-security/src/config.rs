//! Security configuration
//!
//! Centralized security configuration for all components.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Authorization configuration
    pub authz: AuthzConfig,
    /// Audit configuration
    pub audit: AuditConfig,
    /// Encryption configuration
    pub encryption: EncryptionConfig,
    /// Threat detection configuration
    pub threat: ThreatConfig,
    /// Compliance configuration
    pub compliance: ComplianceConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            authz: AuthzConfig::default(),
            audit: AuditConfig::default(),
            encryption: EncryptionConfig::default(),
            threat: ThreatConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Password policy
    pub password_policy: PasswordPolicy,
    /// Session configuration
    pub session: SessionConfig,
    /// MFA configuration
    pub mfa: MfaConfig,
    /// SSO configuration
    pub sso: SsoConfig,
    /// JWT configuration
    pub jwt: JwtConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            password_policy: PasswordPolicy::default(),
            session: SessionConfig::default(),
            mfa: MfaConfig::default(),
            sso: SsoConfig::default(),
            jwt: JwtConfig::default(),
        }
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require digits
    pub require_digits: bool,
    /// Require special characters
    pub require_special: bool,
    /// Minimum password strength score (0-4)
    pub min_strength_score: u8,
    /// Password expiry in days (None = never expires)
    pub expiry_days: Option<u32>,
    /// Remember last N passwords
    pub password_history: usize,
    /// Maximum failed login attempts
    pub max_failed_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration_secs: u64,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
            min_strength_score: 3,
            expiry_days: Some(90),
            password_history: 5,
            max_failed_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_secs: u64,
    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Absolute timeout in seconds
    pub absolute_timeout_secs: u64,
    /// Enable concurrent session limits
    pub max_concurrent_sessions: Option<usize>,
    /// Require session renewal
    pub require_renewal: bool,
    /// Session renewal interval in seconds
    pub renewal_interval_secs: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 3600,           // 1 hour
            idle_timeout_secs: 1800,      // 30 minutes
            absolute_timeout_secs: 28800, // 8 hours
            max_concurrent_sessions: Some(3),
            require_renewal: true,
            renewal_interval_secs: 900, // 15 minutes
        }
    }
}

/// MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    /// Require MFA for all users
    pub required: bool,
    /// TOTP settings
    pub totp: TotpConfig,
    /// WebAuthn settings
    pub webauthn: WebAuthnConfig,
    /// Backup codes count
    pub backup_codes_count: usize,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            required: false,
            totp: TotpConfig::default(),
            webauthn: WebAuthnConfig::default(),
            backup_codes_count: 10,
        }
    }
}

/// TOTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    /// TOTP issuer name
    pub issuer: String,
    /// TOTP time step in seconds
    pub time_step: u64,
    /// TOTP digits
    pub digits: u32,
    /// TOTP skew (allow N steps before/after)
    pub skew: u8,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer: "AccuScene Enterprise".to_string(),
            time_step: 30,
            digits: 6,
            skew: 1,
        }
    }
}

/// WebAuthn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnConfig {
    /// Relying party name
    pub rp_name: String,
    /// Relying party ID
    pub rp_id: String,
    /// Origin URL
    pub origin: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for WebAuthnConfig {
    fn default() -> Self {
        Self {
            rp_name: "AccuScene Enterprise".to_string(),
            rp_id: "accuscene.com".to_string(),
            origin: "https://accuscene.com".to_string(),
            timeout_ms: 60000,
        }
    }
}

/// SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    /// Enable SAML SSO
    pub saml_enabled: bool,
    /// SAML entity ID
    pub saml_entity_id: Option<String>,
    /// Enable OIDC SSO
    pub oidc_enabled: bool,
    /// OIDC issuer URL
    pub oidc_issuer: Option<String>,
    /// OIDC client ID
    pub oidc_client_id: Option<String>,
}

impl Default for SsoConfig {
    fn default() -> Self {
        Self {
            saml_enabled: false,
            saml_entity_id: None,
            oidc_enabled: false,
            oidc_issuer: None,
            oidc_client_id: None,
        }
    }
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Token expiry in seconds
    pub expiry_secs: u64,
    /// Refresh token expiry in seconds
    pub refresh_expiry_secs: u64,
    /// Token issuer
    pub issuer: String,
    /// Token audience
    pub audience: String,
    /// Algorithm
    pub algorithm: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            expiry_secs: 3600,             // 1 hour
            refresh_expiry_secs: 2592000,  // 30 days
            issuer: "accuscene-enterprise".to_string(),
            audience: "accuscene-api".to_string(),
            algorithm: "HS256".to_string(),
        }
    }
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzConfig {
    /// Enable RBAC
    pub rbac_enabled: bool,
    /// Enable ABAC
    pub abac_enabled: bool,
    /// Cache authorization decisions
    pub cache_decisions: bool,
    /// Decision cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for AuthzConfig {
    fn default() -> Self {
        Self {
            rbac_enabled: true,
            abac_enabled: true,
            cache_decisions: true,
            cache_ttl_secs: 300, // 5 minutes
        }
    }
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log all API calls
    pub log_all_api_calls: bool,
    /// Log authentication events
    pub log_auth_events: bool,
    /// Log authorization decisions
    pub log_authz_decisions: bool,
    /// Log data access
    pub log_data_access: bool,
    /// Audit log retention in days
    pub retention_days: u32,
    /// Enable tamper detection
    pub tamper_detection: bool,
    /// Audit log encryption
    pub encrypt_logs: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_all_api_calls: false,
            log_auth_events: true,
            log_authz_decisions: true,
            log_data_access: true,
            retention_days: 365, // 1 year minimum for SOC2
            tamper_detection: true,
            encrypt_logs: true,
        }
    }
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption algorithm
    pub algorithm: String,
    /// Key rotation interval in days
    pub key_rotation_days: u32,
    /// Enable automatic key rotation
    pub auto_rotate_keys: bool,
    /// Key derivation iterations
    pub kdf_iterations: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: "AES-256-GCM".to_string(),
            key_rotation_days: 90,
            auto_rotate_keys: true,
            kdf_iterations: 100_000,
        }
    }
}

/// Threat detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatConfig {
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// Requests per minute
    pub rate_limit_rpm: u32,
    /// Enable brute force detection
    pub brute_force_detection: bool,
    /// Brute force threshold
    pub brute_force_threshold: u32,
    /// Brute force window in seconds
    pub brute_force_window_secs: u64,
    /// Enable anomaly detection
    pub anomaly_detection: bool,
    /// Anomaly detection sensitivity (0.0-1.0)
    pub anomaly_sensitivity: f64,
}

impl Default for ThreatConfig {
    fn default() -> Self {
        Self {
            rate_limiting_enabled: true,
            rate_limit_rpm: 100,
            brute_force_detection: true,
            brute_force_threshold: 5,
            brute_force_window_secs: 300, // 5 minutes
            anomaly_detection: true,
            anomaly_sensitivity: 0.8,
        }
    }
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable SOC2 controls
    pub soc2_enabled: bool,
    /// Enable GDPR controls
    pub gdpr_enabled: bool,
    /// Enable HIPAA controls
    pub hipaa_enabled: bool,
    /// Data retention policy in days
    pub data_retention_days: u32,
    /// Enable data deletion on request
    pub enable_data_deletion: bool,
    /// Enable data export on request
    pub enable_data_export: bool,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            soc2_enabled: true,
            gdpr_enabled: true,
            hipaa_enabled: false,
            data_retention_days: 2555, // 7 years for legal
            enable_data_deletion: true,
            enable_data_export: true,
        }
    }
}

impl SecurityConfig {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> Self {
        // In production, this would read from env vars
        Self::default()
    }

    /// Validate configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Validate password policy
        if self.auth.password_policy.min_length < 8 {
            return Err(crate::error::SecurityError::ConfigurationError(
                "Password minimum length must be at least 8".to_string(),
            ));
        }

        if self.auth.password_policy.min_strength_score > 4 {
            return Err(crate::error::SecurityError::ConfigurationError(
                "Password strength score must be 0-4".to_string(),
            ));
        }

        // Validate session timeouts
        if self.auth.session.idle_timeout_secs > self.auth.session.absolute_timeout_secs {
            return Err(crate::error::SecurityError::ConfigurationError(
                "Idle timeout cannot exceed absolute timeout".to_string(),
            ));
        }

        // Validate JWT expiry
        if self.auth.jwt.expiry_secs > self.auth.jwt.refresh_expiry_secs {
            return Err(crate::error::SecurityError::ConfigurationError(
                "JWT expiry cannot exceed refresh token expiry".to_string(),
            ));
        }

        Ok(())
    }

    /// Get session timeout as Duration
    pub fn session_timeout(&self) -> Duration {
        Duration::from_secs(self.auth.session.timeout_secs)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.auth.session.idle_timeout_secs)
    }

    /// Get JWT expiry as Duration
    pub fn jwt_expiry(&self) -> Duration {
        Duration::from_secs(self.auth.jwt.expiry_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validates() {
        let config = SecurityConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_password_length() {
        let mut config = SecurityConfig::default();
        config.auth.password_policy.min_length = 4;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_timeout_configuration() {
        let mut config = SecurityConfig::default();
        config.auth.session.idle_timeout_secs = 10000;
        config.auth.session.absolute_timeout_secs = 5000;
        assert!(config.validate().is_err());
    }
}
