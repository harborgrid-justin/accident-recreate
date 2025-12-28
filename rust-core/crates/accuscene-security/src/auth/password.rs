//! Password hashing and verification using Argon2id
//!
//! Implements secure password hashing following OWASP guidelines.

use crate::config::PasswordPolicy;
use crate::error::{Result, SecurityError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder, Version,
};
use zxcvbn::zxcvbn;

/// Password hasher using Argon2id
pub struct PasswordHashService {
    policy: PasswordPolicy,
}

impl PasswordHashService {
    /// Create a new password hash service
    pub fn new(policy: PasswordPolicy) -> Self {
        Self { policy }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String> {
        // Validate password strength first
        self.validate_password(password)?;

        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Configure Argon2id parameters
        let params = ParamsBuilder::new()
            .m_cost(19456) // 19 MiB memory
            .t_cost(2)     // 2 iterations
            .p_cost(1)     // 1 parallelism
            .build()
            .map_err(|e| SecurityError::Internal(format!("Argon2 params error: {}", e)))?;

        // Create Argon2 instance with params
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        // Hash password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SecurityError::Internal(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| SecurityError::Internal(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate password against policy
    pub fn validate_password(&self, password: &str) -> Result<()> {
        // Check length
        if password.len() < self.policy.min_length {
            return Err(SecurityError::WeakPassword(format!(
                "Password must be at least {} characters",
                self.policy.min_length
            )));
        }

        // Check character requirements
        if self.policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(SecurityError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if self.policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(SecurityError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if self.policy.require_digits && !password.chars().any(|c| c.is_numeric()) {
            return Err(SecurityError::WeakPassword(
                "Password must contain at least one digit".to_string(),
            ));
        }

        if self.policy.require_special
            && !password.chars().any(|c| !c.is_alphanumeric())
        {
            return Err(SecurityError::WeakPassword(
                "Password must contain at least one special character".to_string(),
            ));
        }

        // Check password strength using zxcvbn
        let strength = zxcvbn(password, &[])
            .map_err(|e| SecurityError::Internal(format!("Password strength check failed: {}", e)))?;

        if strength.score() < self.policy.min_strength_score {
            let feedback = strength
                .feedback()
                .as_ref()
                .and_then(|f| f.warning())
                .map(|w| w.to_string())
                .unwrap_or_else(|| "Password is too weak".to_string());

            return Err(SecurityError::WeakPassword(feedback));
        }

        Ok(())
    }

    /// Check if password needs rehashing (e.g., after policy change)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        // Parse the hash to check its parameters
        if let Ok(parsed_hash) = PasswordHash::new(hash) {
            // Check if algorithm is still Argon2id
            if parsed_hash.algorithm.as_str() != "argon2id" {
                return true;
            }

            // In production, you'd check specific parameters
            // For now, we'll keep existing hashes valid
            false
        } else {
            // Invalid hash format needs rehashing
            true
        }
    }

    /// Generate a secure random password
    pub fn generate_password(&self, length: usize) -> String {
        use rand::Rng;

        const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const DIGITS: &[u8] = b"0123456789";
        const SPECIAL: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

        let mut rng = rand::thread_rng();
        let mut password = String::new();

        // Ensure at least one of each required character type
        if self.policy.require_uppercase {
            password.push(UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char);
        }
        if self.policy.require_lowercase {
            password.push(LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char);
        }
        if self.policy.require_digits {
            password.push(DIGITS[rng.gen_range(0..DIGITS.len())] as char);
        }
        if self.policy.require_special {
            password.push(SPECIAL[rng.gen_range(0..SPECIAL.len())] as char);
        }

        // Fill remaining length
        let all_chars: Vec<u8> = UPPERCASE
            .iter()
            .chain(LOWERCASE.iter())
            .chain(DIGITS.iter())
            .chain(SPECIAL.iter())
            .copied()
            .collect();

        while password.len() < length {
            password.push(all_chars[rng.gen_range(0..all_chars.len())] as char);
        }

        // Shuffle the password
        let mut chars: Vec<char> = password.chars().collect();
        for i in (1..chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            chars.swap(i, j);
        }

        chars.into_iter().collect()
    }
}

/// Password history tracker to prevent password reuse
pub struct PasswordHistory {
    hashes: Vec<String>,
    max_history: usize,
}

impl PasswordHistory {
    /// Create a new password history
    pub fn new(max_history: usize) -> Self {
        Self {
            hashes: Vec::new(),
            max_history,
        }
    }

    /// Add a password hash to history
    pub fn add(&mut self, hash: String) {
        self.hashes.push(hash);
        if self.hashes.len() > self.max_history {
            self.hashes.remove(0);
        }
    }

    /// Check if password was used before
    pub fn is_reused(&self, password: &str, hasher: &PasswordHashService) -> Result<bool> {
        for hash in &self.hashes {
            if hasher.verify_password(password, hash)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get history size
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy() -> PasswordPolicy {
        PasswordPolicy {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
            min_strength_score: 3,
            expiry_days: Some(90),
            password_history: 5,
            max_failed_attempts: 5,
            lockout_duration_secs: 900,
        }
    }

    #[test]
    fn test_password_hashing() {
        let service = PasswordHashService::new(test_policy());
        let password = "SecureP@ssw0rd123!";

        let hash = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let service = PasswordHashService::new(test_policy());

        // Too short
        assert!(service.validate_password("Short1!").is_err());

        // Missing uppercase
        assert!(service.validate_password("lowercase123!").is_err());

        // Missing special
        assert!(service.validate_password("NoSpecial123").is_err());

        // Valid password
        assert!(service.validate_password("SecureP@ssw0rd123!").is_ok());
    }

    #[test]
    fn test_password_generation() {
        let service = PasswordHashService::new(test_policy());
        let password = service.generate_password(16);

        assert_eq!(password.len(), 16);
        assert!(service.validate_password(&password).is_ok());
    }

    #[test]
    fn test_password_history() {
        let service = PasswordHashService::new(test_policy());
        let mut history = PasswordHistory::new(3);

        let pwd1 = "FirstP@ssw0rd123!";
        let pwd2 = "SecondP@ssw0rd123!";

        let hash1 = service.hash_password(pwd1).unwrap();
        history.add(hash1);

        assert!(history.is_reused(pwd1, &service).unwrap());
        assert!(!history.is_reused(pwd2, &service).unwrap());
    }
}
