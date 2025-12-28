//! Secure memory handling with automatic zeroization
//!
//! Provides wrappers for sensitive data that are automatically zeroized on drop.

use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure byte buffer that is zeroized on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureBytes {
    #[zeroize(skip)]
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create a new secure byte buffer from a Vec
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create a new secure byte buffer from a slice
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Create a new secure byte buffer of a specific size filled with zeros
    pub fn zeros(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    /// Get a reference to the data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the data
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to a Vec (consumes self)
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    /// Securely compare two byte buffers in constant time
    pub fn secure_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.data.ct_eq(&other.data).into()
    }
}

impl AsRef<[u8]> for SecureBytes {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for SecureBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for SecureBytes {
    fn from(data: &[u8]) -> Self {
        Self::from_slice(data)
    }
}

// Don't implement Debug to avoid leaking sensitive data
impl fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureBytes([REDACTED {} bytes])", self.data.len())
    }
}

/// A secure string that is zeroized on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    /// Create a new secure string
    pub fn new(data: String) -> Self {
        Self { data }
    }

    /// Create a new secure string from a str slice
    pub fn from_str(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }

    /// Get a reference to the string
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Convert to a String (consumes self)
    pub fn into_string(self) -> String {
        self.data
    }

    /// Securely compare two strings in constant time
    pub fn secure_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.data.as_bytes().ct_eq(other.data.as_bytes()).into()
    }
}

impl From<String> for SecureString {
    fn from(data: String) -> Self {
        Self::new(data)
    }
}

impl From<&str> for SecureString {
    fn from(data: &str) -> Self {
        Self::from_str(data)
    }
}

// Don't implement Debug to avoid leaking sensitive data
impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureString([REDACTED {} chars])", self.data.len())
    }
}

/// A secure key that is zeroized on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct SecureKey<const N: usize> {
    #[serde(with = "serde_bytes")]
    data: [u8; N],
}

impl<const N: usize> SecureKey<N> {
    /// Create a new secure key from an array
    pub fn new(data: [u8; N]) -> Self {
        Self { data }
    }

    /// Create a new secure key from a slice
    pub fn from_slice(data: &[u8]) -> Result<Self, crate::error::CryptoError> {
        if data.len() != N {
            return Err(crate::error::CryptoError::InvalidKeySize {
                expected: N,
                actual: data.len(),
            });
        }
        let mut key = [0u8; N];
        key.copy_from_slice(data);
        Ok(Self { data: key })
    }

    /// Get a reference to the key bytes
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.data
    }

    /// Get a mutable reference to the key bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8; N] {
        &mut self.data
    }

    /// Convert to a byte array (consumes self)
    pub fn into_bytes(self) -> [u8; N] {
        self.data
    }

    /// Securely compare two keys in constant time
    pub fn secure_eq(&self, other: &Self) -> bool {
        use subtle::ConstantTimeEq;
        self.data.ct_eq(&other.data).into()
    }
}

impl<const N: usize> AsRef<[u8]> for SecureKey<N> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<const N: usize> AsMut<[u8]> for SecureKey<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

// Don't implement Debug to avoid leaking sensitive data
impl<const N: usize> fmt::Debug for SecureKey<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureKey<{}>([REDACTED])", N)
    }
}

/// Helper module for serde_bytes
mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, const N: usize>(data: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        vec.try_into()
            .map_err(|_| serde::de::Error::custom("invalid length"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let secure = SecureBytes::new(data.clone());
        assert_eq!(secure.as_bytes(), &data);
        assert_eq!(secure.len(), 5);
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_bytes_zeros() {
        let secure = SecureBytes::zeros(10);
        assert_eq!(secure.len(), 10);
        assert!(secure.as_bytes().iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secure_bytes_eq() {
        let secure1 = SecureBytes::new(vec![1, 2, 3]);
        let secure2 = SecureBytes::new(vec![1, 2, 3]);
        let secure3 = SecureBytes::new(vec![1, 2, 4]);
        assert!(secure1.secure_eq(&secure2));
        assert!(!secure1.secure_eq(&secure3));
    }

    #[test]
    fn test_secure_string() {
        let data = "secret password".to_string();
        let secure = SecureString::new(data.clone());
        assert_eq!(secure.as_str(), &data);
        assert_eq!(secure.len(), data.len());
        assert!(!secure.is_empty());
    }

    #[test]
    fn test_secure_key() {
        let data = [1u8; 32];
        let key = SecureKey::<32>::new(data);
        assert_eq!(key.as_bytes(), &data);
    }

    #[test]
    fn test_secure_key_from_slice() {
        let data = vec![1u8; 32];
        let key = SecureKey::<32>::from_slice(&data).unwrap();
        assert_eq!(key.as_bytes(), &[1u8; 32]);

        let invalid_data = vec![1u8; 16];
        let result = SecureKey::<32>::from_slice(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let secure = SecureBytes::new(vec![1, 2, 3]);
        let debug_str = format!("{:?}", secure);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("1"));
    }
}
