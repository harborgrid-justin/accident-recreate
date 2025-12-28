//! Envelope encryption for large data
//!
//! Provides envelope encryption pattern: data is encrypted with a DEK (Data Encryption Key),
//! and the DEK is encrypted with a KEK (Key Encryption Key).

use crate::error::{CryptoError, CryptoResult};
use crate::secure_memory::SecureBytes;
use crate::symmetric::aes::{encrypt_aes256gcm, decrypt_aes256gcm, EncryptedData};
use crate::symmetric::key::SymmetricKey;
use serde::{Deserialize, Serialize};

/// Envelope-encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeEncrypted {
    /// The encrypted data encryption key (DEK)
    pub encrypted_dek: EncryptedData,
    /// The encrypted data
    pub encrypted_data: EncryptedData,
    /// Metadata about the encryption
    pub metadata: EnvelopeMetadata,
}

/// Metadata for envelope encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeMetadata {
    /// Algorithm used for data encryption
    pub data_algorithm: String,
    /// Algorithm used for DEK encryption
    pub dek_algorithm: String,
    /// Version of the envelope encryption format
    pub version: u32,
}

impl Default for EnvelopeMetadata {
    fn default() -> Self {
        Self {
            data_algorithm: "aes-256-gcm".to_string(),
            dek_algorithm: "aes-256-gcm".to_string(),
            version: 1,
        }
    }
}

impl EnvelopeEncrypted {
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
}

/// Envelope encryptor for large data
pub struct EnvelopeEncryptor {
    kek: SymmetricKey,
}

impl EnvelopeEncryptor {
    /// Create a new envelope encryptor with a KEK (Key Encryption Key)
    pub fn new(kek: SymmetricKey) -> Self {
        Self { kek }
    }

    /// Encrypt data using envelope encryption
    pub fn encrypt(&self, data: &[u8]) -> CryptoResult<EnvelopeEncrypted> {
        // Generate a random DEK (Data Encryption Key)
        let dek = SymmetricKey::generate()?;

        // Encrypt the data with the DEK
        let encrypted_data = encrypt_aes256gcm(&dek, data, None)?;

        // Encrypt the DEK with the KEK
        let dek_bytes = dek.as_bytes();
        let encrypted_dek = encrypt_aes256gcm(&self.kek, dek_bytes, None)?;

        Ok(EnvelopeEncrypted {
            encrypted_dek,
            encrypted_data,
            metadata: EnvelopeMetadata::default(),
        })
    }

    /// Decrypt envelope-encrypted data
    pub fn decrypt(&self, envelope: &EnvelopeEncrypted) -> CryptoResult<SecureBytes> {
        // Decrypt the DEK with the KEK
        let dek_bytes = decrypt_aes256gcm(&self.kek, &envelope.encrypted_dek, None)?;
        let dek = SymmetricKey::from_bytes(dek_bytes.as_bytes())?;

        // Decrypt the data with the DEK
        let data = decrypt_aes256gcm(&dek, &envelope.encrypted_data, None)?;

        Ok(data)
    }

    /// Encrypt data with associated metadata
    pub fn encrypt_with_metadata(
        &self,
        data: &[u8],
        associated_data: &[u8],
    ) -> CryptoResult<EnvelopeEncrypted> {
        // Generate a random DEK
        let dek = SymmetricKey::generate()?;

        // Encrypt the data with the DEK and associated data
        let encrypted_data = encrypt_aes256gcm(&dek, data, Some(associated_data))?;

        // Encrypt the DEK with the KEK
        let dek_bytes = dek.as_bytes();
        let encrypted_dek = encrypt_aes256gcm(&self.kek, dek_bytes, None)?;

        Ok(EnvelopeEncrypted {
            encrypted_dek,
            encrypted_data,
            metadata: EnvelopeMetadata::default(),
        })
    }

    /// Decrypt envelope-encrypted data with associated metadata
    pub fn decrypt_with_metadata(
        &self,
        envelope: &EnvelopeEncrypted,
        associated_data: &[u8],
    ) -> CryptoResult<SecureBytes> {
        // Decrypt the DEK with the KEK
        let dek_bytes = decrypt_aes256gcm(&self.kek, &envelope.encrypted_dek, None)?;
        let dek = SymmetricKey::from_bytes(dek_bytes.as_bytes())?;

        // Decrypt the data with the DEK and associated data
        let data = decrypt_aes256gcm(&dek, &envelope.encrypted_data, Some(associated_data))?;

        Ok(data)
    }
}

/// Encrypt data using envelope encryption
pub fn envelope_encrypt(kek: &SymmetricKey, data: &[u8]) -> CryptoResult<EnvelopeEncrypted> {
    let encryptor = EnvelopeEncryptor::new(kek.clone());
    encryptor.encrypt(data)
}

/// Decrypt envelope-encrypted data
pub fn envelope_decrypt(
    kek: &SymmetricKey,
    envelope: &EnvelopeEncrypted,
) -> CryptoResult<SecureBytes> {
    let encryptor = EnvelopeEncryptor::new(kek.clone());
    encryptor.decrypt(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_encrypt_decrypt() {
        let kek = SymmetricKey::generate().unwrap();
        let data = b"This is secret data that needs to be encrypted";

        let encrypted = envelope_encrypt(&kek, data).unwrap();
        let decrypted = envelope_decrypt(&kek, &encrypted).unwrap();

        assert_eq!(data, decrypted.as_bytes());
    }

    #[test]
    fn test_envelope_encrypt_decrypt_with_metadata() {
        let kek = SymmetricKey::generate().unwrap();
        let encryptor = EnvelopeEncryptor::new(kek);

        let data = b"Secret data";
        let metadata = b"user_id=12345";

        let encrypted = encryptor
            .encrypt_with_metadata(data, metadata)
            .unwrap();
        let decrypted = encryptor
            .decrypt_with_metadata(&encrypted, metadata)
            .unwrap();

        assert_eq!(data, decrypted.as_bytes());
    }

    #[test]
    fn test_envelope_wrong_kek_fails() {
        let kek1 = SymmetricKey::generate().unwrap();
        let kek2 = SymmetricKey::generate().unwrap();

        let data = b"Secret data";
        let encrypted = envelope_encrypt(&kek1, data).unwrap();
        let result = envelope_decrypt(&kek2, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_wrong_metadata_fails() {
        let kek = SymmetricKey::generate().unwrap();
        let encryptor = EnvelopeEncryptor::new(kek);

        let data = b"Secret data";
        let metadata1 = b"user_id=12345";
        let metadata2 = b"user_id=67890";

        let encrypted = encryptor
            .encrypt_with_metadata(data, metadata1)
            .unwrap();
        let result = encryptor.decrypt_with_metadata(&encrypted, metadata2);

        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_base64_roundtrip() {
        let kek = SymmetricKey::generate().unwrap();
        let data = b"Test data";

        let encrypted = envelope_encrypt(&kek, data).unwrap();
        let encoded = encrypted.to_base64().unwrap();
        let decoded = EnvelopeEncrypted::from_base64(&encoded).unwrap();

        let decrypted = envelope_decrypt(&kek, &decoded).unwrap();
        assert_eq!(data, decrypted.as_bytes());
    }

    #[test]
    fn test_envelope_metadata() {
        let kek = SymmetricKey::generate().unwrap();
        let data = b"Test data";

        let encrypted = envelope_encrypt(&kek, data).unwrap();
        assert_eq!(encrypted.metadata.data_algorithm, "aes-256-gcm");
        assert_eq!(encrypted.metadata.dek_algorithm, "aes-256-gcm");
        assert_eq!(encrypted.metadata.version, 1);
    }

    #[test]
    fn test_envelope_large_data() {
        let kek = SymmetricKey::generate().unwrap();
        let data = vec![42u8; 10 * 1024 * 1024]; // 10 MB

        let encrypted = envelope_encrypt(&kek, &data).unwrap();
        let decrypted = envelope_decrypt(&kek, &encrypted).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_bytes());
    }

    #[test]
    fn test_envelope_empty_data() {
        let kek = SymmetricKey::generate().unwrap();
        let data = b"";

        let encrypted = envelope_encrypt(&kek, data).unwrap();
        let decrypted = envelope_decrypt(&kek, &encrypted).unwrap();

        assert_eq!(data, decrypted.as_bytes());
    }
}
