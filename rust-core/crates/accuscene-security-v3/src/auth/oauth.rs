//! OAuth 2.0 / OpenID Connect client
//!
//! Provides OAuth 2.0 and OpenID Connect integration for SSO.

use crate::error::{SecurityError, SecurityResult};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,

    /// Client secret
    #[serde(skip_serializing)]
    pub client_secret: String,

    /// Authorization URL
    pub auth_url: String,

    /// Token URL
    pub token_url: String,

    /// Redirect URL
    pub redirect_url: String,

    /// Scopes to request
    pub scopes: Vec<String>,

    /// Use PKCE
    pub use_pkce: bool,
}

/// OAuth client for authentication
#[derive(Debug)]
pub struct OAuthClient {
    config: OAuthConfig,
    client: BasicClient,
}

impl OAuthClient {
    /// Create a new OAuth client
    pub fn new(config: OAuthConfig) -> SecurityResult<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())
                .map_err(|e| SecurityError::OAuthError(format!("Invalid auth URL: {}", e)))?,
            Some(
                TokenUrl::new(config.token_url.clone())
                    .map_err(|e| SecurityError::OAuthError(format!("Invalid token URL: {}", e)))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone())
                .map_err(|e| SecurityError::OAuthError(format!("Invalid redirect URL: {}", e)))?,
        );

        Ok(Self { config, client })
    }

    /// Get authorization URL
    pub fn get_authorization_url(&self) -> SecurityResult<(String, CsrfToken, Option<String>)> {
        let mut auth_request = self.client.authorize_url(CsrfToken::new_random);

        // Add scopes
        for scope in &self.config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        // Use PKCE if enabled
        let pkce_verifier = if self.config.use_pkce {
            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
            auth_request = auth_request.set_pkce_challenge(pkce_challenge);
            Some(pkce_verifier.secret().clone())
        } else {
            None
        };

        let (url, csrf_token) = auth_request.url();

        Ok((url.to_string(), csrf_token, pkce_verifier))
    }

    /// Exchange authorization code for access token
    #[cfg(feature = "async")]
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: Option<String>,
    ) -> SecurityResult<OAuthToken> {
        use oauth2::PkceCodeVerifier;

        let mut token_request = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(verifier) = pkce_verifier {
            token_request = token_request.set_pkce_verifier(PkceCodeVerifier::new(verifier));
        }

        let token_response = token_request
            .request_async(async_http_client)
            .await
            .map_err(|e| SecurityError::OAuthError(format!("Token exchange failed: {}", e)))?;

        Ok(OAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response
                .refresh_token()
                .map(|t| t.secret().clone()),
            expires_in: token_response
                .expires_in()
                .map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
        })
    }

    /// Refresh an access token
    #[cfg(feature = "async")]
    pub async fn refresh_token(&self, refresh_token: &str) -> SecurityResult<OAuthToken> {
        use oauth2::RefreshToken;

        let token_response = self
            .client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| SecurityError::OAuthError(format!("Token refresh failed: {}", e)))?;

        Ok(OAuthToken {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response
                .refresh_token()
                .map(|t| t.secret().clone()),
            expires_in: token_response
                .expires_in()
                .map(|d| d.as_secs()),
            token_type: "Bearer".to_string(),
        })
    }
}

/// OAuth token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,

    /// Refresh token (optional)
    pub refresh_token: Option<String>,

    /// Expires in seconds
    pub expires_in: Option<u64>,

    /// Token type (usually "Bearer")
    pub token_type: String,
}

/// OpenID Connect user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// Subject (user ID)
    pub sub: String,

    /// Email address
    pub email: Option<String>,

    /// Email verified
    pub email_verified: Option<bool>,

    /// Name
    pub name: Option<String>,

    /// Given name
    pub given_name: Option<String>,

    /// Family name
    pub family_name: Option<String>,

    /// Picture URL
    pub picture: Option<String>,

    /// Additional claims
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Predefined OAuth providers
pub struct OAuthProviders;

impl OAuthProviders {
    /// Google OAuth configuration
    pub fn google(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> OAuthConfig {
        OAuthConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_url: redirect_url.into(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            use_pkce: true,
        }
    }

    /// Microsoft OAuth configuration
    pub fn microsoft(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_url: impl Into<String>,
        tenant: impl Into<String>,
    ) -> OAuthConfig {
        let tenant = tenant.into();
        OAuthConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                tenant
            ),
            token_url: format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                tenant
            ),
            redirect_url: redirect_url.into(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            use_pkce: true,
        }
    }

    /// GitHub OAuth configuration
    pub fn github(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> OAuthConfig {
        OAuthConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            redirect_url: redirect_url.into(),
            scopes: vec!["read:user".to_string(), "user:email".to_string()],
            use_pkce: false,
        }
    }

    /// Okta OAuth configuration
    pub fn okta(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_url: impl Into<String>,
        domain: impl Into<String>,
    ) -> OAuthConfig {
        let domain = domain.into();
        OAuthConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            auth_url: format!("https://{}/oauth2/v1/authorize", domain),
            token_url: format!("https://{}/oauth2/v1/token", domain),
            redirect_url: redirect_url.into(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            use_pkce: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config_creation() {
        let config = OAuthProviders::google(
            "client-id",
            "client-secret",
            "https://example.com/callback",
        );

        assert_eq!(config.client_id, "client-id");
        assert!(config.auth_url.contains("google"));
        assert_eq!(config.scopes.len(), 3);
    }

    #[test]
    fn test_oauth_client_creation() {
        let config = OAuthProviders::google(
            "client-id",
            "client-secret",
            "https://example.com/callback",
        );

        let client = OAuthClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuthProviders::google(
            "client-id",
            "client-secret",
            "https://example.com/callback",
        );

        let client = OAuthClient::new(config).unwrap();
        let (url, _csrf, pkce) = client.get_authorization_url().unwrap();

        assert!(url.contains("accounts.google.com"));
        assert!(url.contains("client_id=client-id"));
        assert!(pkce.is_some()); // PKCE enabled for Google
    }

    #[test]
    fn test_microsoft_provider() {
        let config = OAuthProviders::microsoft(
            "client-id",
            "client-secret",
            "https://example.com/callback",
            "common",
        );

        assert!(config.auth_url.contains("login.microsoftonline.com"));
        assert!(config.auth_url.contains("common"));
    }

    #[test]
    fn test_github_provider() {
        let config = OAuthProviders::github(
            "client-id",
            "client-secret",
            "https://example.com/callback",
        );

        assert!(config.auth_url.contains("github.com"));
        assert!(!config.use_pkce); // GitHub doesn't support PKCE
    }
}
