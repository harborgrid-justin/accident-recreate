//! ChaCha20-Poly1305 authenticated encryption
//!
//! Provides ChaCha20-Poly1305 encryption and decryption with authenticated encryption.

use crate::error::{CryptoError, CryptoResult};
use crate::random::generate_chacha_nonce;
use crate::secure_memory::SecureBytes;
use crate::symmetric::aes::EncryptedData;
use crate::symmetric::key::SymmetricKey;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305 as ChaChaPolycipher, Nonce,
};

/// Nonce size for ChaCha20-Poly1305 (96 bits / 12 bytes)
pub const NONCE_SIZE: usize = 12;

/// ChaCha20-Poly1305 cipher
pub struct ChaCha20Poly1305 {
    cipher: ChaChaPolycipher,
}

impl ChaCha20Poly1305 {
    /// Create a new ChaCha20-Poly1305 cipher with the given key
    pub fn new(key: &SymmetricKey) -> Self {
        let cipher = ChaChaPolycipher::new(key.as_bytes().into());
        Self { cipher }
    }

    /// Encrypt plaintext with optional associated data
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> CryptoResult<EncryptedData> {
        let nonce_bytes = generate_chacha_nonce()?;
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
            algorithm: "chacha20-poly1305".to_string(),
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

/// Encrypt data using ChaCha20-Poly1305
pub fn encrypt_chacha20poly1305(
    key: &SymmetricKey,
    plaintext: &[u8],
    associated_data: Option<&[u8]>,
) -> CryptoResult<EncryptedData> {
    let cipher = ChaCha20Poly1305::new(key);
    cipher.encrypt(plaintext, associated_data)
}

/// Decrypt data using ChaCha20-Poly1305
pub fn decrypt_chacha20poly1305(
    key: &SymmetricKey,
    encrypted: &EncryptedData,
    associated_data: Option<&[u8]>,
) -> CryptoResult<SecureBytes> {
    let cipher = ChaCha20Poly1305::new(key);
    cipher.decrypt(encrypted, associated_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();
        let decrypted = decrypt_chacha20poly1305(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";
        let aad = b"metadata";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, Some(aad)).unwrap();
        let decrypted = decrypt_chacha20poly1305(&key, &encrypted, Some(aad)).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";
        let aad = b"metadata";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, Some(aad)).unwrap();
        let result = decrypt_chacha20poly1305(&key, &encrypted, Some(b"wrong_metadata"));

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key1 = SymmetricKey::generate().unwrap();
        let key2 = SymmetricKey::generate().unwrap();
        let plaintext = b"Secret message";

        let encrypted = encrypt_chacha20poly1305(&key1, plaintext, None).unwrap();
        let result = decrypt_chacha20poly1305(&key2, &encrypted, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_algorithm_field() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Test data";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();
        assert_eq!(encrypted.algorithm, "chacha20-poly1305");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Same plaintext";

        let encrypted1 = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();
        let encrypted2 = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();

        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // Ciphertexts should be different due to different nonces
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();
        let decrypted = decrypt_chacha20poly1305(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext, decrypted.as_bytes());
    }

    #[test]
    fn test_large_plaintext() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = vec![42u8; 1024 * 1024]; // 1 MB

        let encrypted = encrypt_chacha20poly1305(&key, &plaintext, None).unwrap();
        let decrypted = decrypt_chacha20poly1305(&key, &encrypted, None).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_bytes());
    }

    #[test]
    fn test_cipher_reuse() {
        let key = SymmetricKey::generate().unwrap();
        let cipher = ChaCha20Poly1305::new(&key);

        let plaintext1 = b"First message";
        let plaintext2 = b"Second message";

        let encrypted1 = cipher.encrypt(plaintext1, None).unwrap();
        let encrypted2 = cipher.encrypt(plaintext2, None).unwrap();

        let decrypted1 = cipher.decrypt(&encrypted1, None).unwrap();
        let decrypted2 = cipher.decrypt(&encrypted2, None).unwrap();

        assert_eq!(plaintext1, decrypted1.as_bytes());
        assert_eq!(plaintext2, decrypted2.as_bytes());
    }

    #[test]
    fn test_encrypted_data_base64_roundtrip() {
        let key = SymmetricKey::generate().unwrap();
        let plaintext = b"Test data";

        let encrypted = encrypt_chacha20poly1305(&key, plaintext, None).unwrap();
        let encoded = encrypted.to_base64().unwrap();
        let decoded = EncryptedData::from_base64(&encoded).unwrap();

        let decrypted = decrypt_chacha20poly1305(&key, &decoded, None).unwrap();
        assert_eq!(plaintext, decrypted.as_bytes());
    }
}
