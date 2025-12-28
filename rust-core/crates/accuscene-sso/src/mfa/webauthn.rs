//! WebAuthn/FIDO2 Implementation

use crate::{SSOError, SSOResult};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;
use std::sync::Arc;

/// Credential registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRegistration {
    /// User ID
    pub user_id: String,
    /// Credential ID
    pub credential_id: Vec<u8>,
    /// Public key
    pub public_key: Vec<u8>,
    /// Counter
    pub counter: u32,
    /// Credential name
    pub name: String,
}

/// Authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    /// Challenge bytes
    pub challenge: Vec<u8>,
    /// User ID
    pub user_id: String,
    /// Timeout in milliseconds
    pub timeout: u32,
}

/// WebAuthn manager
pub struct WebAuthnManager {
    webauthn: Arc<Webauthn>,
}

impl WebAuthnManager {
    /// Create new WebAuthn manager
    pub fn new(
        rp_name: String,
        rp_id: String,
        rp_origin: Url,
    ) -> SSOResult<Self> {
        let rp = RelyingPartyBuilder::new(rp_name.as_str())
            .rp_id(rp_id)
            .build()
            .map_err(|e| SSOError::InternalError(format!("Failed to build RP: {:?}", e)))?;

        let webauthn = WebauthnBuilder::new(&rp_origin.domain().unwrap_or("localhost"), &rp)
            .map_err(|e| SSOError::InternalError(format!("Failed to build WebAuthn: {:?}", e)))?
            .build()
            .map_err(|e| SSOError::InternalError(format!("Failed to build WebAuthn: {:?}", e)))?;

        Ok(Self {
            webauthn: Arc::new(webauthn),
        })
    }

    /// Start credential registration
    pub fn start_registration(
        &self,
        user_id: &str,
        username: &str,
        display_name: &str,
    ) -> SSOResult<(CreationChallengeResponse, PasskeyRegistration)> {
        let user_unique_id = {
            let mut bytes = [0u8; 32];
            let user_id_bytes = user_id.as_bytes();
            let len = user_id_bytes.len().min(32);
            bytes[..len].copy_from_slice(&user_id_bytes[..len]);
            bytes
        };

        let (ccr, reg_state) = self.webauthn
            .start_passkey_registration(
                Uuid::from_bytes(user_unique_id),
                username,
                display_name,
                None,
            )
            .map_err(|e| SSOError::InternalError(format!("Failed to start registration: {:?}", e)))?;

        Ok((ccr, reg_state))
    }

    /// Finish credential registration
    pub fn finish_registration(
        &self,
        user_id: &str,
        name: String,
        reg: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> SSOResult<CredentialRegistration> {
        let passkey = self.webauthn
            .finish_passkey_registration(reg, state)
            .map_err(|e| SSOError::InternalError(format!("Failed to finish registration: {:?}", e)))?;

        // Extract credential data
        Ok(CredentialRegistration {
            user_id: user_id.to_string(),
            credential_id: passkey.cred_id().to_vec(),
            public_key: Vec::new(), // Serialized in production
            counter: passkey.counter(),
            name,
        })
    }

    /// Start authentication
    pub fn start_authentication(
        &self,
        credentials: Vec<Passkey>,
    ) -> SSOResult<(RequestChallengeResponse, PasskeyAuthentication)> {
        let (rcr, auth_state) = self.webauthn
            .start_passkey_authentication(&credentials)
            .map_err(|e| SSOError::InternalError(format!("Failed to start authentication: {:?}", e)))?;

        Ok((rcr, auth_state))
    }

    /// Finish authentication
    pub fn finish_authentication(
        &self,
        auth: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> SSOResult<()> {
        self.webauthn
            .finish_passkey_authentication(auth, state)
            .map_err(|e| SSOError::InvalidMFACode)?;

        Ok(())
    }
}

/// WebAuthn credential storage
pub struct CredentialStorage {
    /// In-memory storage (use database in production)
    credentials: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<Passkey>>>>,
}

impl CredentialStorage {
    /// Create new credential storage
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add credential for user
    pub async fn add_credential(&self, user_id: String, credential: Passkey) {
        let mut creds = self.credentials.write().await;
        creds.entry(user_id).or_insert_with(Vec::new).push(credential);
    }

    /// Get credentials for user
    pub async fn get_credentials(&self, user_id: &str) -> Vec<Passkey> {
        let creds = self.credentials.read().await;
        creds.get(user_id).cloned().unwrap_or_default()
    }

    /// Remove credential
    pub async fn remove_credential(&self, user_id: &str, credential_id: &[u8]) {
        let mut creds = self.credentials.write().await;
        if let Some(user_creds) = creds.get_mut(user_id) {
            user_creds.retain(|c| c.cred_id() != credential_id);
        }
    }
}

impl Default for CredentialStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webauthn_manager_creation() {
        let result = WebAuthnManager::new(
            "AccuScene Enterprise".to_string(),
            "localhost".to_string(),
            Url::parse("http://localhost:3000").unwrap(),
        );

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_credential_storage() {
        let storage = CredentialStorage::new();
        let user_id = "user123".to_string();

        let creds = storage.get_credentials(&user_id).await;
        assert_eq!(creds.len(), 0);
    }
}
