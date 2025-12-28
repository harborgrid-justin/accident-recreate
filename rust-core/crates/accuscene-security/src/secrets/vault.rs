//! Secure vault for secrets storage

use crate::encryption::AtRestEncryption;
use crate::error::{Result, SecurityError};
use std::collections::HashMap;

/// Secure vault for storing secrets
pub struct SecretVault {
    secrets: HashMap<String, EncryptedSecret>,
    encryption: AtRestEncryption,
    sealed: bool,
}

impl SecretVault {
    /// Create a new vault
    pub fn new(master_key: &[u8; 32]) -> Self {
        Self {
            secrets: HashMap::new(),
            encryption: AtRestEncryption::new(master_key),
            sealed: false,
        }
    }

    /// Store a secret
    pub fn store(&mut self, key: String, value: String) -> Result<()> {
        if self.sealed {
            return Err(SecurityError::VaultSealed);
        }

        let encrypted = self.encryption.encrypt_string(&value)?;
        let secret = EncryptedSecret {
            key: key.clone(),
            encrypted_value: encrypted,
            created_at: chrono::Utc::now(),
        };

        self.secrets.insert(key, secret);
        Ok(())
    }

    /// Retrieve a secret
    pub fn get(&self, key: &str) -> Result<String> {
        if self.sealed {
            return Err(SecurityError::VaultSealed);
        }

        let secret = self
            .secrets
            .get(key)
            .ok_or_else(|| SecurityError::SecretNotFound(key.to_string()))?;

        self.encryption.decrypt_string(&secret.encrypted_value)
    }

    /// Delete a secret
    pub fn delete(&mut self, key: &str) -> Result<()> {
        if self.sealed {
            return Err(SecurityError::VaultSealed);
        }

        self.secrets.remove(key);
        Ok(())
    }

    /// Seal the vault
    pub fn seal(&mut self) {
        self.sealed = true;
    }

    /// Unseal the vault
    pub fn unseal(&mut self) {
        self.sealed = false;
    }
}

/// Encrypted secret
#[derive(Debug, Clone)]
struct EncryptedSecret {
    key: String,
    encrypted_value: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn test_store_retrieve() {
        let mut vault = SecretVault::new(&test_key());
        vault.store("api_key".to_string(), "secret123".to_string()).unwrap();

        let retrieved = vault.get("api_key").unwrap();
        assert_eq!(retrieved, "secret123");
    }

    #[test]
    fn test_sealed_vault() {
        let mut vault = SecretVault::new(&test_key());
        vault.seal();

        assert!(vault.store("key".to_string(), "value".to_string()).is_err());
    }
}
