//! Data at rest encryption using AES-256-GCM

use crate::error::{Result, SecurityError};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Data at rest encryption service
pub struct AtRestEncryption {
    cipher: Aes256Gcm,
}

impl AtRestEncryption {
    /// Create with a 256-bit key
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new(key.into()),
        }
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| SecurityError::EncryptionFailed(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(SecurityError::DecryptionFailed(
                "Invalid ciphertext length".to_string(),
            ));
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))
    }

    /// Encrypt string to base64
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        Ok(BASE64.encode(encrypted))
    }

    /// Decrypt base64 string
    pub fn decrypt_string(&self, encrypted: &str) -> Result<String> {
        let encrypted_bytes = BASE64
            .decode(encrypted)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))?;

        let decrypted = self.decrypt(&encrypted_bytes)?;
        String::from_utf8(decrypted)
            .map_err(|e| SecurityError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt() {
        let service = AtRestEncryption::new(&test_key());
        let plaintext = b"sensitive data";

        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let service = AtRestEncryption::new(&test_key());
        let plaintext = "sensitive data";

        let encrypted = service.encrypt_string(plaintext).unwrap();
        let decrypted = service.decrypt_string(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
