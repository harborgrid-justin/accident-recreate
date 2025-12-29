//! Cryptographically secure random number generation

use crate::error::{SecurityError, SecurityResult};
use getrandom::getrandom;

/// Secure random number generator
pub struct SecureRandom;

impl SecureRandom {
    /// Generate random bytes
    pub fn bytes(length: usize) -> SecurityResult<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        getrandom(&mut buffer)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate random bytes: {}", e)))?;

        Ok(buffer)
    }

    /// Generate a random u32
    pub fn u32() -> SecurityResult<u32> {
        let mut bytes = [0u8; 4];
        getrandom(&mut bytes)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate random u32: {}", e)))?;

        Ok(u32::from_le_bytes(bytes))
    }

    /// Generate a random u64
    pub fn u64() -> SecurityResult<u64> {
        let mut bytes = [0u8; 8];
        getrandom(&mut bytes)
            .map_err(|e| SecurityError::CryptoError(format!("Failed to generate random u64: {}", e)))?;

        Ok(u64::from_le_bytes(bytes))
    }

    /// Generate a random number in range [0, max)
    pub fn range(max: u32) -> SecurityResult<u32> {
        if max == 0 {
            return Ok(0);
        }

        // Use rejection sampling to avoid modulo bias
        let range = max;
        let limit = u32::MAX - (u32::MAX % range);

        loop {
            let num = Self::u32()?;
            if num < limit {
                return Ok(num % range);
            }
        }
    }

    /// Generate a random alphanumeric string
    pub fn alphanumeric(length: usize) -> SecurityResult<String> {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut result = String::with_capacity(length);

        for _ in 0..length {
            let idx = Self::range(CHARSET.len() as u32)? as usize;
            result.push(CHARSET[idx] as char);
        }

        Ok(result)
    }

    /// Generate a random hex string
    pub fn hex(length: usize) -> SecurityResult<String> {
        let bytes = Self::bytes(length / 2 + length % 2)?;
        let hex = hex::encode(bytes);

        Ok(hex[..length].to_string())
    }

    /// Generate a random token (URL-safe base64)
    pub fn token(byte_length: usize) -> SecurityResult<String> {
        let bytes = Self::bytes(byte_length)?;
        Ok(base64::encode_config(
            bytes,
            base64::URL_SAFE_NO_PAD,
        ))
    }

    /// Generate a random UUID v4
    pub fn uuid() -> SecurityResult<String> {
        let bytes = Self::bytes(16)?;

        // Set version (4) and variant (RFC4122)
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&bytes);
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40; // Version 4
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80; // Variant RFC4122

        Ok(format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3],
            uuid_bytes[4], uuid_bytes[5],
            uuid_bytes[6], uuid_bytes[7],
            uuid_bytes[8], uuid_bytes[9],
            uuid_bytes[10], uuid_bytes[11], uuid_bytes[12], uuid_bytes[13], uuid_bytes[14], uuid_bytes[15]
        ))
    }

    /// Shuffle a slice using Fisher-Yates algorithm
    pub fn shuffle<T>(slice: &mut [T]) -> SecurityResult<()> {
        let len = slice.len();
        for i in (1..len).rev() {
            let j = Self::range(i as u32 + 1)? as usize;
            slice.swap(i, j);
        }
        Ok(())
    }

    /// Choose a random element from a slice
    pub fn choose<T>(slice: &[T]) -> SecurityResult<&T> {
        if slice.is_empty() {
            return Err(SecurityError::InvalidInput("Cannot choose from empty slice".to_string()));
        }

        let idx = Self::range(slice.len() as u32)? as usize;
        Ok(&slice[idx])
    }

    /// Sample n random elements from a slice without replacement
    pub fn sample<T: Clone>(slice: &[T], n: usize) -> SecurityResult<Vec<T>> {
        if n > slice.len() {
            return Err(SecurityError::InvalidInput(
                "Sample size cannot exceed slice length".to_string(),
            ));
        }

        let mut indices: Vec<usize> = (0..slice.len()).collect();
        Self::shuffle(&mut indices)?;

        Ok(indices[..n].iter().map(|&i| slice[i].clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes() {
        let bytes1 = SecureRandom::bytes(32).unwrap();
        let bytes2 = SecureRandom::bytes(32).unwrap();

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_random_u32() {
        let num1 = SecureRandom::u32().unwrap();
        let num2 = SecureRandom::u32().unwrap();

        // Very unlikely to be equal
        assert_ne!(num1, num2);
    }

    #[test]
    fn test_random_range() {
        for _ in 0..100 {
            let num = SecureRandom::range(10).unwrap();
            assert!(num < 10);
        }
    }

    #[test]
    fn test_alphanumeric() {
        let s = SecureRandom::alphanumeric(20).unwrap();
        assert_eq!(s.len(), 20);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_hex() {
        let s = SecureRandom::hex(32).unwrap();
        assert_eq!(s.len(), 32);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_token() {
        let token = SecureRandom::token(32).unwrap();
        assert!(!token.is_empty());
        assert!(!token.contains('=')); // URL-safe, no padding
    }

    #[test]
    fn test_uuid() {
        let uuid = SecureRandom::uuid().unwrap();
        assert_eq!(uuid.len(), 36); // Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        assert!(uuid.contains('-'));

        // Check version 4
        let version_char = uuid.chars().nth(14).unwrap();
        assert_eq!(version_char, '4');
    }

    #[test]
    fn test_shuffle() {
        let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let original = data.clone();

        SecureRandom::shuffle(&mut data).unwrap();

        // Should have same elements but likely different order
        assert_eq!(data.len(), original.len());
        for &item in &original {
            assert!(data.contains(&item));
        }
    }

    #[test]
    fn test_choose() {
        let data = vec![1, 2, 3, 4, 5];
        let chosen = SecureRandom::choose(&data).unwrap();
        assert!(data.contains(chosen));
    }

    #[test]
    fn test_choose_empty() {
        let data: Vec<i32> = vec![];
        let result = SecureRandom::choose(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_sample() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let sample = SecureRandom::sample(&data, 5).unwrap();

        assert_eq!(sample.len(), 5);

        // All sampled items should be in original
        for item in &sample {
            assert!(data.contains(item));
        }

        // No duplicates
        let mut sorted = sample.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), sample.len());
    }

    #[test]
    fn test_sample_too_large() {
        let data = vec![1, 2, 3];
        let result = SecureRandom::sample(&data, 5);
        assert!(result.is_err());
    }
}
