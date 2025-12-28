//! Password hashing and verification
//!
//! Provides secure password hashing using Argon2id and scrypt.

use crate::error::{CryptoError, CryptoResult};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder, Version,
};
use serde::{Deserialize, Serialize};

/// Password hashing algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PasswordHashingAlgorithm {
    /// Argon2id (recommended)
    Argon2id,
    /// Scrypt (legacy support)
    Scrypt,
}

impl Default for PasswordHashingAlgorithm {
    fn default() -> Self {
        Self::Argon2id
    }
}

/// Password hasher configuration
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    algorithm: PasswordHashingAlgorithm,
    argon2_params: Argon2Params,
}

/// Argon2 parameters
#[derive(Debug, Clone)]
pub struct Argon2Params {
    /// Memory cost in KiB (default: 19456 = 19 MiB)
    pub memory_cost: u32,
    /// Time cost (iterations, default: 2)
    pub time_cost: u32,
    /// Parallelism (default: 1)
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_cost: 19456, // 19 MiB
            time_cost: 2,
            parallelism: 1,
        }
    }
}

impl PasswordHasher {
    /// Create a new password hasher with default settings
    pub fn new() -> Self {
        Self {
            algorithm: PasswordHashingAlgorithm::default(),
            argon2_params: Argon2Params::default(),
        }
    }

    /// Create a password hasher with a specific algorithm
    pub fn with_algorithm(algorithm: PasswordHashingAlgorithm) -> Self {
        Self {
            algorithm,
            argon2_params: Argon2Params::default(),
        }
    }

    /// Set Argon2 parameters
    pub fn with_argon2_params(mut self, params: Argon2Params) -> Self {
        self.argon2_params = params;
        self
    }

    /// Hash a password
    pub fn hash_password(&self, password: &str) -> CryptoResult<String> {
        match self.algorithm {
            PasswordHashingAlgorithm::Argon2id => self.hash_password_argon2(password),
            PasswordHashingAlgorithm::Scrypt => self.hash_password_scrypt(password),
        }
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> CryptoResult<bool> {
        // Try to parse the hash to determine the algorithm
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| CryptoError::PasswordVerificationFailed(e.to_string()))?;

        match parsed_hash.algorithm.as_str() {
            "argon2id" | "argon2i" | "argon2d" => self.verify_password_argon2(password, hash),
            "scrypt" => self.verify_password_scrypt(password, hash),
            _ => Err(CryptoError::PasswordVerificationFailed(
                "Unknown hash algorithm".to_string(),
            )),
        }
    }

    fn hash_password_argon2(&self, password: &str) -> CryptoResult<String> {
        let salt = SaltString::generate(&mut rand::thread_rng());

        let params = ParamsBuilder::new()
            .m_cost(self.argon2_params.memory_cost)
            .t_cost(self.argon2_params.time_cost)
            .p_cost(self.argon2_params.parallelism)
            .build()
            .map_err(|e| CryptoError::PasswordHashingFailed(e.to_string()))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::PasswordHashingFailed(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    fn verify_password_argon2(&self, password: &str, hash: &str) -> CryptoResult<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(CryptoError::PasswordVerificationFailed(e.to_string())),
        }
    }

    fn hash_password_scrypt(&self, password: &str) -> CryptoResult<String> {
        use scrypt::{
            password_hash::{PasswordHasher as _, SaltString},
            Scrypt,
        };

        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Scrypt
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CryptoError::PasswordHashingFailed(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    fn verify_password_scrypt(&self, password: &str, hash: &str) -> CryptoResult<bool> {
        use scrypt::{password_hash::PasswordVerifier, Scrypt};

        let parsed_hash = PasswordHash::new(hash)?;

        match Scrypt.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(CryptoError::PasswordVerificationFailed(e.to_string())),
        }
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a password using Argon2id with default parameters
pub fn hash_password(password: &str) -> CryptoResult<String> {
    PasswordHasher::new().hash_password(password)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> CryptoResult<bool> {
    PasswordHasher::new().verify_password(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "my_secure_password_123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_argon2id_hash_format() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();

        // Argon2id hash should start with $argon2id$
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = hash_password(password1).unwrap();
        let hash2 = hash_password(password2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_hashes() {
        // Same password should produce different hashes due to random salt
        let password = "test_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        assert_ne!(hash1, hash2);
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_custom_argon2_params() {
        let params = Argon2Params {
            memory_cost: 4096,
            time_cost: 1,
            parallelism: 1,
        };

        let hasher = PasswordHasher::new().with_argon2_params(params);
        let password = "test_password";
        let hash = hasher.hash_password(password).unwrap();

        assert!(hasher.verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_scrypt_algorithm() {
        let hasher = PasswordHasher::with_algorithm(PasswordHashingAlgorithm::Scrypt);
        let password = "test_password";
        let hash = hasher.hash_password(password).unwrap();

        assert!(hash.starts_with("$scrypt$"));
        assert!(hasher.verify_password(password, &hash).unwrap());
        assert!(!hasher.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_empty_password() {
        let password = "";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("not_empty", &hash).unwrap());
    }

    #[test]
    fn test_long_password() {
        let password = "a".repeat(1000);
        let hash = hash_password(&password).unwrap();
        assert!(verify_password(&password, &hash).unwrap());
    }
}
