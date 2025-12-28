//! AES-256-GCM authenticated encryption
//!
//! Provides AES-256-GCM encryption and decryption with authenticated encryption.

use crate::error::{CryptoError, CryptoResult};
use crate::random::generate_aes_gcm_nonce;
use crate::secure_memory::SecureBytes;
use crate::symmetric::key::SymmetricKey;
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm as AesGcmCipher, Nonce,
};
use serde::{Deserialize, Serialize};

/// Nonce size for AES-GCM (96 bits / 12 bytes)
pub const NONCE_SIZE: usize = 12;

/// AES-256-GCM cipher
pub struct Aes256Gcm {
    cipher: AesGcmCipher,
}

impl Aes256Gcm {
    /// Create a new AES-256-GCM cipher with the given key
    pub fn new(key: &SymmetricKey) -> Self {
        let cipher = AesGcmCipher::new(key.as_bytes().into());
        Self { cipher }
    }

    /// Encrypt plaintext with optional associated data
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> CryptoResult<EncryptedData> {
        let nonce_bytes = generate_aes_gcm_nonce()?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad: associated_data.unwrap_or(&[]),
        };

        let ciphertext = self
            .cipher
            .encrypt(nonce, payload)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            algorithm: "aes-256-gcm".to_string(),
        })
    }

    /// Decrypt ciphertext with optional associated data
    pub fn decrypt(
        &self,
        encrypted: &EncryptedData,
        associated_data: Option<&[u8]>,
    ) -> CryptoResult<SecureBytes> {
        if encrypted.nonce.len() != NONCE_SIZE {
            return Err(CryptoError::InvalidNonceSize {
                expected: NONCE_SIZE,
                actual: encrypted.nonce.len(),
            });
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let payload = Payload {
            msg: &encrypted.ciphertext,
            aad: associated_data.unwrap_or(&[]),
        };

        let plaintext = self
            .cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        Ok(SecureBytes::new(plaintext))
    }
}

/// Encrypted data with nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The nonce used for encryption
    pub nonce: Vec<u8>,
    /// The ciphertext (includes authentication tag)
    pub ciphertext: Vec<u8>,
    /// The algorithm used
    pub algorithm: String,
}

impl EncryptedData {
    /// Encode to base64 for storage/transmission
    pub fn to_base64(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json.as_bytes()))
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let json_bytes = base64::decode(encoded)?;
        let json_str = std::str::from_utf8(&json_bytes)
            .map_err(|e| CryptoError::DecodingError(e.to_string()))?;
        let data = serde_json::from_str(json_str)?;
        Ok(data)
    }

    /// Get the total size (nonce + ciphertext)
    pub fn total_size(&self) -> usize {
        self.nonce.len() + self.ciphertext.len()
    }
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_aes256gcm(
    key: &SymmetricKey,
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> CryptoResult<EncryptedData> {
    let cipher = Aes256Gcm::new(key);
    cipher.encrypt(plaintext, associated_data)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_aes256gcm(
    key: &SymmetricKey,
    encrypted: &EncryptedData,
    associated_data: Option<&[u8]>,
) -> CryptoResult<SecureBytes> {
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(encrypted, associated_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_aes256gcm(&key, plaintext, None).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";
        let aad = b"metadata";

        let encrypted = encrypt_aes256gcm(&key, plaintext, Some(aad)).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted, Some(aad)).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";
        let aad = b"metadata";

        let encrypted = encrypt_aes256gcm(&key, plaintext, Some(aad)).unwrap();
        let result = decrypt_aes256gcm(&key, &encrypted, Some(b"wrong_metadata"));

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key1 = SymmetricKey::generate().unwrap();
        let key2 = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";

        let encrypted = encrypt_aes256gcm(&key1, plaintext, None).unwrap();
        let result = decrypt_aes256gcm(&key2, &encrypted, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_data_base64_roundtrip() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Test data";

        let encrypted = encrypt_aes256gcm(&key, plaintext, None).unwrap();
        let encoded = encrypted.to_base64().unwrap();
        let decoded = EncryptedData::from_base64(&encoded).unwrap();

        assert_eq!(encrypted.nonce, decoded.nonce);
        assert_eq!(encrypted.ciphertext, decoded.ciphertext);
        assert_eq!(encrypted.algorithm, decoded.algorithm);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Same plaintext";

        let encrypted1 = encrypt_aes256gcm(&key, plaintext, None).unwrap();
        let encrypted2 = encrypt_aes256gcm(&key, plaintext, None).unwrap();

        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // Ciphertexts should be different due to different nonces
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"";

        let encrypted = encrypt_aes256gcm(&key, plaintext, None).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_large_plaintext() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = vec![42u8; 1024 * 1024]; // 1 MB

        let encrypted = encrypt_aes256gcm(&key, &plaintext, None).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_bytes());
    }

    #[test]
    fn test_cipher_reuse() {
        let key = SymmetricKey::generate().unwrap();
        let cipher = Aes256Gcm::new(&key);

        let plaintext1 = b"First message";
        let plaintext2 = b"Second message";

        let encrypted1 = cipher.encrypt(plaintext1, None).unwrap();
        let encrypted2 = cipher.encrypt(plaintext2, None).unwrap();

        let decrypted1 = cipher.decrypt(&encrypted1, None).unwrap();
        let decrypted2 = cipher.decrypt(&encrypted2, None).unwrap();

        assert_eq!(plaintext1, decrypted1.as_bytes());
        assert_eq!(plaintext2, decrypted2.as_bytes());
    }
}
