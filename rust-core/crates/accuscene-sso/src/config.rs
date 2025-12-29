//! SSO Configuration Module
//!
//! Centralized configuration for all SSO providers and authentication settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Main SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SSOConfig {
    /// Session configuration
    pub session: SessionConfig,

    /// MFA configuration
    pub mfa: MFAConfig,

    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,

    /// Redirect URIs
    pub redirect_uris: RedirectConfig,

    /// Security settings
    pub security: SecurityConfig,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SessionConfig {
    /// JWT secret for signing tokens
    #[validate(length(min = 32))]
    pub jwt_secret: String,

    /// Access token expiry in seconds (default: 900 = 15 minutes)
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry: i64,

    /// Refresh token expiry in seconds (default: 604800 = 7 days)
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry: i64,

    /// Session timeout in seconds (default: 86400 = 24 hours)
    #[serde(default = "default_session_timeout")]
    pub session_timeout: i64,

    /// Maximum concurrent sessions per user
    #[serde(default = "default_max_sessions")]
    pub max_sessions_per_user: usize,

    /// Token issuer
    #[serde(default = "default_issuer")]
    pub issuer: String,

    /// Audience
    #[serde(default = "default_audience")]
    pub audience: String,
}

fn default_access_token_expiry() -> i64 { 900 }
fn default_refresh_token_expiry() -> i64 { 604800 }
fn default_session_timeout() -> i64 { 86400 }
fn default_max_sessions() -> usize { 5 }
fn default_issuer() -> String { "accuscene-enterprise".to_string() }
fn default_audience() -> String { "accuscene-users".to_string() }

/// MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFAConfig {
    /// Enable TOTP
    #[serde(default)]
    pub totp_enabled: bool,

    /// TOTP issuer name
    #[serde(default = "default_totp_issuer")]
    pub totp_issuer: String,

    /// Enable WebAuthn/FIDO2
    #[serde(default)]
    pub webauthn_enabled: bool,

    /// WebAuthn relying party name
    #[serde(default = "default_rp_name")]
    pub webauthn_rp_name: String,

    /// WebAuthn relying party ID
    pub webauthn_rp_id: Option<String>,

    /// WebAuthn origin
    pub webauthn_origin: Option<String>,

    /// Require MFA for all users
    #[serde(default)]
    pub require_mfa: bool,

    /// MFA grace period in seconds
    #[serde(default = "default_mfa_grace_period")]
    pub mfa_grace_period: i64,
}

fn default_totp_issuer() -> String { "AccuScene Enterprise".to_string() }
fn default_rp_name() -> String { "AccuScene Enterprise".to_string() }
fn default_mfa_grace_period() -> i64 { 300 }

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ProviderConfig {
    /// SAML 2.0 configuration
    SAML {
        /// IdP entity ID
        entity_id: String,

        /// IdP SSO URL
        sso_url: String,

        /// IdP SLO URL
        slo_url: Option<String>,

        /// IdP certificate (PEM format)
        idp_certificate: String,

        /// SP entity ID
        sp_entity_id: String,

        /// SP ACS URL
        acs_url: String,

        /// SP private key (PEM format)
        sp_private_key: String,

        /// SP certificate (PEM format)
        sp_certificate: String,

        /// Sign authentication requests
        #[serde(default = "default_true")]
        sign_authn_request: bool,

        /// Want assertions signed
        #[serde(default = "default_true")]
        want_assertions_signed: bool,
    },

    /// OpenID Connect configuration
    OIDC {
        /// Client ID
        client_id: String,

        /// Client secret
        client_secret: String,

        /// Issuer URL
        issuer_url: String,

        /// Authorization endpoint
        auth_endpoint: Option<String>,

        /// Token endpoint
        token_endpoint: Option<String>,

        /// UserInfo endpoint
        userinfo_endpoint: Option<String>,

        /// JWKS URI
        jwks_uri: Option<String>,

        /// Scopes to request
        #[serde(default = "default_oidc_scopes")]
        scopes: Vec<String>,

        /// Use PKCE
        #[serde(default = "default_true")]
        use_pkce: bool,
    },

    /// OAuth 2.0 configuration
    OAuth2 {
        /// Client ID
        client_id: String,

        /// Client secret
        client_secret: String,

        /// Authorization endpoint
        auth_endpoint: String,

        /// Token endpoint
        token_endpoint: String,

        /// UserInfo endpoint
        userinfo_endpoint: String,

        /// Scopes to request
        scopes: Vec<String>,

        /// Use PKCE
        #[serde(default = "default_true")]
        use_pkce: bool,
    },

    /// LDAP/Active Directory configuration
    LDAP {
        /// LDAP server URL
        url: String,

        /// Bind DN
        bind_dn: String,

        /// Bind password
        bind_password: String,

        /// Base DN for user search
        user_base_dn: String,

        /// User search filter
        #[serde(default = "default_ldap_user_filter")]
        user_filter: String,

        /// User ID attribute
        #[serde(default = "default_ldap_uid_attr")]
        uid_attribute: String,

        /// Email attribute
        #[serde(default = "default_ldap_email_attr")]
        email_attribute: String,

        /// Name attribute
        #[serde(default = "default_ldap_name_attr")]
        name_attribute: String,

        /// Use TLS
        #[serde(default = "default_true")]
        use_tls: bool,

        /// TLS CA certificate
        tls_ca_cert: Option<String>,
    },
}

fn default_true() -> bool { true }
fn default_oidc_scopes() -> Vec<String> {
    vec!["openid".to_string(), "profile".to_string(), "email".to_string()]
}
fn default_ldap_user_filter() -> String { "(objectClass=person)".to_string() }
fn default_ldap_uid_attr() -> String { "uid".to_string() }
fn default_ldap_email_attr() -> String { "mail".to_string() }
fn default_ldap_name_attr() -> String { "cn".to_string() }

/// Redirect URI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectConfig {
    /// Base URL for redirects
    pub base_url: String,

    /// Callback path
    #[serde(default = "default_callback_path")]
    pub callback_path: String,

    /// Logout callback path
    #[serde(default = "default_logout_path")]
    pub logout_path: String,

    /// Error page path
    #[serde(default = "default_error_path")]
    pub error_path: String,
}

fn default_callback_path() -> String { "/auth/callback".to_string() }
fn default_logout_path() -> String { "/auth/logout".to_string() }
fn default_error_path() -> String { "/auth/error".to_string() }

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Allowed redirect domains
    pub allowed_redirect_domains: Vec<String>,

    /// Enable CSRF protection
    #[serde(default = "default_true")]
    pub csrf_protection: bool,

    /// State token expiry in seconds
    #[serde(default = "default_state_expiry")]
    pub state_token_expiry: i64,

    /// Maximum login attempts before lockout
    #[serde(default = "default_max_login_attempts")]
    pub max_login_attempts: u32,

    /// Lockout duration in seconds
    #[serde(default = "default_lockout_duration")]
    pub lockout_duration: i64,

    /// Enable audit logging
    #[serde(default = "default_true")]
    pub audit_logging: bool,
}

fn default_state_expiry() -> i64 { 600 }
fn default_max_login_attempts() -> u32 { 5 }
fn default_lockout_duration() -> i64 { 900 }

impl Default for SSOConfig {
    fn default() -> Self {
        Self {
            session: SessionConfig {
                jwt_secret: "change-this-secret-in-production-min-32-chars".to_string(),
                access_token_expiry: default_access_token_expiry(),
                refresh_token_expiry: default_refresh_token_expiry(),
                session_timeout: default_session_timeout(),
                max_sessions_per_user: default_max_sessions(),
                issuer: default_issuer(),
                audience: default_audience(),
            },
            mfa: MFAConfig {
                totp_enabled: true,
                totp_issuer: default_totp_issuer(),
                webauthn_enabled: true,
                webauthn_rp_name: default_rp_name(),
                webauthn_rp_id: None,
                webauthn_origin: None,
                require_mfa: false,
                mfa_grace_period: default_mfa_grace_period(),
            },
            providers: HashMap::new(),
            redirect_uris: RedirectConfig {
                base_url: "http://localhost:3000".to_string(),
                callback_path: default_callback_path(),
                logout_path: default_logout_path(),
                error_path: default_error_path(),
            },
            security: SecurityConfig {
                allowed_redirect_domains: vec!["localhost".to_string()],
                csrf_protection: true,
                state_token_expiry: default_state_expiry(),
                max_login_attempts: default_max_login_attempts(),
                lockout_duration: default_lockout_duration(),
                audit_logging: true,
            },
        }
    }
}

impl SSOConfig {
    /// Load configuration from environment
    pub fn from_env() -> Result<Self, String> {
        // In production, load from environment variables
        Ok(Self::default())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        self.session.validate()
            .map_err(|e| format!("Session config validation failed: {}", e))?;

        if self.providers.is_empty() {
            return Err("At least one SSO provider must be configured".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SSOConfig::default();
        assert!(config.session.access_token_expiry > 0);
        assert!(config.mfa.totp_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = SSOConfig::default();
        config.providers.insert(
            "test".to_string(),
            ProviderConfig::OIDC {
                client_id: "test-client".to_string(),
                client_secret: "test-secret".to_string(),
                issuer_url: "https://example.com".to_string(),
                auth_endpoint: None,
                token_endpoint: None,
                userinfo_endpoint: None,
                jwks_uri: None,
                scopes: default_oidc_scopes(),
                use_pkce: true,
            }
        );

        assert!(config.validate().is_ok());
    }
}
