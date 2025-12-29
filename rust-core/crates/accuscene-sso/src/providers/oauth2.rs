//! OAuth 2.0 Provider Implementation

use crate::{SSOProvider, SSOUser, SSOError, SSOResult, AuthenticationResult, config::ProviderConfig};
use async_trait::async_trait;
use chrono::{Utc, Duration};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
    basic::BasicClient,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// OAuth 2.0 provider
pub struct OAuth2Provider {
    name: String,
    client_id: String,
    client_secret: String,
    auth_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    scopes: Vec<String>,
    use_pkce: bool,
    client: BasicClient,
    pkce_verifiers: Arc<RwLock<std::collections::HashMap<String, PkceCodeVerifier>>>,
}

impl OAuth2Provider {
    /// Create new OAuth 2.0 provider
    pub fn new(name: &str, config: ProviderConfig) -> SSOResult<Self> {
        match config {
            ProviderConfig::OAuth2 {
                client_id,
                client_secret,
                auth_endpoint,
                token_endpoint,
                userinfo_endpoint,
                scopes,
                use_pkce,
            } => {
                let client = BasicClient::new(
                    ClientId::new(client_id.clone()),
                    Some(ClientSecret::new(client_secret.clone())),
                    AuthUrl::new(auth_endpoint.clone())
                        .map_err(|e| SSOError::OAuth2Error(format!("Invalid auth URL: {}", e)))?,
                    Some(TokenUrl::new(token_endpoint.clone())
                        .map_err(|e| SSOError::OAuth2Error(format!("Invalid token URL: {}", e)))?),
                );

                Ok(Self {
                    name: name.to_string(),
                    client_id,
                    client_secret,
                    auth_endpoint,
                    token_endpoint,
                    userinfo_endpoint,
                    scopes,
                    use_pkce,
                    client,
                    pkce_verifiers: Arc::new(RwLock::new(std::collections::HashMap::new())),
                })
            },
            _ => Err(SSOError::ConfigError("Invalid OAuth2 configuration".to_string())),
        }
    }

    /// Fetch user info from UserInfo endpoint
    async fn fetch_user_info(&self, access_token: &str) -> SSOResult<SSOUser> {
        let client = reqwest::Client::new();

        let response = client
            .get(&self.userinfo_endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| SSOError::OAuth2Error(format!("UserInfo request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SSOError::OAuth2Error(format!(
                "UserInfo request failed with status: {}",
                response.status()
            )));
        }

        let user_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SSOError::OAuth2Error(format!("Failed to parse UserInfo: {}", e)))?;

        // Extract user information
        let id = user_info["id"]
            .as_str()
            .or_else(|| user_info["sub"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let email = user_info["email"]
            .as_str()
            .unwrap_or("no-email@example.com")
            .to_string();

        let name = user_info["name"].as_str().map(|s| s.to_string());
        let given_name = user_info["given_name"].as_str().map(|s| s.to_string());
        let family_name = user_info["family_name"].as_str().map(|s| s.to_string());
        let picture = user_info["picture"].as_str().map(|s| s.to_string());

        Ok(SSOUser {
            id,
            email,
            name,
            given_name,
            family_name,
            picture,
            metadata: user_info.clone(),
            provider: self.name.clone(),
        })
    }
}

#[async_trait]
impl SSOProvider for OAuth2Provider {
    async fn get_authorization_url(&self, state: &str, _nonce: Option<&str>) -> SSOResult<String> {
        // Build authorization URL
        let mut auth_request = self.client
            .authorize_url(|| CsrfToken::new(state.to_string()));

        // Add scopes
        for scope in &self.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Add PKCE if enabled
        let (auth_url, _csrf_token) = if self.use_pkce {
            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

            // Store verifier for later use
            {
                let mut verifiers = self.pkce_verifiers.write().await;
                verifiers.insert(state.to_string(), pkce_verifier);
            }

            auth_request.set_pkce_challenge(pkce_challenge).url()
        } else {
            auth_request.url()
        };

        Ok(auth_url.to_string())
    }

    async fn exchange_code(&self, code: &str, state: &str) -> SSOResult<SSOUser> {
        // Get PKCE verifier if used
        let pkce_verifier = if self.use_pkce {
            let mut verifiers = self.pkce_verifiers.write().await;
            verifiers.remove(state)
        } else {
            None
        };

        // Exchange authorization code for tokens
        let mut token_request = self.client
            .exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(verifier) = pkce_verifier {
            token_request = token_request.set_pkce_verifier(verifier);
        }

        let token_response = token_request
            .request_async(async_http_client)
            .await
            .map_err(|e| SSOError::OAuth2Error(format!("Token exchange failed: {}", e)))?;

        // Get access token
        let access_token = token_response.access_token().secret();

        // Fetch user info
        self.fetch_user_info(access_token).await
    }

    async fn validate_token(&self, token: &str) -> SSOResult<SSOUser> {
        // Fetch user info using the access token
        self.fetch_user_info(token).await
    }

    async fn refresh_token(&self, refresh_token: &str) -> SSOResult<AuthenticationResult> {
        // In production, implement token refresh
        Err(SSOError::OAuth2Error("Token refresh not yet implemented".to_string()))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_provider_creation() {
        let config = ProviderConfig::OAuth2 {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            auth_endpoint: "https://provider.com/oauth/authorize".to_string(),
            token_endpoint: "https://provider.com/oauth/token".to_string(),
            userinfo_endpoint: "https://provider.com/oauth/userinfo".to_string(),
            scopes: vec!["user".to_string(), "email".to_string()],
            use_pkce: true,
        };

        let provider = OAuth2Provider::new("test-oauth2", config);
        assert!(provider.is_ok());
    }
}
