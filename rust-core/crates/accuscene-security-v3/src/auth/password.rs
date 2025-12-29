//! Password hashing with Argon2id
//!
//! Uses Argon2id (winner of the Password Hashing Competition) for secure password storage.
//! Provides protection against:
//! - Brute force attacks
//! - Rainbow table attacks
//! - Side-channel attacks (timing)

use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2, ParamsBuilder, Version,
};
use crate::config::PasswordPolicy;
use crate::error::{SecurityError, SecurityResult};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Password hasher using Argon2id
#[derive(Debug)]
pub struct PasswordHasher {
    /// Argon2 instance
    argon2: Argon2<'static>,
    /// Password policy
    policy: PasswordPolicy,
}

impl PasswordHasher {
    /// Create a new password hasher with default settings
    pub fn new() -> Self {
        Self::with_policy(PasswordPolicy::default())
    }

    /// Create a password hasher with custom policy
    pub fn with_policy(policy: PasswordPolicy) -> Self {
        let params = ParamsBuilder::new()
            .m_cost(policy.argon2_memory_cost)
            .t_cost(policy.argon2_time_cost)
            .p_cost(policy.argon2_parallelism)
            .build()
            .expect("Invalid Argon2 parameters");

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        Self { argon2, policy }
    }

    /// Hash a password securely
    ///
    /// # Security
    ///
    /// - Uses Argon2id with recommended parameters
    /// - Automatically generates a cryptographically secure salt
    /// - Memory is zeroed after use
    pub fn hash(&self, password: &str) -> SecurityResult<String> {
        // Validate password against policy
        self.policy.validate(password).map_err(|e| {
            SecurityError::InvalidInput(e)
        })?;

        // Use Zeroizing to ensure password is cleared from memory
        let password = Zeroizing::new(password.as_bytes().to_vec());

        // Generate a random salt
        let salt = SaltString::generate(&mut rand::thread_rng());

        // Hash the password
        let hash = self
            .argon2
            .hash_password(&password, &salt)
            .map_err(|e| {
                SecurityError::CryptoError(format!("Password hashing failed: {}", e))
            })?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// # Security
    ///
    /// - Uses constant-time comparison to prevent timing attacks
    /// - Memory is zeroed after use
    pub fn verify(&self, password: &str, hash: &str) -> SecurityResult<bool> {
        // Use Zeroizing to ensure password is cleared from memory
        let password = Zeroizing::new(password.as_bytes().to_vec());

        // Parse the hash
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            SecurityError::InvalidInput(format!("Invalid password hash: {}", e))
        })?;

        // Verify the password
        match self.argon2.verify_password(&password, &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(SecurityError::CryptoError(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }

    /// Check if a hash needs to be rehashed (due to updated parameters)
    pub fn needs_rehash(&self, hash: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(hash) {
            if let Some(params) = parsed_hash.params.as_ref() {
                let current_m_cost = params.m_cost().unwrap_or(0);
                let current_t_cost = params.t_cost().unwrap_or(0);
                let current_p_cost = params.p_cost().unwrap_or(0);

                return current_m_cost != self.policy.argon2_memory_cost
                    || current_t_cost != self.policy.argon2_time_cost
                    || current_p_cost != self.policy.argon2_parallelism;
            }
        }
        false
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant-time string comparison to prevent timing attacks
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Password strength estimator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// Very weak password
    VeryWeak,
    /// Weak password
    Weak,
    /// Medium strength password
    Medium,
    /// Strong password
    Strong,
    /// Very strong password
    VeryStrong,
}

impl PasswordStrength {
    /// Estimate password strength based on entropy
    pub fn estimate(password: &str) -> Self {
        let mut score = 0;

        // Length bonus
        score += password.len() * 4;

        // Character diversity
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if has_lowercase {
            score += 10;
        }
        if has_uppercase {
            score += 10;
        }
        if has_digit {
            score += 10;
        }
        if has_special {
            score += 20;
        }

        // Penalty for common patterns
        if password.chars().collect::<Vec<_>>().windows(3).any(|w| {
            w[0] == w[1] && w[1] == w[2]
        }) {
            score -= 10;
        }

        // Check for sequential characters
        if is_sequential(password) {
            score -= 15;
        }

        match score {
            0..=30 => Self::VeryWeak,
            31..=50 => Self::Weak,
            51..=70 => Self::Medium,
            71..=90 => Self::Strong,
            _ => Self::VeryStrong,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::VeryWeak => "Very weak - not recommended",
            Self::Weak => "Weak - consider using a stronger password",
            Self::Medium => "Medium - acceptable but could be stronger",
            Self::Strong => "Strong - good password",
            Self::VeryStrong => "Very strong - excellent password",
        }
    }
}

/// Check if password contains sequential characters
fn is_sequential(password: &str) -> bool {
    let bytes = password.as_bytes();
    for window in bytes.windows(3) {
        if window[1] == window[0] + 1 && window[2] == window[1] + 1 {
            return true;
        }
        if window[1] + 1 == window[0] && window[2] + 1 == window[1] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "SecurePassword123!";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$argon2id$"));

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_policy_validation() {
        let hasher = PasswordHasher::new();

        // Too short
        assert!(hasher.hash("Short1!").is_err());

        // Missing uppercase
        assert!(hasher.hash("lowercase123!").is_err());

        // Missing digit
        assert!(hasher.hash("NoDigits!").is_err());

        // Valid password
        assert!(hasher.hash("ValidPassword123!").is_ok());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello!"));
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(
            PasswordStrength::estimate("weak"),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            PasswordStrength::estimate("Weak123"),
            PasswordStrength::Weak
        );
        assert_eq!(
            PasswordStrength::estimate("Medium123!"),
            PasswordStrength::Medium
        );
        assert_eq!(
            PasswordStrength::estimate("StrongPassword123!"),
            PasswordStrength::Strong
        );
        assert_eq!(
            PasswordStrength::estimate("VeryStrongP@ssw0rd!2024"),
            PasswordStrength::VeryStrong
        );
    }

    #[test]
    fn test_sequential_detection() {
        assert!(is_sequential("abc123"));
        assert!(is_sequential("password123"));
        assert!(!is_sequential("p@ssw0rd"));
    }

    #[test]
    fn test_needs_rehash() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("SecurePassword123!").unwrap();

        assert!(!hasher.needs_rehash(&hash));

        // Create hasher with different parameters
        let mut policy = PasswordPolicy::default();
        policy.argon2_memory_cost = 32768;
        let hasher2 = PasswordHasher::with_policy(policy);

        assert!(hasher2.needs_rehash(&hash));
    }
}
