//! Key Derivation Functions (KDF)
//!
//! Provides secure key derivation using HKDF and PBKDF2.

use crate::error::{CryptoError, CryptoResult};
use crate::secure_memory::SecureBytes;
use crate::symmetric::key::SymmetricKey;
use ring::pbkdf2;
use std::num::NonZeroU32;

/// HKDF-SHA256 key derivation
pub struct Hkdf {
    salt: Vec<u8>,
    info: Vec<u8>,
}

impl Hkdf {
    /// Create a new HKDF instance
    pub fn new(salt: Vec<u8>, info: Vec<u8>) -> Self {
        Self { salt, info }
    }

    /// Create an HKDF instance with a default salt
    pub fn with_info(info: Vec<u8>) -> Self {
        Self {
            salt: Vec::new(),
            info,
        }
    }

    /// Derive a key from input key material
    pub fn derive(&self, ikm: &[u8], output_len: usize) -> CryptoResult<SecureBytes> {
        // Extract phase
        let mut hasher = hmac_sha256::Hash::new();
        let salt = if self.salt.is_empty() {
            vec![0u8; 32]
        } else {
            self.salt.clone()
        };

        hasher.update(&salt);
        hasher.update(ikm);
        let prk = hasher.finalize();

        // Expand phase
        let mut output = Vec::with_capacity(output_len);
        let mut previous = Vec::new();
        let mut counter = 1u8;

        while output.len() < output_len {
            let mut hasher = hmac_sha256::Hash::new();
            hasher.update(&prk);
            hasher.update(&previous);
            hasher.update(&self.info);
            hasher.update(&[counter]);

            let t = hasher.finalize();
            let to_copy = std::cmp::min(t.len(), output_len - output.len());
            output.extend_from_slice(&t[..to_copy]);
            previous = t.to_vec();
            counter += 1;
        }

        Ok(SecureBytes::new(output))
    }

    /// Derive a 256-bit symmetric key
    pub fn derive_key(&self, ikm: &[u8]) -> CryptoResult<SymmetricKey> {
        let derived = self.derive(ikm, 32)?;
        SymmetricKey::from_bytes(derived.as_bytes())
    }
}

/// Simple HMAC-SHA256 implementation
mod hmac_sha256 {
    use sha2::{Digest, Sha256};

    pub struct Hash {
        key: Vec<u8>,
        data: Vec<u8>,
    }

    impl Hash {
        pub fn new() -> Self {
            Self {
                key: Vec::new(),
                data: Vec::new(),
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            if self.key.is_empty() {
                self.key = data.to_vec();
            } else {
                self.data.extend_from_slice(data);
            }
        }

        pub fn finalize(self) -> [u8; 32] {
            let mut key = self.key;
            let block_size = 64;

            if key.len() > block_size {
                let mut hasher = Sha256::new();
                hasher.update(&key);
                key = hasher.finalize().to_vec();
            }

            if key.len() < block_size {
                key.resize(block_size, 0);
            }

            let mut ipad = vec![0x36u8; block_size];
            let mut opad = vec![0x5cu8; block_size];

            for i in 0..block_size {
                ipad[i] ^= key[i];
                opad[i] ^= key[i];
            }

            let mut inner_hasher = Sha256::new();
            inner_hasher.update(&ipad);
            inner_hasher.update(&self.data);
            let inner_hash = inner_hasher.finalize();

            let mut outer_hasher = Sha256::new();
            outer_hasher.update(&opad);
            outer_hasher.update(&inner_hash);

            outer_hasher.finalize().into()
        }
    }
}

/// PBKDF2-HMAC-SHA256 key derivation
pub struct Pbkdf2 {
    iterations: u32,
    salt: Vec<u8>,
}

impl Pbkdf2 {
    /// Create a new PBKDF2 instance
    pub fn new(iterations: u32, salt: Vec<u8>) -> Self {
        Self { iterations, salt }
    }

    /// Create a PBKDF2 instance with recommended parameters
    pub fn recommended(salt: Vec<u8>) -> Self {
        Self {
            iterations: 100_000,
            salt,
        }
    }

    /// Derive a key from a password
    pub fn derive(&self, password: &[u8], output_len: usize) -> CryptoResult<SecureBytes> {
        let mut output = vec![0u8; output_len];
        let iterations = NonZeroU32::new(self.iterations)
            .ok_or_else(|| CryptoError::KeyDerivationFailed("Invalid iterations".to_string()))?;

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            &self.salt,
            password,
            &mut output,
        );

        Ok(SecureBytes::new(output))
    }

    /// Derive a 256-bit symmetric key from a password
    pub fn derive_key(&self, password: &[u8]) -> CryptoResult<SymmetricKey> {
        let derived = self.derive(password, 32)?;
        SymmetricKey::from_bytes(derived.as_bytes())
    }
}

/// Derive a key using HKDF-SHA256
pub fn derive_key_hkdf(
    ikm: &[u8],
    salt: &[u8],
    info: &[u8],
    output_len: usize,
) -> CryptoResult<SecureBytes> {
    let hkdf = Hkdf::new(salt.to_vec(), info.to_vec());
    hkdf.derive(ikm, output_len)
}

/// Derive a key using PBKDF2-HMAC-SHA256
pub fn derive_key_pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    output_len: usize,
) -> CryptoResult<SecureBytes> {
    let pbkdf2 = Pbkdf2::new(iterations, salt.to_vec());
    pbkdf2.derive(password, output_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_derive() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"application info";

        let hkdf = Hkdf::new(salt.to_vec(), info.to_vec());
        let derived = hkdf.derive(ikm, 32).unwrap();

        assert_eq!(derived.len(), 32);
    }

    #[test]
    fn test_hkdf_derive_key() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"application info";

        let hkdf = Hkdf::new(salt.to_vec(), info.to_vec());
        let key = hkdf.derive_key(ikm).unwrap();

        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_hkdf_deterministic() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"application info";

        let hkdf = Hkdf::new(salt.to_vec(), info.to_vec());
        let derived1 = hkdf.derive(ikm, 32).unwrap();
        let derived2 = hkdf.derive(ikm, 32).unwrap();

        assert_eq!(derived1.as_bytes(), derived2.as_bytes());
    }

    #[test]
    fn test_hkdf_different_info() {
        let ikm = b"input key material";
        let salt = b"salt";

        let hkdf1 = Hkdf::new(salt.to_vec(), b"info1".to_vec());
        let hkdf2 = Hkdf::new(salt.to_vec(), b"info2".to_vec());

        let derived1 = hkdf1.derive(ikm, 32).unwrap();
        let derived2 = hkdf2.derive(ikm, 32).unwrap();

        assert_ne!(derived1.as_bytes(), derived2.as_bytes());
    }

    #[test]
    fn test_pbkdf2_derive() {
        let password = b"my_password";
        let salt = b"random_salt";

        let pbkdf2 = Pbkdf2::new(10_000, salt.to_vec());
        let derived = pbkdf2.derive(password, 32).unwrap();

        assert_eq!(derived.len(), 32);
    }

    #[test]
    fn test_pbkdf2_derive_key() {
        let password = b"my_password";
        let salt = b"random_salt";

        let pbkdf2 = Pbkdf2::new(10_000, salt.to_vec());
        let key = pbkdf2.derive_key(password).unwrap();

        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_pbkdf2_deterministic() {
        let password = b"my_password";
        let salt = b"random_salt";

        let pbkdf2 = Pbkdf2::new(10_000, salt.to_vec());
        let derived1 = pbkdf2.derive(password, 32).unwrap();
        let derived2 = pbkdf2.derive(password, 32).unwrap();

        assert_eq!(derived1.as_bytes(), derived2.as_bytes());
    }

    #[test]
    fn test_pbkdf2_different_salt() {
        let password = b"my_password";

        let pbkdf2_1 = Pbkdf2::new(10_000, b"salt1".to_vec());
        let pbkdf2_2 = Pbkdf2::new(10_000, b"salt2".to_vec());

        let derived1 = pbkdf2_1.derive(password, 32).unwrap();
        let derived2 = pbkdf2_2.derive(password, 32).unwrap();

        assert_ne!(derived1.as_bytes(), derived2.as_bytes());
    }

    #[test]
    fn test_derive_key_hkdf() {
        let derived = derive_key_hkdf(b"ikm", b"salt", b"info", 32).unwrap();
        assert_eq!(derived.len(), 32);
    }

    #[test]
    fn test_derive_key_pbkdf2() {
        let derived = derive_key_pbkdf2(b"password", b"salt", 10_000, 32).unwrap();
        assert_eq!(derived.len(), 32);
    }
}
