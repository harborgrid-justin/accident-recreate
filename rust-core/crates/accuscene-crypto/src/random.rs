//! Cryptographically secure random number generation
//!
//! Provides secure random number generation using ChaCha20-based RNG.

use crate::error::{CryptoError, CryptoResult};
use crate::secure_memory::SecureBytes;
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// A cryptographically secure random number generator
pub struct SecureRng {
    rng: ChaCha20Rng,
}

impl SecureRng {
    /// Create a new secure RNG from OS entropy
    pub fn new() -> CryptoResult<Self> {
        Ok(Self {
            rng: ChaCha20Rng::from_entropy(),
        })
    }

    /// Create a new secure RNG from a seed (for deterministic testing)
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            rng: ChaCha20Rng::from_seed(seed),
        }
    }

    /// Generate random bytes
    pub fn generate_bytes(&mut self, len: usize) -> SecureBytes {
        let mut bytes = vec![0u8; len];
        self.rng.fill_bytes(&mut bytes);
        SecureBytes::new(bytes)
    }

    /// Generate a random u32
    pub fn generate_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    /// Generate a random u64
    pub fn generate_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Generate random bytes into an existing buffer
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest);
    }

    /// Generate a random value in the range [0, n)
    pub fn generate_range(&mut self, n: u64) -> u64 {
        use rand::Rng;
        self.rng.gen_range(0..n)
    }
}

impl Default for SecureRng {
    fn default() -> Self {
        Self::new().expect("Failed to create secure RNG")
    }
}

impl RngCore for SecureRng {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl CryptoRng for SecureRng {}

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(len: usize) -> CryptoResult<SecureBytes> {
    let mut rng = SecureRng::new()?;
    Ok(rng.generate_bytes(len))
}

/// Generate a random 128-bit (16 byte) value
pub fn generate_random_128() -> CryptoResult<[u8; 16]> {
    let mut rng = SecureRng::new()?;
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    Ok(bytes)
}

/// Generate a random 256-bit (32 byte) value
pub fn generate_random_256() -> CryptoResult<[u8; 32]> {
    let mut rng = SecureRng::new()?;
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    Ok(bytes)
}

/// Generate a random nonce for AES-GCM (96 bits / 12 bytes)
pub fn generate_aes_gcm_nonce() -> CryptoResult<[u8; 12]> {
    let mut rng = SecureRng::new()?;
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    Ok(nonce)
}

/// Generate a random nonce for ChaCha20-Poly1305 (96 bits / 12 bytes)
pub fn generate_chacha_nonce() -> CryptoResult<[u8; 12]> {
    let mut rng = SecureRng::new()?;
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    Ok(nonce)
}

/// Generate a random salt for key derivation (recommended 16 bytes minimum)
pub fn generate_salt(len: usize) -> CryptoResult<SecureBytes> {
    if len < 16 {
        return Err(CryptoError::InvalidSaltSize {
            expected: 16,
            actual: len,
        });
    }
    generate_random_bytes(len)
}

/// Generate a random alphanumeric string of specified length
pub fn generate_random_string(len: usize) -> CryptoResult<String> {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = SecureRng::new()?;
    let random_string: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    Ok(random_string)
}

/// Generate a random URL-safe base64 string of specified byte length
pub fn generate_random_base64(byte_len: usize) -> CryptoResult<String> {
    let bytes = generate_random_bytes(byte_len)?;
    Ok(base64::encode(bytes.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_rng_creation() {
        let rng = SecureRng::new();
        assert!(rng.is_ok());
    }

    #[test]
    fn test_generate_bytes() {
        let mut rng = SecureRng::new().unwrap();
        let bytes1 = rng.generate_bytes(32);
        let bytes2 = rng.generate_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        // Extremely unlikely to generate the same random bytes
        assert_ne!(bytes1.as_bytes(), bytes2.as_bytes());
    }

    #[test]
    fn test_generate_random_128() {
        let bytes1 = generate_random_128().unwrap();
        let bytes2 = generate_random_128().unwrap();
        assert_eq!(bytes1.len(), 16);
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_generate_random_256() {
        let bytes1 = generate_random_256().unwrap();
        let bytes2 = generate_random_256().unwrap();
        assert_eq!(bytes1.len(), 32);
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_generate_aes_gcm_nonce() {
        let nonce = generate_aes_gcm_nonce().unwrap();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt(16).unwrap();
        assert_eq!(salt.len(), 16);

        let invalid_salt = generate_salt(8);
        assert!(invalid_salt.is_err());
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(32).unwrap();
        let s2 = generate_random_string(32).unwrap();
        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2);
        assert!(s1.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_random_base64() {
        let s = generate_random_base64(32).unwrap();
        assert!(!s.is_empty());
        // Base64 encoding should work
        assert!(base64::decode(&s).is_ok());
    }

    #[test]
    fn test_from_seed_deterministic() {
        let seed = [42u8; 32];
        let mut rng1 = SecureRng::from_seed(seed);
        let mut rng2 = SecureRng::from_seed(seed);

        let bytes1 = rng1.generate_bytes(32);
        let bytes2 = rng2.generate_bytes(32);

        assert_eq!(bytes1.as_bytes(), bytes2.as_bytes());
    }
}
