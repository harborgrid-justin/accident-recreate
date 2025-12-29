//! Multi-factor authentication (TOTP, WebAuthn)
//!
//! Provides MFA support for enhanced security:
//! - TOTP (Time-based One-Time Password) like Google Authenticator
//! - WebAuthn for hardware security keys and biometric authentication

use crate::config::MfaConfig;
use crate::error::{SecurityError, SecurityResult};
use base32::Alphabet;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1};
use webauthn_rs::prelude::*;
use zeroize::Zeroizing;

/// TOTP manager for time-based one-time passwords
#[derive(Debug)]
pub struct TotpManager {
    config: MfaConfig,
}

impl TotpManager {
    /// Create a new TOTP manager
    pub fn new(config: MfaConfig) -> Self {
        Self { config }
    }

    /// Generate a new TOTP secret for a user
    pub fn generate_secret(&self) -> SecurityResult<TotpSecret> {
        let mut secret = vec![0u8; 20]; // 160 bits
        getrandom::getrandom(&mut secret)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate secret: {}", e)))?;

        let secret_b32 = base32::encode(Alphabet::RFC4648 { padding: false }, &secret);

        Ok(TotpSecret {
            secret: Zeroizing::new(secret_b32),
            algorithm: "SHA1".to_string(),
            digits: 6,
            period: self.config.totp_step_secs,
        })
    }

    /// Generate a provisioning URI for QR code
    pub fn get_provisioning_uri(
        &self,
        secret: &TotpSecret,
        account_name: &str,
        issuer: &str,
    ) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            urlencoding::encode(issuer),
            urlencoding::encode(account_name),
            secret.secret.as_str(),
            urlencoding::encode(issuer),
            secret.algorithm,
            secret.digits,
            secret.period
        )
    }

    /// Verify a TOTP code
    pub fn verify_code(
        &self,
        secret: &TotpSecret,
        code: &str,
    ) -> SecurityResult<bool> {
        if code.len() != secret.digits as usize {
            return Ok(false);
        }

        // Decode the base32 secret
        let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: false }, &secret.secret)
            .ok_or_else(|| SecurityError::MfaError("Invalid secret encoding".to_string()))?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::CryptoError(format!("Time error: {}", e)))?
            .as_secs();

        // Check current time step and adjacent time steps (to handle clock skew)
        for offset in -(self.config.totp_window as i64)..=(self.config.totp_window as i64) {
            let time_step = (current_time as i64 + offset * secret.period as i64) / secret.period as i64;
            let expected_code = totp_custom::<Sha1>(
                secret.period,
                secret.digits,
                &secret_bytes,
                time_step as u64,
            );

            if code == expected_code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate current TOTP code (for testing)
    #[cfg(test)]
    pub fn generate_code(&self, secret: &TotpSecret) -> SecurityResult<String> {
        let secret_bytes = base32::decode(Alphabet::RFC4648 { padding: false }, &secret.secret)
            .ok_or_else(|| SecurityError::MfaError("Invalid secret encoding".to_string()))?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::CryptoError(format!("Time error: {}", e)))?
            .as_secs();

        let time_step = current_time / secret.period;
        let code = totp_custom::<Sha1>(secret.period, secret.digits, &secret_bytes, time_step);

        Ok(code)
    }
}

/// TOTP secret information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    /// Base32-encoded secret
    #[serde(skip_serializing)]
    pub secret: Zeroizing<String>,

    /// Hash algorithm (SHA1, SHA256, SHA512)
    pub algorithm: String,

    /// Number of digits in the code
    pub digits: u32,

    /// Time step in seconds
    pub period: u64,
}

/// WebAuthn manager for hardware security keys and biometrics
#[derive(Debug)]
pub struct WebAuthnManager {
    webauthn: Webauthn,
}

impl WebAuthnManager {
    /// Create a new WebAuthn manager
    pub fn new(config: &MfaConfig) -> SecurityResult<Self> {
        let rp_origin = Url::parse(&format!("https://{}", config.webauthn_rp_id))
            .map_err(|e| SecurityError::ConfigError(format!("Invalid RP ID: {}", e)))?;

        let builder = WebauthnBuilder::new(&config.webauthn_rp_id, &rp_origin)
            .map_err(|e| SecurityError::WebAuthnError(format!("Failed to create builder: {}", e)))?;

        let webauthn = builder
            .rp_name(&config.webauthn_rp_name)
            .build()
            .map_err(|e| SecurityError::WebAuthnError(format!("Failed to build: {}", e)))?;

        Ok(Self { webauthn })
    }

    /// Start credential registration (for new security key)
    pub fn start_registration(
        &self,
        user_id: &str,
        username: &str,
        display_name: &str,
    ) -> SecurityResult<(CreationChallengeResponse, PasskeyRegistration)> {
        let user_unique_id = Uuid::parse_str(user_id)
            .unwrap_or_else(|_| Uuid::new_v4());

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(
                user_unique_id,
                username,
                display_name,
                None,
            )
            .map_err(|e| SecurityError::WebAuthnError(format!("Registration failed: {}", e)))?;

        Ok((ccr, reg_state))
    }

    /// Finish credential registration
    pub fn finish_registration(
        &self,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> SecurityResult<Passkey> {
        let passkey = self
            .webauthn
            .finish_passkey_registration(reg, state)
            .map_err(|e| SecurityError::WebAuthnError(format!("Registration verification failed: {}", e)))?;

        Ok(passkey)
    }

    /// Start authentication challenge
    pub fn start_authentication(
        &self,
        passkeys: &[Passkey],
    ) -> SecurityResult<(RequestChallengeResponse, PasskeyAuthentication)> {
        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(passkeys)
            .map_err(|e| SecurityError::WebAuthnError(format!("Authentication start failed: {}", e)))?;

        Ok((rcr, auth_state))
    }

    /// Finish authentication
    pub fn finish_authentication(
        &self,
        auth: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> SecurityResult<()> {
        self.webauthn
            .finish_passkey_authentication(auth, state)
            .map_err(|e| SecurityError::WebAuthnError(format!("Authentication verification failed: {}", e)))?;

        Ok(())
    }
}

/// MFA enrollment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollment {
    /// User ID
    pub user_id: String,

    /// TOTP secret (if enrolled)
    pub totp_secret: Option<TotpSecret>,

    /// WebAuthn passkeys
    pub passkeys: Vec<Passkey>,

    /// Backup codes
    pub backup_codes: Vec<String>,

    /// Whether MFA is enabled
    pub enabled: bool,
}

impl MfaEnrollment {
    /// Create a new MFA enrollment
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            totp_secret: None,
            passkeys: Vec::new(),
            backup_codes: Vec::new(),
            enabled: false,
        }
    }

    /// Enable TOTP
    pub fn enable_totp(&mut self, secret: TotpSecret) {
        self.totp_secret = Some(secret);
        self.enabled = true;
    }

    /// Add a passkey
    pub fn add_passkey(&mut self, passkey: Passkey) {
        self.passkeys.push(passkey);
        self.enabled = true;
    }

    /// Generate backup codes
    pub fn generate_backup_codes(&mut self, count: usize) -> SecurityResult<Vec<String>> {
        let mut codes = Vec::new();

        for _ in 0..count {
            let mut code_bytes = vec![0u8; 8];
            getrandom::getrandom(&mut code_bytes)
                .map_err(|e| SecurityError::CryptoError(format!("Failed to generate backup code: {}", e)))?;

            let code = base32::encode(Alphabet::RFC4648 { padding: false }, &code_bytes);
            codes.push(code);
        }

        self.backup_codes = codes.clone();
        Ok(codes)
    }

    /// Verify a backup code
    pub fn verify_backup_code(&mut self, code: &str) -> bool {
        if let Some(pos) = self.backup_codes.iter().position(|c| c == code) {
            self.backup_codes.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> MfaConfig {
        MfaConfig {
            require_mfa: false,
            require_mfa_for_admins: true,
            totp_window: 1,
            totp_step_secs: 30,
            enable_webauthn: true,
            webauthn_rp_name: "AccuScene Test".to_string(),
            webauthn_rp_id: "localhost".to_string(),
        }
    }

    #[test]
    fn test_totp_secret_generation() {
        let config = create_test_config();
        let manager = TotpManager::new(config);

        let secret = manager.generate_secret().unwrap();
        assert!(!secret.secret.is_empty());
        assert_eq!(secret.algorithm, "SHA1");
        assert_eq!(secret.digits, 6);
    }

    #[test]
    fn test_totp_provisioning_uri() {
        let config = create_test_config();
        let manager = TotpManager::new(config);

        let secret = manager.generate_secret().unwrap();
        let uri = manager.get_provisioning_uri(&secret, "user@example.com", "AccuScene");

        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("user@example.com"));
        assert!(uri.contains("AccuScene"));
    }

    #[test]
    fn test_totp_code_verification() {
        let config = create_test_config();
        let manager = TotpManager::new(config);

        let secret = manager.generate_secret().unwrap();
        let code = manager.generate_code(&secret).unwrap();

        assert!(manager.verify_code(&secret, &code).unwrap());
        assert!(!manager.verify_code(&secret, "000000").unwrap());
    }

    #[test]
    fn test_mfa_enrollment() {
        let mut enrollment = MfaEnrollment::new("user123");
        assert!(!enrollment.enabled);

        let config = create_test_config();
        let manager = TotpManager::new(config);
        let secret = manager.generate_secret().unwrap();

        enrollment.enable_totp(secret);
        assert!(enrollment.enabled);
        assert!(enrollment.totp_secret.is_some());
    }

    #[test]
    fn test_backup_codes() {
        let mut enrollment = MfaEnrollment::new("user123");
        let codes = enrollment.generate_backup_codes(10).unwrap();

        assert_eq!(codes.len(), 10);
        assert_eq!(enrollment.backup_codes.len(), 10);

        let first_code = codes[0].clone();
        assert!(enrollment.verify_backup_code(&first_code));
        assert_eq!(enrollment.backup_codes.len(), 9);

        // Code should not work twice
        assert!(!enrollment.verify_backup_code(&first_code));
    }
}
