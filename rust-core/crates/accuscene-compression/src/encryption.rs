//! Compression with optional encryption layer

#[cfg(feature = "encryption")]
use crate::error::{CompressionError, Result};
#[cfg(feature = "encryption")]
use crate::traits::{Algorithm, CompressionLevel};
#[cfg(feature = "encryption")]
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
#[cfg(feature = "encryption")]
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
#[cfg(feature = "encryption")]
use tracing::{debug, trace};

#[cfg(feature = "encryption")]
const ENCRYPTION_MAGIC: u32 = 0x454E4352; // "ENCR"

#[cfg(feature = "encryption")]
const ENCRYPTION_VERSION: u32 = 1;

/// Encrypted data container
#[cfg(feature = "encryption")]
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Nonce used for encryption
    nonce: [u8; 12],
    /// Salt used for key derivation
    salt: [u8; 16],
    /// Encrypted ciphertext
    ciphertext: Vec<u8>,
    /// Compression algorithm used (before encryption)
    algorithm: Algorithm,
}

#[cfg(feature = "encryption")]
impl EncryptedData {
    /// Compress and encrypt data with a password
    pub fn encrypt(
        data: &[u8],
        password: &str,
        algorithm: Algorithm,
        level: CompressionLevel,
    ) -> Result<Self> {
        debug!(
            "Encrypting {} bytes with {:?} compression",
            data.len(),
            algorithm
        );

        // First compress the data
        let compressed = crate::algorithms::compress(data, algorithm, level)?;

        trace!(
            "Compressed {} -> {} bytes before encryption",
            data.len(),
            compressed.len()
        );

        // Generate salt for key derivation
        let salt = SaltString::generate(&mut OsRng);
        let salt_bytes: [u8; 16] = salt.as_str()[0..16]
            .as_bytes()
            .try_into()
            .map_err(|_| CompressionError::Encryption("Failed to generate salt".to_string()))?;

        // Derive key from password using Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CompressionError::Encryption(e.to_string()))?;

        // Extract the 32-byte key from the hash
        let key_bytes = password_hash.hash.ok_or_else(|| {
            CompressionError::Encryption("Failed to derive key".to_string())
        })?;

        let key: [u8; 32] = key_bytes.as_bytes()[0..32]
            .try_into()
            .map_err(|_| CompressionError::Encryption("Invalid key length".to_string()))?;

        // Create cipher
        let cipher = Aes256Gcm::new(&key.into());

        // Generate nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, compressed.as_ref())
            .map_err(|e| CompressionError::Encryption(e.to_string()))?;

        debug!("Encrypted data: {} bytes", ciphertext.len());

        Ok(Self {
            nonce: nonce_bytes,
            salt: salt_bytes,
            ciphertext,
            algorithm,
        })
    }

    /// Decrypt and decompress data with a password
    pub fn decrypt(&self, password: &str) -> Result<Vec<u8>> {
        debug!("Decrypting {} bytes", self.ciphertext.len());

        // Reconstruct salt
        let salt_str = std::str::from_utf8(&self.salt)
            .map_err(|e| CompressionError::Decryption(e.to_string()))?;
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| CompressionError::Decryption(e.to_string()))?;

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CompressionError::Decryption(e.to_string()))?;

        let key_bytes = password_hash.hash.ok_or_else(|| {
            CompressionError::Decryption("Failed to derive key".to_string())
        })?;

        let key: [u8; 32] = key_bytes.as_bytes()[0..32]
            .try_into()
            .map_err(|_| CompressionError::Decryption("Invalid key length".to_string()))?;

        // Create cipher
        let cipher = Aes256Gcm::new(&key.into());

        // Decrypt
        let nonce = Nonce::from_slice(&self.nonce);
        let compressed = cipher
            .decrypt(nonce, self.ciphertext.as_ref())
            .map_err(|e| CompressionError::Decryption(e.to_string()))?;

        trace!("Decrypted {} bytes, decompressing", compressed.len());

        // Decompress
        let decompressed = crate::algorithms::decompress(&compressed, self.algorithm)?;

        debug!("Decrypted and decompressed to {} bytes", decompressed.len());
        Ok(decompressed)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write header
        bytes.extend_from_slice(&ENCRYPTION_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&ENCRYPTION_VERSION.to_le_bytes());

        // Write algorithm
        bytes.push(self.algorithm as u8);

        // Write nonce
        bytes.extend_from_slice(&self.nonce);

        // Write salt
        bytes.extend_from_slice(&self.salt);

        // Write ciphertext length and data
        bytes.extend_from_slice(&(self.ciphertext.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.ciphertext);

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 41 {
            return Err(CompressionError::Decryption(
                "Data too small for encrypted container".to_string(),
            ));
        }

        let mut offset = 0;

        // Read and verify header
        let magic = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if magic != ENCRYPTION_MAGIC {
            return Err(CompressionError::InvalidMagic {
                expected: ENCRYPTION_MAGIC,
                actual: magic,
            });
        }

        let version = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if version != ENCRYPTION_VERSION {
            return Err(CompressionError::UnsupportedVersion(version));
        }

        // Read algorithm
        let algorithm = match bytes[offset] {
            0 => Algorithm::Lz4,
            1 => Algorithm::Zstd,
            2 => Algorithm::Brotli,
            3 => Algorithm::Deflate,
            4 => Algorithm::Snappy,
            _ => {
                return Err(CompressionError::Decryption(format!(
                    "Invalid algorithm: {}",
                    bytes[offset]
                )))
            }
        };
        offset += 1;

        // Read nonce
        let nonce: [u8; 12] = bytes[offset..offset + 12]
            .try_into()
            .map_err(|_| CompressionError::Decryption("Invalid nonce".to_string()))?;
        offset += 12;

        // Read salt
        let salt: [u8; 16] = bytes[offset..offset + 16]
            .try_into()
            .map_err(|_| CompressionError::Decryption("Invalid salt".to_string()))?;
        offset += 16;

        // Read ciphertext
        let ciphertext_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let ciphertext = bytes[offset..offset + ciphertext_len].to_vec();

        Ok(Self {
            nonce,
            salt,
            ciphertext,
            algorithm,
        })
    }
}

/// Compress and encrypt data in one operation
#[cfg(feature = "encryption")]
pub fn compress_encrypt(
    data: &[u8],
    password: &str,
    algorithm: Algorithm,
    level: CompressionLevel,
) -> Result<Vec<u8>> {
    let encrypted = EncryptedData::encrypt(data, password, algorithm, level)?;
    Ok(encrypted.to_bytes())
}

/// Decrypt and decompress data in one operation
#[cfg(feature = "encryption")]
pub fn decrypt_decompress(data: &[u8], password: &str) -> Result<Vec<u8>> {
    let encrypted = EncryptedData::from_bytes(data)?;
    encrypted.decrypt(password)
}

// Provide stub implementations when encryption feature is disabled
#[cfg(not(feature = "encryption"))]
pub fn compress_encrypt(
    _data: &[u8],
    _password: &str,
    _algorithm: crate::traits::Algorithm,
    _level: crate::traits::CompressionLevel,
) -> crate::error::Result<Vec<u8>> {
    Err(crate::error::CompressionError::Encryption(
        "Encryption feature not enabled".to_string(),
    ))
}

#[cfg(not(feature = "encryption"))]
pub fn decrypt_decompress(_data: &[u8], _password: &str) -> crate::error::Result<Vec<u8>> {
    Err(crate::error::CompressionError::Decryption(
        "Encryption feature not enabled".to_string(),
    ))
}

#[cfg(all(test, feature = "encryption"))]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_round_trip() {
        let data = b"Sensitive AccuScene case data that needs encryption";
        let password = "secure_password_123";

        let encrypted = EncryptedData::encrypt(
            data,
            password,
            Algorithm::Zstd,
            CompressionLevel::Default,
        )
        .unwrap();

        let decrypted = encrypted.decrypt(password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_encryption_serialization() {
        let data = b"Test data for encryption serialization";
        let password = "test_password";

        let encrypted = EncryptedData::encrypt(
            data,
            password,
            Algorithm::Lz4,
            CompressionLevel::Fast,
        )
        .unwrap();

        let bytes = encrypted.to_bytes();
        let restored = EncryptedData::from_bytes(&bytes).unwrap();

        let decrypted = restored.decrypt(password).unwrap();
        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let data = b"Secret data";
        let password = "correct_password";

        let encrypted = EncryptedData::encrypt(
            data,
            password,
            Algorithm::Zstd,
            CompressionLevel::Default,
        )
        .unwrap();

        let result = encrypted.decrypt("wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_encrypt_helpers() {
        let data = b"Helper function test data";
        let password = "helper_password";

        let encrypted = compress_encrypt(
            data,
            password,
            Algorithm::Zstd,
            CompressionLevel::Default,
        )
        .unwrap();

        let decrypted = decrypt_decompress(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }
}
