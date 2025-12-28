//! Multi-factor authentication (TOTP and WebAuthn)
//!
//! Implements MFA following security best practices.

use crate::config::MfaConfig;
use crate::error::{Result, SecurityError};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1};

/// MFA service for TOTP and WebAuthn
pub struct MfaService {
    config: MfaConfig,
}

impl MfaService {
    /// Create a new MFA service
    pub fn new(config: MfaConfig) -> Self {
        Self { config }
    }

    /// Generate a new TOTP secret
    pub fn generate_totp_secret(&self) -> TotpSecret {
        let mut rng = rand::thread_rng();
        let secret: Vec<u8> = (0..20).map(|_| rng.gen()).collect();

        TotpSecret {
            secret: BASE64.encode(&secret),
            algorithm: "SHA1".to_string(),
            digits: self.config.totp.digits,
            period: self.config.totp.time_step,
        }
    }

    /// Generate TOTP provisioning URI for QR code
    pub fn generate_totp_uri(
        &self,
        secret: &TotpSecret,
        account_name: &str,
    ) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            urlencoding::encode(&self.config.totp.issuer),
            urlencoding::encode(account_name),
            secret.secret,
            urlencoding::encode(&self.config.totp.issuer),
            secret.algorithm,
            secret.digits,
            secret.period
        )
    }

    /// Verify TOTP code
    pub fn verify_totp(&self, secret: &TotpSecret, code: &str) -> Result<bool> {
        if code.len() != self.config.totp.digits as usize {
            return Ok(false);
        }

        // Decode secret
        let secret_bytes = BASE64
            .decode(&secret.secret)
            .map_err(|e| SecurityError::Internal(format!("Invalid TOTP secret: {}", e)))?;

        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SecurityError::Internal(format!("Time error: {}", e)))?
            .as_secs();

        // Check current time and adjacent time steps (to account for clock skew)
        for skew in -(self.config.totp.skew as i64)..=(self.config.totp.skew as i64) {
            let time_step = (timestamp as i64 / self.config.totp.time_step as i64) + skew;
            let generated = totp_custom::<Sha1>(
                self.config.totp.time_step,
                self.config.totp.digits,
                &secret_bytes,
                time_step as u64,
            );

            if format!("{:0width$}", generated, width = self.config.totp.digits as usize) == code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate backup codes
    pub fn generate_backup_codes(&self) -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..self.config.backup_codes_count)
            .map(|_| {
                format!(
                    "{:04}-{:04}-{:04}",
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000)
                )
            })
            .collect()
    }

    /// Hash backup code for storage
    pub fn hash_backup_code(&self, code: &str) -> Result<String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    /// Verify backup code
    pub fn verify_backup_code(&self, code: &str, hash: &str) -> Result<bool> {
        let code_hash = self.hash_backup_code(code)?;
        Ok(code_hash == hash)
    }
}

/// TOTP secret information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    /// Base64-encoded secret
    pub secret: String,
    /// Algorithm (SHA1, SHA256, SHA512)
    pub algorithm: String,
    /// Number of digits
    pub digits: u32,
    /// Time period in seconds
    pub period: u64,
}

/// MFA enrollment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaEnrollment {
    /// User ID
    pub user_id: String,
    /// TOTP enabled
    pub totp_enabled: bool,
    /// TOTP secret (if enrolled)
    pub totp_secret: Option<TotpSecret>,
    /// WebAuthn enabled
    pub webauthn_enabled: bool,
    /// WebAuthn credentials
    pub webauthn_credentials: Vec<WebAuthnCredential>,
    /// Backup codes (hashed)
    pub backup_codes: Vec<String>,
    /// Backup codes used count
    pub backup_codes_used: usize,
}

impl MfaEnrollment {
    /// Create new enrollment
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            totp_enabled: false,
            totp_secret: None,
            webauthn_enabled: false,
            webauthn_credentials: Vec::new(),
            backup_codes: Vec::new(),
            backup_codes_used: 0,
        }
    }

    /// Check if any MFA method is enabled
    pub fn is_enrolled(&self) -> bool {
        self.totp_enabled || self.webauthn_enabled
    }

    /// Check if MFA is required
    pub fn requires_verification(&self) -> bool {
        self.is_enrolled()
    }
}

/// WebAuthn credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnCredential {
    /// Credential ID
    pub id: String,
    /// Credential public key
    pub public_key: Vec<u8>,
    /// Sign count
    pub sign_count: u32,
    /// Credential nickname
    pub nickname: Option<String>,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// MFA verification result
#[derive(Debug, Clone)]
pub struct MfaVerification {
    /// Verification successful
    pub verified: bool,
    /// Method used
    pub method: MfaMethod,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// MFA method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaMethod {
    /// TOTP (Time-based One-Time Password)
    Totp,
    /// WebAuthn (FIDO2)
    WebAuthn,
    /// Backup code
    BackupCode,
}

impl std::fmt::Display for MfaMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MfaMethod::Totp => write!(f, "TOTP"),
            MfaMethod::WebAuthn => write!(f, "WebAuthn"),
            MfaMethod::BackupCode => write!(f, "BackupCode"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TotpConfig;

    fn test_config() -> MfaConfig {
        MfaConfig {
            required: false,
            totp: TotpConfig {
                issuer: "AccuScene Test".to_string(),
                time_step: 30,
                digits: 6,
                skew: 1,
            },
            webauthn: crate::config::WebAuthnConfig::default(),
            backup_codes_count: 10,
        }
    }

    #[test]
    fn test_totp_secret_generation() {
        let service = MfaService::new(test_config());
        let secret = service.generate_totp_secret();

        assert!(!secret.secret.is_empty());
        assert_eq!(secret.digits, 6);
        assert_eq!(secret.period, 30);
    }

    #[test]
    fn test_totp_uri_generation() {
        let service = MfaService::new(test_config());
        let secret = service.generate_totp_secret();
        let uri = service.generate_totp_uri(&secret, "test@example.com");

        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("test@example.com"));
        assert!(uri.contains("AccuScene%20Test"));
    }

    #[test]
    fn test_backup_codes_generation() {
        let service = MfaService::new(test_config());
        let codes = service.generate_backup_codes();

        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), 14); // Format: XXXX-XXXX-XXXX
            assert_eq!(code.chars().filter(|&c| c == '-').count(), 2);
        }
    }

    #[test]
    fn test_backup_code_verification() {
        let service = MfaService::new(test_config());
        let code = "1234-5678-9012";
        let hash = service.hash_backup_code(code).unwrap();

        assert!(service.verify_backup_code(code, &hash).unwrap());
        assert!(!service.verify_backup_code("0000-0000-0000", &hash).unwrap());
    }

    #[test]
    fn test_mfa_enrollment() {
        let enrollment = MfaEnrollment::new("user123".to_string());

        assert!(!enrollment.is_enrolled());
        assert!(!enrollment.requires_verification());

        let mut enrollment = enrollment;
        enrollment.totp_enabled = true;
        assert!(enrollment.is_enrolled());
        assert!(enrollment.requires_verification());
    }
}
