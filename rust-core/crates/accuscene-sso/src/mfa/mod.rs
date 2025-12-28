//! Multi-Factor Authentication Module

pub mod totp;
pub mod webauthn;

use crate::{SSOError, SSOResult};
use serde::{Deserialize, Serialize};

pub use totp::{TOTPManager, TOTPSecret};
pub use webauthn::{WebAuthnManager, CredentialRegistration, AuthenticationChallenge};

/// MFA method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MFAMethod {
    /// Time-based One-Time Password
    TOTP,
    /// WebAuthn/FIDO2
    WebAuthn,
    /// SMS (future support)
    SMS,
    /// Email (future support)
    Email,
}

/// MFA verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFAVerification {
    /// Verification successful
    pub verified: bool,
    /// Method used
    pub method: MFAMethod,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Device identifier (for WebAuthn)
    pub device_id: Option<String>,
}

/// User MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMFAConfig {
    /// User ID
    pub user_id: String,
    /// Enabled methods
    pub enabled_methods: Vec<MFAMethod>,
    /// TOTP secret (if TOTP enabled)
    pub totp_secret: Option<String>,
    /// WebAuthn credentials (if WebAuthn enabled)
    pub webauthn_credentials: Vec<WebAuthnCredential>,
    /// Backup codes
    pub backup_codes: Vec<String>,
    /// MFA enabled
    pub mfa_enabled: bool,
}

/// WebAuthn credential info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    /// Credential ID
    pub id: String,
    /// Credential name/label
    pub name: String,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl UserMFAConfig {
    /// Create new MFA config for user
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            enabled_methods: Vec::new(),
            totp_secret: None,
            webauthn_credentials: Vec::new(),
            backup_codes: Vec::new(),
            mfa_enabled: false,
        }
    }

    /// Check if MFA is enabled
    pub fn is_enabled(&self) -> bool {
        self.mfa_enabled && !self.enabled_methods.is_empty()
    }

    /// Check if specific method is enabled
    pub fn has_method(&self, method: MFAMethod) -> bool {
        self.enabled_methods.contains(&method)
    }

    /// Add MFA method
    pub fn add_method(&mut self, method: MFAMethod) {
        if !self.enabled_methods.contains(&method) {
            self.enabled_methods.push(method);
        }
    }

    /// Remove MFA method
    pub fn remove_method(&mut self, method: MFAMethod) {
        self.enabled_methods.retain(|m| m != &method);

        // Clean up associated data
        match method {
            MFAMethod::TOTP => self.totp_secret = None,
            MFAMethod::WebAuthn => self.webauthn_credentials.clear(),
            _ => {}
        }

        // Disable MFA if no methods left
        if self.enabled_methods.is_empty() {
            self.mfa_enabled = false;
        }
    }

    /// Generate backup codes
    pub fn generate_backup_codes(&mut self, count: usize) -> Vec<String> {
        use rand::Rng;

        self.backup_codes = (0..count)
            .map(|_| {
                let code: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(12)
                    .map(char::from)
                    .collect();
                code.to_uppercase()
            })
            .collect();

        self.backup_codes.clone()
    }

    /// Verify backup code
    pub fn verify_backup_code(&mut self, code: &str) -> bool {
        let code_upper = code.to_uppercase();
        if let Some(index) = self.backup_codes.iter().position(|c| c == &code_upper) {
            // Remove used backup code
            self.backup_codes.remove(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_mfa_config() {
        let mut config = UserMFAConfig::new("user123".to_string());
        assert!(!config.is_enabled());

        config.add_method(MFAMethod::TOTP);
        config.mfa_enabled = true;
        assert!(config.is_enabled());
        assert!(config.has_method(MFAMethod::TOTP));
        assert!(!config.has_method(MFAMethod::WebAuthn));
    }

    #[test]
    fn test_backup_codes() {
        let mut config = UserMFAConfig::new("user123".to_string());
        let codes = config.generate_backup_codes(5);
        assert_eq!(codes.len(), 5);

        let first_code = codes[0].clone();
        assert!(config.verify_backup_code(&first_code));
        assert!(!config.verify_backup_code(&first_code)); // Should fail second time
    }
}
