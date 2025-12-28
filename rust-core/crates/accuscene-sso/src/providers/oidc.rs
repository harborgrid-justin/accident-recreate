//! OpenID Connect Provider Implementation

use crate::{SSOProvider, SSOUser, SSOError, SSOResult, AuthenticationResult, config::ProviderConfig};
use async_trait::async_trait;
use chrono::{Utc, Duration};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType, CoreAuthenticationFlow},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// OIDC provider
pub struct OIDCProvider {
    name: String,
    client_id: String,
    client_secret: String,
    issuer_url: String,
    scopes: Vec<String>,
    use_pkce: bool,
    client: Arc<RwLock<Option<CoreClient>>>,
    pkce_verifiers: Arc<RwLock<std::collections::HashMap<String, PkceCodeVerifier>>>,
}

impl OIDCProvider {
    /// Create new OIDC provider
    pub fn new(name: &str, config: ProviderConfig) -> SSOResult<Self> {
        match config {
            ProviderConfig::OIDC {
                client_id,
                client_secret,
                issuer_url,
                scopes,
                use_pkce,
                ..
            } => Ok(Self {
                name: name.to_string(),
                client_id: client_id.clone(),
                client_secret: client_secret.clone(),
                issuer_url: issuer_url.clone(),
                scopes,
                use_pkce,
                client: Arc::new(RwLock::new(None)),
                pkce_verifiers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            }),
            _ => Err(SSOError::ConfigError("Invalid OIDC configuration".to_string())),
        }
    }

    /// Initialize OIDC client
    async fn init_client(&self) -> SSOResult<CoreClient> {
        // Check if client already initialized
        {
            let client_lock = self.client.read().await;
            if let Some(client) = &*client_lock {
                return Ok(client.clone());
            }
        }

        // Discover provider metadata
        let issuer_url = IssuerUrl::new(self.issuer_url.clone())
            .map_err(|e| SSOError::OIDCError(format!("Invalid issuer URL: {}", e)))?;

        let metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .map_err(|e| SSOError::OIDCError(format!("Failed to discover provider: {}", e)))?;

        // Create client
        let client = CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
        );

        // Cache client
        {
            let mut client_lock = self.client.write().await;
            *client_lock = Some(client.clone());
        }

        Ok(client)
    }

    /// Extract user info from ID token and UserInfo endpoint
    async fn get_user_info(&self, id_token: &str, access_token: &str) -> SSOResult<SSOUser> {
        // In production, validate and decode ID token
        // and optionally fetch additional claims from UserInfo endpoint

        // Placeholder user info extraction
        Ok(SSOUser {
            id: "oidc_user_123".to_string(),
            email: "user@example.com".to_string(),
            name: Some("OIDC User".to_string()),
            given_name: Some("OIDC".to_string()),
            family_name: Some("User".to_string()),
            picture: None,
            metadata: serde_json::json!({
                "provider": "oidc",
                "issuer": self.issuer_url
            }),
            provider: self.name.clone(),
        })
    }
}

#[async_trait]
impl SSOProvider for OIDCProvider {
    async fn get_authorization_url(&self, state: &str, nonce: Option<&str>) -> SSOResult<String> {
        let client = self.init_client().await?;

        // Build authorization URL
        let mut auth_request = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                || CsrfToken::new(state.to_string()),
                || Nonce::new(nonce.unwrap_or("default_nonce").to_string()),
            );

        // Add scopes
        for scope in &self.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Add PKCE if enabled
        let (pkce_challenge, pkce_verifier) = if self.use_pkce {
            let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

            // Store verifier for later use
            {
                let mut verifiers = self.pkce_verifiers.write().await;
                verifiers.insert(state.to_string(), verifier);
            }

            (Some(challenge), true)
        } else {
            (None, false)
        };

        let (auth_url, _csrf_token, _nonce) = if let Some(challenge) = pkce_challenge {
            auth_request.set_pkce_challenge(challenge).url()
        } else {
            auth_request.url()
        };

        Ok(auth_url.to_string())
    }

    async fn exchange_code(&self, code: &str, state: &str) -> SSOResult<SSOUser> {
        let client = self.init_client().await?;

        // Get PKCE verifier if used
        let pkce_verifier = if self.use_pkce {
            let mut verifiers = self.pkce_verifiers.write().await;
            verifiers.remove(state)
        } else {
            None
        };

        // Exchange authorization code for tokens
        let mut token_request = client
            .exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(verifier) = pkce_verifier {
            token_request = token_request.set_pkce_verifier(verifier);
        }

        let token_response = token_request
            .request_async(async_http_client)
            .await
            .map_err(|e| SSOError::OIDCError(format!("Token exchange failed: {}", e)))?;

        // Extract tokens
        let access_token = token_response.access_token().secret();
        let id_token = token_response
            .id_token()
            .ok_or_else(|| SSOError::OIDCError("No ID token in response".to_string()))?;

        // Get user info
        self.get_user_info(id_token.to_string().as_str(), access_token).await
    }

    async fn validate_token(&self, token: &str) -> SSOResult<SSOUser> {
        // In production, validate JWT signature and claims
        // This is a placeholder
        Ok(SSOUser {
            id: "oidc_user_from_token".to_string(),
            email: "user@example.com".to_string(),
            name: Some("OIDC User".to_string()),
            given_name: None,
            family_name: None,
            picture: None,
            metadata: serde_json::json!({}),
            provider: self.name.clone(),
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> SSOResult<AuthenticationResult> {
        let client = self.init_client().await?;

        // In production, implement token refresh
        Err(SSOError::OIDCError("Token refresh not yet implemented".to_string()))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_creation() {
        let config = ProviderConfig::OIDC {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            issuer_url: "https://accounts.google.com".to_string(),
            auth_endpoint: None,
            token_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            use_pkce: true,
        };

        let provider = OIDCProvider::new("test-oidc", config);
        assert!(provider.is_ok());
    }
}
