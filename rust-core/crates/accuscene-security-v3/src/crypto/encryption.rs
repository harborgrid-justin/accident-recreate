//! AES-256-GCM encryption and decryption
//!
//! Provides authenticated encryption using AES-256-GCM (Galois/Counter Mode).
//! This provides both confidentiality and authenticity.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// AES-GCM encryptor
#[derive(Debug)]
pub struct AesGcmEncryptor {
    cipher: Aes256Gcm,
}

impl AesGcmEncryptor {
    /// Create a new encryptor with a random key
    pub fn new() -> SecurityResult<(Self, Zeroizing<Vec<u8>>)> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let key_bytes = Zeroizing::new(key.to_vec());
        let cipher = Aes256Gcm::new(&key);

        Ok((Self { cipher }, key_bytes))
    }

    /// Create an encryptor from an existing key
    pub fn from_key(key: &[u8]) -> SecurityResult<Self> {
        if key.len() != 32 {
            return Err(SecurityError::InvalidKey(
                "AES-256 requires a 32-byte key".to_string(),
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> SecurityResult<EncryptedData> {
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12]; // 96 bits for GCM
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate nonce: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| {
                SecurityError::EncryptionFailed
            })?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted: &EncryptedData) -> SecurityResult<Zeroizing<Vec<u8>>> {
        if encrypted.nonce.len() != 12 {
            return Err(SecurityError::InvalidInput(
                "Invalid nonce length".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| {
                SecurityError::DecryptionFailed
            })?;

        Ok(Zeroizing::new(plaintext))
    }

    /// Encrypt and encode as base64
    pub fn encrypt_to_base64(&self, plaintext: &[u8]) -> SecurityResult<String> {
        let encrypted = self.encrypt(plaintext)?;
        Ok(encrypted.to_base64())
    }

    /// Decrypt from base64
    pub fn decrypt_from_base64(&self, encoded: &str) -> SecurityResult<Zeroizing<Vec<u8>>> {
        let encrypted = EncryptedData::from_base64(encoded)?;
        self.decrypt(&encrypted)
    }
}

impl Default for AesGcmEncryptor {
    fn default() -> Self {
        Self::new().unwrap().0
    }
}

/// Encrypted data with nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Ciphertext (includes authentication tag)
    pub ciphertext: Vec<u8>,

    /// Nonce used for encryption
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    /// Encode to base64 (nonce:ciphertext)
    pub fn to_base64(&self) -> String {
        let combined = [&self.nonce[..], &self.ciphertext[..]].concat();
        base64::encode(&combined)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> SecurityResult<Self> {
        let combined = base64::decode(encoded)
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid base64: {}", e)))?;

        if combined.len() < 12 {
            return Err(SecurityError::InvalidInput(
                "Encoded data too short".to_string(),
            ));
        }

        let nonce = combined[..12].to_vec();
        let ciphertext = combined[12..].to_vec();

        Ok(Self { ciphertext, nonce })
    }
}

/// Envelope encryption for large data
pub struct EnvelopeEncryption;

impl EnvelopeEncryption {
    /// Encrypt data using envelope encryption
    /// Returns (encrypted_data, encrypted_data_key)
    pub fn encrypt(
        plaintext: &[u8],
        master_key: &[u8],
    ) -> SecurityResult<(EncryptedData, EncryptedData)> {
        // Generate a random data encryption key (DEK)
        let (dek_encryptor, dek) = AesGcmEncryptor::new()?;

        // Encrypt the data with the DEK
        let encrypted_data = dek_encryptor.encrypt(plaintext)?;

        // Encrypt the DEK with the master key (KEK - Key Encryption Key)
        let kek_encryptor = AesGcmEncryptor::from_key(master_key)?;
        let encrypted_dek = kek_encryptor.encrypt(&dek)?;

        Ok((encrypted_data, encrypted_dek))
    }

    /// Decrypt data using envelope encryption
    pub fn decrypt(
        encrypted_data: &EncryptedData,
        encrypted_dek: &EncryptedData,
        master_key: &[u8],
    ) -> SecurityResult<Zeroizing<Vec<u8>>> {
        // Decrypt the DEK with the master key
        let kek_encryptor = AesGcmEncryptor::from_key(master_key)?;
        let dek = kek_encryptor.decrypt(encrypted_dek)?;

        // Decrypt the data with the DEK
        let dek_encryptor = AesGcmEncryptor::from_key(&dek)?;
        dek_encryptor.decrypt(encrypted_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let (encryptor, _key) = AesGcmEncryptor::new().unwrap();
        let plaintext = b"Secret message";

        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_encryption_with_known_key() {
        let key = [0u8; 32]; // Not secure, just for testing
        let encryptor = AesGcmEncryptor::from_key(&key).unwrap();

        let plaintext = b"Secret message";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_base64_encoding() {
        let (encryptor, _key) = AesGcmEncryptor::new().unwrap();
        let plaintext = b"Secret message";

        let encoded = encryptor.encrypt_to_base64(plaintext).unwrap();
        let decrypted = encryptor.decrypt_from_base64(&encoded).unwrap();

        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_tampered_ciphertext() {
        let (encryptor, _key) = AesGcmEncryptor::new().unwrap();
        let plaintext = b"Secret message";

        let mut encrypted = encryptor.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext
        encrypted.ciphertext[0] ^= 1;

        // Decryption should fail due to authentication tag mismatch
        assert!(encryptor.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let result = AesGcmEncryptor::from_key(&[0u8; 16]); // Wrong key size
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_encryption() {
        let master_key = [0u8; 32]; // Not secure, just for testing
        let plaintext = b"Large secret data that needs envelope encryption";

        let (encrypted_data, encrypted_dek) =
            EnvelopeEncryption::encrypt(plaintext, &master_key).unwrap();

        let decrypted = EnvelopeEncryption::decrypt(
            &encrypted_data,
            &encrypted_dek,
            &master_key,
        )
        .unwrap();

        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_encrypted_data_base64() {
        let data = EncryptedData {
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![13, 14, 15, 16],
        };

        let encoded = data.to_base64();
        let decoded = EncryptedData::from_base64(&encoded).unwrap();

        assert_eq!(data.nonce, decoded.nonce);
        assert_eq!(data.ciphertext, decoded.ciphertext);
    }
}
