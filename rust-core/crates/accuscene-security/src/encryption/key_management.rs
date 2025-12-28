//! Encryption key management and rotation

use crate::error::{Result, SecurityError};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key management service
pub struct KeyManagementService {
    keys: HashMap<String, EncryptionKey>,
    rotation_days: u32,
}

impl KeyManagementService {
    pub fn new(rotation_days: u32) -> Self {
        Self {
            keys: HashMap::new(),
            rotation_days,
        }
    }

    /// Generate a new encryption key
    pub fn generate_key(&mut self, key_id: String, purpose: KeyPurpose) -> Result<EncryptionKey> {
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);

        let key = EncryptionKey {
            id: key_id.clone(),
            key_bytes: key_bytes.to_vec(),
            purpose,
            created_at: chrono::Utc::now(),
            expires_at: None,
            status: KeyStatus::Active,
        };

        self.keys.insert(key_id, key.clone());
        Ok(key)
    }

    /// Get active key
    pub fn get_key(&self, key_id: &str) -> Result<&EncryptionKey> {
        self.keys
            .get(key_id)
            .filter(|k| matches!(k.status, KeyStatus::Active))
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))
    }

    /// Rotate key
    pub fn rotate_key(&mut self, old_key_id: &str) -> Result<EncryptionKey> {
        // Mark old key as rotated
        if let Some(old_key) = self.keys.get_mut(old_key_id) {
            old_key.status = KeyStatus::Rotated;
        }

        // Generate new key with same purpose
        let purpose = self
            .keys
            .get(old_key_id)
            .map(|k| k.purpose)
            .unwrap_or(KeyPurpose::DataEncryption);

        let new_key_id = format!("{}-v2", old_key_id);
        self.generate_key(new_key_id, purpose)
    }

    /// Check if key needs rotation
    pub fn needs_rotation(&self, key_id: &str) -> bool {
        if let Some(key) = self.keys.get(key_id) {
            let age = chrono::Utc::now()
                .signed_duration_since(key.created_at)
                .num_days() as u32;
            age >= self.rotation_days
        } else {
            false
        }
    }
}

/// Encryption key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: String,
    #[serde(skip_serializing)]
    pub key_bytes: Vec<u8>,
    pub purpose: KeyPurpose,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: KeyStatus,
}

/// Key purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    DataEncryption,
    TokenSigning,
    SecretEncryption,
}

/// Key status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Rotated,
    Revoked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let mut kms = KeyManagementService::new(90);
        let key = kms
            .generate_key("test-key".to_string(), KeyPurpose::DataEncryption)
            .unwrap();

        assert_eq!(key.key_bytes.len(), 32);
        assert_eq!(key.purpose, KeyPurpose::DataEncryption);
    }

    #[test]
    fn test_key_rotation() {
        let mut kms = KeyManagementService::new(90);
        kms.generate_key("key-1".to_string(), KeyPurpose::DataEncryption)
            .unwrap();

        let rotated = kms.rotate_key("key-1").unwrap();
        assert!(rotated.id.contains("-v2"));

        let old_key = kms.keys.get("key-1").unwrap();
        assert_eq!(old_key.status, KeyStatus::Rotated);
    }
}
