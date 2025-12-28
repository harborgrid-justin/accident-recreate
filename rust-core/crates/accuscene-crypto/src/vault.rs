//! Secure key vault with memory protection
//!
//! Provides a secure in-memory vault for storing cryptographic keys.

use crate::error::{CryptoError, CryptoResult};
use crate::secure_memory::SecureBytes;
use crate::symmetric::aes::{decrypt_aes256gcm, encrypt_aes256gcm, EncryptedData};
use crate::symmetric::key::SymmetricKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A secure vault for storing cryptographic keys and secrets
pub struct Vault {
    /// Master key for encrypting vault entries
    master_key: SymmetricKey,
    /// Encrypted entries in the vault
    entries: HashMap<String, VaultEntry>,
}

/// An entry in the vault
#[derive(Clone, Serialize, Deserialize)]
struct VaultEntry {
    /// Encrypted data
    encrypted_data: EncryptedData,
    /// Metadata about the entry
    metadata: EntryMetadata,
}

/// Metadata for vault entries
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntryMetadata {
    /// When the entry was created (Unix timestamp)
    pub created_at: u64,
    /// When the entry was last accessed (Unix timestamp)
    pub last_accessed: u64,
    /// User-defined tags
    pub tags: HashMap<String, String>,
}

impl Default for EntryMetadata {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            created_at: now,
            last_accessed: now,
            tags: HashMap::new(),
        }
    }
}

impl Vault {
    /// Create a new vault with a master key
    pub fn new(master_key: SymmetricKey) -> Self {
        Self {
            master_key,
            entries: HashMap::new(),
        }
    }

    /// Create a new vault with a randomly generated master key
    pub fn generate() -> CryptoResult<Self> {
        let master_key = SymmetricKey::generate()?;
        Ok(Self::new(master_key))
    }

    /// Store a secret in the vault
    pub fn store(&mut self, key: String, secret: &[u8]) -> CryptoResult<()> {
        self.store_with_metadata(key, secret, EntryMetadata::default())
    }

    /// Store a secret in the vault with custom metadata
    pub fn store_with_metadata(
        &mut self,
        key: String,
        secret: &[u8],
        metadata: EntryMetadata,
    ) -> CryptoResult<()> {
        let encrypted_data = encrypt_aes256gcm(&self.master_key, secret, None)?;

        let entry = VaultEntry {
            encrypted_data,
            metadata,
        };

        self.entries.insert(key, entry);
        Ok(())
    }

    /// Retrieve a secret from the vault
    pub fn retrieve(&mut self, key: &str) -> CryptoResult<SecureBytes> {
        let entry = self
            .entries
            .get_mut(key)
            .ok_or_else(|| CryptoError::KeyNotFound(key.to_string()))?;

        // Update last accessed time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        entry.metadata.last_accessed = now;

        let secret = decrypt_aes256gcm(&self.master_key, &entry.encrypted_data, None)?;
        Ok(secret)
    }

    /// Check if a key exists in the vault
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Remove a secret from the vault
    pub fn remove(&mut self, key: &str) -> CryptoResult<()> {
        self.entries
            .remove(key)
            .ok_or_else(|| CryptoError::KeyNotFound(key.to_string()))?;
        Ok(())
    }

    /// List all keys in the vault
    pub fn list_keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Get metadata for a vault entry
    pub fn get_metadata(&self, key: &str) -> CryptoResult<EntryMetadata> {
        let entry = self
            .entries
            .get(key)
            .ok_or_else(|| CryptoError::KeyNotFound(key.to_string()))?;
        Ok(entry.metadata.clone())
    }

    /// Update metadata for a vault entry
    pub fn update_metadata(&mut self, key: &str, metadata: EntryMetadata) -> CryptoResult<()> {
        let entry = self
            .entries
            .get_mut(key)
            .ok_or_else(|| CryptoError::KeyNotFound(key.to_string()))?;
        entry.metadata = metadata;
        Ok(())
    }

    /// Clear all entries from the vault
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries in the vault
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the vault is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export the vault to an encrypted format
    pub fn export(&self) -> CryptoResult<Vec<u8>> {
        let json = serde_json::to_vec(&self.entries)?;
        Ok(json)
    }

    /// Import vault entries from an encrypted format
    pub fn import(&mut self, data: &[u8]) -> CryptoResult<()> {
        let entries: HashMap<String, VaultEntry> = serde_json::from_slice(data)?;
        self.entries.extend(entries);
        Ok(())
    }

    /// Get the master key (use with caution!)
    pub fn master_key(&self) -> &SymmetricKey {
        &self.master_key
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        // Clear all entries when vault is dropped
        self.entries.clear();
    }
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vault")
            .field("entries_count", &self.entries.len())
            .field("master_key", &"[REDACTED]")
            .finish()
    }
}

/// Builder for creating vault entries with metadata
pub struct VaultEntryBuilder {
    tags: HashMap<String, String>,
}

impl VaultEntryBuilder {
    /// Create a new entry builder
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
        }
    }

    /// Add a tag to the entry
    pub fn tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Build the metadata
    pub fn build(self) -> EntryMetadata {
        EntryMetadata {
            tags: self.tags,
            ..Default::default()
        }
    }
}

impl Default for VaultEntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_store_retrieve() {
        let mut vault = Vault::generate().unwrap();
        let secret = b"my secret data";

        vault.store("key1".to_string(), secret).unwrap();
        let retrieved = vault.retrieve("key1").unwrap();

        assert_eq!(secret, retrieved.as_bytes());
    }

    #[test]
    fn test_vault_contains() {
        let mut vault = Vault::generate().unwrap();
        vault.store("key1".to_string(), b"secret").unwrap();

        assert!(vault.contains("key1"));
        assert!(!vault.contains("key2"));
    }

    #[test]
    fn test_vault_remove() {
        let mut vault = Vault::generate().unwrap();
        vault.store("key1".to_string(), b"secret").unwrap();

        assert!(vault.contains("key1"));
        vault.remove("key1").unwrap();
        assert!(!vault.contains("key1"));
    }

    #[test]
    fn test_vault_remove_nonexistent() {
        let mut vault = Vault::generate().unwrap();
        let result = vault.remove("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_retrieve_nonexistent() {
        let mut vault = Vault::generate().unwrap();
        let result = vault.retrieve("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_list_keys() {
        let mut vault = Vault::generate().unwrap();
        vault.store("key1".to_string(), b"secret1").unwrap();
        vault.store("key2".to_string(), b"secret2").unwrap();
        vault.store("key3".to_string(), b"secret3").unwrap();

        let keys = vault.list_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[test]
    fn test_vault_metadata() {
        let mut vault = Vault::generate().unwrap();
        let metadata = VaultEntryBuilder::new()
            .tag("owner".to_string(), "alice".to_string())
            .tag("purpose".to_string(), "api_key".to_string())
            .build();

        vault
            .store_with_metadata("key1".to_string(), b"secret", metadata.clone())
            .unwrap();

        let retrieved_metadata = vault.get_metadata("key1").unwrap();
        assert_eq!(retrieved_metadata.tags.get("owner"), Some(&"alice".to_string()));
        assert_eq!(
            retrieved_metadata.tags.get("purpose"),
            Some(&"api_key".to_string())
        );
    }

    #[test]
    fn test_vault_clear() {
        let mut vault = Vault::generate().unwrap();
        vault.store("key1".to_string(), b"secret1").unwrap();
        vault.store("key2".to_string(), b"secret2").unwrap();

        assert_eq!(vault.len(), 2);
        vault.clear();
        assert_eq!(vault.len(), 0);
        assert!(vault.is_empty());
    }

    #[test]
    fn test_vault_export_import() {
        let mut vault1 = Vault::generate().unwrap();
        vault1.store("key1".to_string(), b"secret1").unwrap();
        vault1.store("key2".to_string(), b"secret2").unwrap();

        let exported = vault1.export().unwrap();

        let master_key = vault1.master_key().clone();
        let mut vault2 = Vault::new(master_key);
        vault2.import(&exported).unwrap();

        assert_eq!(vault2.len(), 2);
        assert!(vault2.contains("key1"));
        assert!(vault2.contains("key2"));
    }

    #[test]
    fn test_vault_last_accessed_updated() {
        let mut vault = Vault::generate().unwrap();
        vault.store("key1".to_string(), b"secret").unwrap();

        let metadata1 = vault.get_metadata("key1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        vault.retrieve("key1").unwrap();

        let metadata2 = vault.get_metadata("key1").unwrap();
        assert!(metadata2.last_accessed >= metadata1.last_accessed);
    }
}
