//! Security configuration

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Global security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication configuration
    pub auth: AuthConfig,

    /// JWT configuration
    pub jwt: JwtConfig,

    /// Session configuration
    pub session: SessionConfig,

    /// Password policy
    pub password_policy: PasswordPolicy,

    /// MFA configuration
    pub mfa: MfaConfig,

    /// Audit configuration
    pub audit: AuditConfig,

    /// Compliance configuration
    pub compliance: ComplianceConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,

    /// Lockout duration in seconds
    pub lockout_duration_secs: i64,

    /// Enable rate limiting
    pub enable_rate_limiting: bool,

    /// Requests per minute per IP
    pub rate_limit_per_minute: u32,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key (should be loaded from secure storage)
    #[serde(skip_serializing)]
    pub secret: String,

    /// JWT issuer
    pub issuer: String,

    /// JWT audience
    pub audience: String,

    /// Access token expiration in seconds
    pub access_token_expiry_secs: i64,

    /// Refresh token expiration in seconds
    pub refresh_token_expiry_secs: i64,

    /// Algorithm (HS256, HS384, HS512, RS256, RS384, RS512, ES256, ES384)
    pub algorithm: String,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_secs: i64,

    /// Idle timeout in seconds
    pub idle_timeout_secs: i64,

    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: u32,

    /// Enable session fingerprinting
    pub enable_fingerprinting: bool,
}

/// Password policy
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
    pub require_special_chars: bool,

    /// Password expiration in days (0 = never expires)
    pub expiration_days: u32,

    /// Number of previous passwords to check against
    pub password_history: u32,

    /// Argon2 memory cost in KiB
    pub argon2_memory_cost: u32,

    /// Argon2 time cost (iterations)
    pub argon2_time_cost: u32,

    /// Argon2 parallelism
    pub argon2_parallelism: u32,
}

/// MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    /// Require MFA for all users
    pub require_mfa: bool,

    /// Require MFA for admins only
    pub require_mfa_for_admins: bool,

    /// TOTP window (number of time steps to check)
    pub totp_window: u8,

    /// TOTP step in seconds
    pub totp_step_secs: u64,

    /// Enable WebAuthn
    pub enable_webauthn: bool,

    /// WebAuthn relying party name
    pub webauthn_rp_name: String,

    /// WebAuthn relying party ID
    pub webauthn_rp_id: String,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enable_audit_log: bool,

    /// Log successful authentications
    pub log_successful_auth: bool,

    /// Log failed authentications
    pub log_failed_auth: bool,

    /// Log authorization decisions
    pub log_authz_decisions: bool,

    /// Log data access
    pub log_data_access: bool,

    /// Log configuration changes
    pub log_config_changes: bool,

    /// Audit log retention in days
    pub retention_days: u32,

    /// Enable real-time alerting
    pub enable_alerting: bool,

    /// Alert threshold for failed logins
    pub failed_login_alert_threshold: u32,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable GDPR compliance features
    pub enable_gdpr: bool,

    /// Default data retention period in days
    pub default_retention_days: u32,

    /// Enable data minimization
    pub enable_data_minimization: bool,

    /// Enable right to be forgotten
    pub enable_right_to_be_forgotten: bool,

    /// Enable data portability
    pub enable_data_portability: bool,

    /// Require consent for data processing
    pub require_consent: bool,

    /// Enable SOC2 compliance features
    pub enable_soc2: bool,

    /// Enable ISO27001 compliance features
    pub enable_iso27001: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            jwt: JwtConfig::default(),
            session: SessionConfig::default(),
            password_policy: PasswordPolicy::default(),
            mfa: MfaConfig::default(),
            audit: AuditConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            max_login_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
            enable_rate_limiting: true,
            rate_limit_per_minute: 60,
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(), // Must be set by user
            issuer: "accuscene".to_string(),
            audience: "accuscene-api".to_string(),
            access_token_expiry_secs: 900, // 15 minutes
            refresh_token_expiry_secs: 604800, // 7 days
            algorithm: "HS256".to_string(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 3600, // 1 hour
            idle_timeout_secs: 1800, // 30 minutes
            max_concurrent_sessions: 5,
            enable_fingerprinting: true,
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special_chars: true,
            expiration_days: 90,
            password_history: 5,
            argon2_memory_cost: 65536, // 64 MiB
            argon2_time_cost: 3,
            argon2_parallelism: 4,
        }
    }
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            require_mfa: false,
            require_mfa_for_admins: true,
            totp_window: 1,
            totp_step_secs: 30,
            enable_webauthn: true,
            webauthn_rp_name: "AccuScene".to_string(),
            webauthn_rp_id: "accuscene.com".to_string(),
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enable_audit_log: true,
            log_successful_auth: true,
            log_failed_auth: true,
            log_authz_decisions: true,
            log_data_access: true,
            log_config_changes: true,
            retention_days: 365,
            enable_alerting: true,
            failed_login_alert_threshold: 10,
        }
    }
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            enable_gdpr: true,
            default_retention_days: 2555, // 7 years
            enable_data_minimization: true,
            enable_right_to_be_forgotten: true,
            enable_data_portability: true,
            require_consent: true,
            enable_soc2: true,
            enable_iso27001: true,
        }
    }
}

impl PasswordPolicy {
    /// Validate a password against the policy
    pub fn validate(&self, password: &str) -> Result<(), String> {
        if password.len() < self.min_length {
            return Err(format!(
                "Password must be at least {} characters long",
                self.min_length
            ));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err("Password must contain at least one lowercase letter".to_string());
        }

        if self.require_digits && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err("Password must contain at least one digit".to_string());
        }

        if self.require_special_chars
            && !password.chars().any(|c| !c.is_alphanumeric())
        {
            return Err("Password must contain at least one special character".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy::default();

        assert!(policy.validate("Weak").is_err());
        assert!(policy.validate("NoUpperCase123!").is_err());
        assert!(policy.validate("NOLOWERCASE123!").is_err());
        assert!(policy.validate("NoDigits!@#").is_err());
        assert!(policy.validate("NoSpecial123").is_err());
        assert!(policy.validate("ValidPassword123!").is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert_eq!(config.auth.max_login_attempts, 5);
        assert_eq!(config.jwt.access_token_expiry_secs, 900);
        assert_eq!(config.password_policy.min_length, 12);
        assert!(config.compliance.enable_gdpr);
    }
}
