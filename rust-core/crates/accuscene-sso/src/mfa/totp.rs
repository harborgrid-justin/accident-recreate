//! TOTP (Time-based One-Time Password) Implementation

use crate::{SSOError, SSOResult};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use totp_lite::{totp_custom, Sha1};

/// TOTP secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TOTPSecret {
    /// Base32-encoded secret
    pub secret: String,
    /// Issuer name
    pub issuer: String,
    /// Account name (usually email)
    pub account_name: String,
}

impl TOTPSecret {
    /// Generate new TOTP secret
    pub fn generate(issuer: String, account_name: String) -> Self {
        use rand::Rng;
        let secret_bytes: Vec<u8> = (0..20)
            .map(|_| rand::thread_rng().gen())
            .collect();

        let secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);

        Self {
            secret,
            issuer,
            account_name,
        }
    }

    /// Get provisioning URI for QR code
    pub fn provisioning_uri(&self) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            urlencoding::encode(&self.issuer),
            urlencoding::encode(&self.account_name),
            self.secret,
            urlencoding::encode(&self.issuer)
        )
    }

    /// Generate QR code as PNG bytes
    pub fn generate_qr_code(&self) -> SSOResult<Vec<u8>> {
        let uri = self.provisioning_uri();
        let code = QrCode::new(uri.as_bytes())
            .map_err(|e| SSOError::InternalError(format!("Failed to generate QR code: {}", e)))?;

        let image = code.render::<qrcode::render::unicode::Dense1x2>().build();

        // For actual PNG generation, use image crate in production
        // This is a placeholder
        Ok(image.as_bytes().to_vec())
    }

    /// Decode base32 secret to bytes
    fn decode_secret(&self) -> SSOResult<Vec<u8>> {
        base32::decode(base32::Alphabet::RFC4648 { padding: false }, &self.secret)
            .ok_or_else(|| SSOError::InternalError("Failed to decode TOTP secret".to_string()))
    }
}

/// TOTP manager
pub struct TOTPManager {
    /// Issuer name
    issuer: String,
    /// Time step in seconds (default: 30)
    time_step: u64,
    /// Number of digits (default: 6)
    digits: usize,
    /// Time tolerance in steps (allow codes from previous/next step)
    tolerance: u64,
}

impl TOTPManager {
    /// Create new TOTP manager
    pub fn new(issuer: String) -> Self {
        Self {
            issuer,
            time_step: 30,
            digits: 6,
            tolerance: 1,
        }
    }

    /// Generate TOTP secret for user
    pub fn generate_secret(&self, account_name: String) -> TOTPSecret {
        TOTPSecret::generate(self.issuer.clone(), account_name)
    }

    /// Verify TOTP code
    pub fn verify(&self, secret: &TOTPSecret, code: &str) -> SSOResult<bool> {
        // Decode secret
        let secret_bytes = secret.decode_secret()?;

        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SSOError::InternalError(format!("System time error: {}", e)))?
            .as_secs();

        // Try current time step and tolerance window
        for offset in -(self.tolerance as i64)..=(self.tolerance as i64) {
            let adjusted_time = (timestamp as i64 + offset * self.time_step as i64) as u64;
            let expected_code = self.generate_code(&secret_bytes, adjusted_time);

            if expected_code == code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Generate TOTP code for given timestamp
    fn generate_code(&self, secret: &[u8], timestamp: u64) -> String {
        let steps = timestamp / self.time_step;
        totp_custom::<Sha1>(self.time_step, self.digits, secret, steps)
    }

    /// Get current TOTP code (for testing)
    #[cfg(test)]
    pub fn get_current_code(&self, secret: &TOTPSecret) -> SSOResult<String> {
        let secret_bytes = secret.decode_secret()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(self.generate_code(&secret_bytes, timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_secret_generation() {
        let secret = TOTPSecret::generate(
            "AccuScene".to_string(),
            "user@example.com".to_string(),
        );

        assert!(!secret.secret.is_empty());
        assert_eq!(secret.issuer, "AccuScene");
        assert_eq!(secret.account_name, "user@example.com");
    }

    #[test]
    fn test_provisioning_uri() {
        let secret = TOTPSecret {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            issuer: "AccuScene".to_string(),
            account_name: "user@example.com".to_string(),
        };

        let uri = secret.provisioning_uri();
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("AccuScene"));
        assert!(uri.contains("user@example.com"));
    }

    #[test]
    fn test_totp_verification() {
        let manager = TOTPManager::new("AccuScene".to_string());
        let secret = manager.generate_secret("user@example.com".to_string());

        // Generate current code
        let code = manager.get_current_code(&secret).unwrap();

        // Verify it
        assert!(manager.verify(&secret, &code).unwrap());

        // Wrong code should fail
        assert!(!manager.verify(&secret, "000000").unwrap());
    }
}
