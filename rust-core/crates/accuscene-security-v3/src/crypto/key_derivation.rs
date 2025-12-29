//! Key derivation functions (HKDF, PBKDF2)
//!
//! Provides secure key derivation for cryptographic operations.

use crate::error::{SecurityError, SecurityResult};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroizing;

/// Key derivation utilities
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key using HKDF (HMAC-based Extract-and-Expand Key Derivation Function)
    ///
    /// HKDF is suitable for deriving multiple keys from a single source key.
    pub fn hkdf(
        input_key_material: &[u8],
        salt: Option<&[u8]>,
        info: &[u8],
        output_length: usize,
    ) -> SecurityResult<DerivedKey> {
        let hk = Hkdf::<Sha256>::new(salt, input_key_material);

        let mut output = Zeroizing::new(vec![0u8; output_length]);
        hk.expand(info, &mut output)
            .map_err(|e| SecurityError::KeyDerivationFailed)?;

        Ok(DerivedKey { key: output })
    }

    /// Derive a key using PBKDF2 (Password-Based Key Derivation Function 2)
    ///
    /// PBKDF2 is suitable for deriving keys from passwords.
    pub fn pbkdf2(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        output_length: usize,
    ) -> SecurityResult<DerivedKey> {
        if iterations < 10000 {
            return Err(SecurityError::InvalidInput(
                "PBKDF2 iterations should be at least 10000".to_string(),
            ));
        }

        let mut output = Zeroizing::new(vec![0u8; output_length]);
        pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut output);

        Ok(DerivedKey { key: output })
    }

    /// Derive multiple keys from a master key using HKDF
    pub fn derive_multiple(
        master_key: &[u8],
        salt: Option<&[u8]>,
        contexts: &[&str],
        key_length: usize,
    ) -> SecurityResult<Vec<DerivedKey>> {
        let mut keys = Vec::new();

        for context in contexts {
            let key = Self::hkdf(master_key, salt, context.as_bytes(), key_length)?;
            keys.push(key);
        }

        Ok(keys)
    }

    /// Generate a salt for key derivation
    pub fn generate_salt(length: usize) -> SecurityResult<Vec<u8>> {
        let mut salt = vec![0u8; length];
        getrandom::getrandom(&mut salt)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate salt: {}", e)))?;

        Ok(salt)
    }
}

/// Derived key with automatic zeroization
#[derive(Debug, Clone)]
pub struct DerivedKey {
    key: Zeroizing<Vec<u8>>,
}

impl DerivedKey {
    /// Get the key bytes (use with caution!)
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }

    /// Get the key length
    pub fn len(&self) -> usize {
        self.key.len()
    }

    /// Check if key is empty
    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }

    /// Convert to Vec (consuming the key)
    pub fn into_vec(self) -> Zeroizing<Vec<u8>> {
        self.key
    }

    /// Encode to base64
    pub fn to_base64(&self) -> String {
        base64::encode(&self.key)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> SecurityResult<Self> {
        let bytes = base64::decode(encoded)
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid base64: {}", e)))?;

        Ok(Self {
            key: Zeroizing::new(bytes),
        })
    }
}

/// Key hierarchy for envelope encryption
pub struct KeyHierarchy {
    master_key: Zeroizing<Vec<u8>>,
}

impl KeyHierarchy {
    /// Create a new key hierarchy with a master key
    pub fn new(master_key: Vec<u8>) -> Self {
        Self {
            master_key: Zeroizing::new(master_key),
        }
    }

    /// Generate a new random master key
    pub fn generate(key_length: usize) -> SecurityResult<Self> {
        let mut master_key = vec![0u8; key_length];
        getrandom::getrandom(&mut master_key)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate master key: {}", e)))?;

        Ok(Self {
            master_key: Zeroizing::new(master_key),
        })
    }

    /// Derive a data encryption key
    pub fn derive_dek(&self, context: &str) -> SecurityResult<DerivedKey> {
        KeyDerivation::hkdf(
            &self.master_key,
            None,
            format!("dek:{}", context).as_bytes(),
            32, // 256 bits
        )
    }

    /// Derive a key encryption key
    pub fn derive_kek(&self, context: &str) -> SecurityResult<DerivedKey> {
        KeyDerivation::hkdf(
            &self.master_key,
            None,
            format!("kek:{}", context).as_bytes(),
            32, // 256 bits
        )
    }

    /// Derive a signing key
    pub fn derive_signing_key(&self, context: &str) -> SecurityResult<DerivedKey> {
        KeyDerivation::hkdf(
            &self.master_key,
            None,
            format!("sign:{}", context).as_bytes(),
            32, // 256 bits
        )
    }

    /// Rotate the master key (re-encrypt with new master key)
    pub fn rotate(&mut self) -> SecurityResult<Zeroizing<Vec<u8>>> {
        let old_key = self.master_key.clone();

        let mut new_key = vec![0u8; self.master_key.len()];
        getrandom::getrandom(&mut new_key)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate new master key: {}", e)))?;

        self.master_key = Zeroizing::new(new_key);

        Ok(old_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"context";

        let key = KeyDerivation::hkdf(ikm, Some(salt), info, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hkdf_deterministic() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"context";

        let key1 = KeyDerivation::hkdf(ikm, Some(salt), info, 32).unwrap();
        let key2 = KeyDerivation::hkdf(ikm, Some(salt), info, 32).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_hkdf_different_contexts() {
        let ikm = b"input key material";
        let salt = b"salt";

        let key1 = KeyDerivation::hkdf(ikm, Some(salt), b"context1", 32).unwrap();
        let key2 = KeyDerivation::hkdf(ikm, Some(salt), b"context2", 32).unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_pbkdf2() {
        let password = b"password";
        let salt = b"salt";

        let key = KeyDerivation::pbkdf2(password, salt, 10000, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_pbkdf2_min_iterations() {
        let password = b"password";
        let salt = b"salt";

        let result = KeyDerivation::pbkdf2(password, salt, 100, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_multiple() {
        let master_key = b"master key material";
        let contexts = vec!["encryption", "signing", "authentication"];

        let keys = KeyDerivation::derive_multiple(master_key, None, &contexts, 32).unwrap();
        assert_eq!(keys.len(), 3);

        // All keys should be different
        assert_ne!(keys[0].as_bytes(), keys[1].as_bytes());
        assert_ne!(keys[1].as_bytes(), keys[2].as_bytes());
        assert_ne!(keys[0].as_bytes(), keys[2].as_bytes());
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = KeyDerivation::generate_salt(16).unwrap();
        let salt2 = KeyDerivation::generate_salt(16).unwrap();

        assert_eq!(salt1.len(), 16);
        assert_eq!(salt2.len(), 16);
        assert_ne!(salt1, salt2); // Should be random
    }

    #[test]
    fn test_derived_key_base64() {
        let key = KeyDerivation::hkdf(b"input", None, b"info", 32).unwrap();

        let encoded = key.to_base64();
        let decoded = DerivedKey::from_base64(&encoded).unwrap();

        assert_eq!(key.as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_key_hierarchy() {
        let hierarchy = KeyHierarchy::generate(32).unwrap();

        let dek1 = hierarchy.derive_dek("user123").unwrap();
        let dek2 = hierarchy.derive_dek("user456").unwrap();
        let kek = hierarchy.derive_kek("organization").unwrap();

        assert_ne!(dek1.as_bytes(), dek2.as_bytes());
        assert_ne!(dek1.as_bytes(), kek.as_bytes());
    }

    #[test]
    fn test_key_hierarchy_deterministic() {
        let hierarchy = KeyHierarchy::generate(32).unwrap();

        let key1 = hierarchy.derive_dek("user123").unwrap();
        let key2 = hierarchy.derive_dek("user123").unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_key_rotation() {
        let mut hierarchy = KeyHierarchy::generate(32).unwrap();

        let key_before = hierarchy.derive_dek("user123").unwrap();
        let old_master = hierarchy.rotate().unwrap();
        let key_after = hierarchy.derive_dek("user123").unwrap();

        assert_ne!(key_before.as_bytes(), key_after.as_bytes());
    }
}
