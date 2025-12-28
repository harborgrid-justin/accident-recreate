//! Authentication framework
//!
//! Comprehensive authentication system with password hashing, MFA, SSO, sessions, and JWT tokens.

pub mod mfa;
pub mod password;
pub mod session;
pub mod sso;
pub mod token;

use crate::config::AuthConfig;
use crate::error::{Result, SecurityError};
use serde::{Deserialize, Serialize};

pub use mfa::{MfaEnrollment, MfaMethod, MfaService, TotpSecret};
pub use password::{PasswordHashService, PasswordHistory};
pub use session::{GeoLocation, Session, SessionManager, SessionMetadata};
pub use sso::{OidcClaims, SamlAssertion, SsoProvider, SsoService};
pub use token::{JwtClaims, TokenBlacklist, TokenClaims, TokenPair, TokenService, TokenType};

/// Authentication service coordinating all auth mechanisms
pub struct AuthenticationService {
    password_service: PasswordHashService,
    mfa_service: MfaService,
    sso_service: SsoService,
    session_manager: SessionManager,
    token_service: TokenService,
    token_blacklist: TokenBlacklist,
}

impl AuthenticationService {
    /// Create a new authentication service
    pub fn new(config: AuthConfig, jwt_secret: &[u8]) -> Self {
        Self {
            password_service: PasswordHashService::new(config.password_policy.clone()),
            mfa_service: MfaService::new(config.mfa.clone()),
            sso_service: SsoService::new(config.sso.clone()),
            session_manager: SessionManager::new(config.session.clone()),
            token_service: TokenService::new(config.jwt.clone(), jwt_secret),
            token_blacklist: TokenBlacklist::new(),
        }
    }

    /// Authenticate user with password
    pub async fn authenticate_password(
        &mut self,
        credentials: PasswordCredentials,
    ) -> Result<AuthenticationResult> {
        // In production, load user from database
        // For now, we'll validate the password format
        self.password_service
            .verify_password(&credentials.password, &credentials.password_hash)?;

        // Check if MFA is required
        if credentials.mfa_required {
            return Ok(AuthenticationResult::MfaRequired {
                user_id: credentials.user_id,
                available_methods: vec![MfaMethod::Totp],
            });
        }

        // Create session
        let session = self.session_manager.create_session(
            credentials.user_id.clone(),
            credentials.metadata,
        )?;

        // Generate tokens
        let token_claims = TokenClaims {
            session_id: Some(session.id.clone()),
            mfa_verified: false,
            ..Default::default()
        };

        let access_token = self
            .token_service
            .generate_access_token(&credentials.user_id, token_claims)?;

        let refresh_token = self
            .token_service
            .generate_refresh_token(&credentials.user_id)?;

        Ok(AuthenticationResult::Success {
            session,
            tokens: TokenPair::new(access_token, refresh_token, 3600),
        })
    }

    /// Verify MFA and complete authentication
    pub async fn verify_mfa(
        &mut self,
        user_id: String,
        method: MfaMethod,
        code: &str,
        enrollment: &MfaEnrollment,
        metadata: SessionMetadata,
    ) -> Result<AuthenticationResult> {
        // Verify MFA code
        let verified = match method {
            MfaMethod::Totp => {
                let secret = enrollment
                    .totp_secret
                    .as_ref()
                    .ok_or(SecurityError::InvalidMfaToken)?;
                self.mfa_service.verify_totp(secret, code)?
            }
            MfaMethod::BackupCode => {
                // Check against stored backup codes
                false // Simplified for now
            }
            MfaMethod::WebAuthn => {
                // Verify WebAuthn assertion
                false // Simplified for now
            }
        };

        if !verified {
            return Err(SecurityError::InvalidMfaToken);
        }

        // Create session
        let session = self.session_manager.create_session(user_id.clone(), metadata)?;

        // Generate tokens with MFA verified
        let token_claims = TokenClaims {
            session_id: Some(session.id.clone()),
            mfa_verified: true,
            ..Default::default()
        };

        let access_token = self
            .token_service
            .generate_access_token(&user_id, token_claims)?;

        let refresh_token = self.token_service.generate_refresh_token(&user_id)?;

        Ok(AuthenticationResult::Success {
            session,
            tokens: TokenPair::new(access_token, refresh_token, 3600),
        })
    }

    /// Authenticate via SSO
    pub async fn authenticate_sso(
        &mut self,
        provider: SsoProvider,
        assertion: SsoAssertion,
        metadata: SessionMetadata,
    ) -> Result<AuthenticationResult> {
        let user_id = match provider {
            SsoProvider::Saml => {
                let saml_assertion = assertion
                    .saml
                    .ok_or(SecurityError::InvalidToken("Missing SAML assertion".to_string()))?;
                self.sso_service.validate_saml_response(&saml_assertion)?;
                saml_assertion.assertion.subject
            }
            SsoProvider::Oidc => {
                let oidc_token = assertion
                    .oidc_token
                    .ok_or(SecurityError::InvalidToken("Missing OIDC token".to_string()))?;
                let claims = self.sso_service.validate_oidc_token(&oidc_token)?;
                claims.sub
            }
        };

        // Create session
        let session = self
            .session_manager
            .create_session(user_id.clone(), metadata)?;

        // Generate tokens
        let token_claims = TokenClaims {
            session_id: Some(session.id.clone()),
            mfa_verified: true, // SSO implies MFA
            ..Default::default()
        };

        let access_token = self
            .token_service
            .generate_access_token(&user_id, token_claims)?;

        let refresh_token = self.token_service.generate_refresh_token(&user_id)?;

        Ok(AuthenticationResult::Success {
            session,
            tokens: TokenPair::new(access_token, refresh_token, 3600),
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&mut self, refresh_token: &str) -> Result<TokenPair> {
        // Validate refresh token
        let claims = self.token_service.validate_refresh_token(refresh_token)?;

        // Check if token is blacklisted
        if self.token_blacklist.is_revoked(&claims.jti) {
            return Err(SecurityError::InvalidToken("Token revoked".to_string()));
        }

        // Generate new access token
        let token_claims = TokenClaims {
            session_id: claims.custom.session_id.clone(),
            mfa_verified: claims.custom.mfa_verified,
            roles: claims.custom.roles.clone(),
            permissions: claims.custom.permissions.clone(),
            ..Default::default()
        };

        let access_token = self
            .token_service
            .generate_access_token(&claims.sub, token_claims)?;

        // Generate new refresh token
        let new_refresh_token = self.token_service.generate_refresh_token(&claims.sub)?;

        // Revoke old refresh token
        let exp = chrono::DateTime::from_timestamp(claims.exp, 0)
            .unwrap_or_else(chrono::Utc::now);
        self.token_blacklist.revoke(claims.jti, exp);

        Ok(TokenPair::new(access_token, new_refresh_token, 3600))
    }

    /// Logout user
    pub async fn logout(&mut self, session_id: &str, token_jti: &str) -> Result<()> {
        // Invalidate session
        self.session_manager.invalidate_session(session_id)?;

        // Blacklist token
        let exp = chrono::Utc::now() + chrono::Duration::hours(24);
        self.token_blacklist.revoke(token_jti.to_string(), exp);

        Ok(())
    }

    /// Validate authentication token
    pub async fn validate_request(&mut self, token: &str) -> Result<AuthContext> {
        // Validate token
        let claims = self.token_service.validate_token(token)?;

        // Check if token is blacklisted
        if self.token_blacklist.is_revoked(&claims.jti) {
            return Err(SecurityError::InvalidToken("Token revoked".to_string()));
        }

        // Validate session if present
        if let Some(session_id) = &claims.custom.session_id {
            let session = self.session_manager.get_session(session_id)?;

            // Touch session to update activity
            self.session_manager.touch_session(session_id)?;

            return Ok(AuthContext {
                user_id: claims.sub,
                session_id: Some(session_id.clone()),
                roles: claims.custom.roles,
                permissions: claims.custom.permissions,
                mfa_verified: claims.custom.mfa_verified,
                session_metadata: Some(session.metadata.clone()),
            });
        }

        Ok(AuthContext {
            user_id: claims.sub,
            session_id: None,
            roles: claims.custom.roles,
            permissions: claims.custom.permissions,
            mfa_verified: claims.custom.mfa_verified,
            session_metadata: None,
        })
    }
}

/// Password authentication credentials
#[derive(Debug, Clone)]
pub struct PasswordCredentials {
    /// User ID
    pub user_id: String,
    /// Password to verify
    pub password: String,
    /// Stored password hash
    pub password_hash: String,
    /// MFA required flag
    pub mfa_required: bool,
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// SSO assertion data
#[derive(Debug, Clone)]
pub struct SsoAssertion {
    /// SAML response
    pub saml: Option<sso::SamlResponse>,
    /// OIDC ID token
    pub oidc_token: Option<String>,
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthenticationResult {
    /// Authentication successful
    Success {
        /// User session
        session: Session,
        /// Access and refresh tokens
        tokens: TokenPair,
    },
    /// MFA required
    MfaRequired {
        /// User ID
        user_id: String,
        /// Available MFA methods
        available_methods: Vec<MfaMethod>,
    },
}

/// Authentication context for authorized requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// Authenticated user ID
    pub user_id: String,
    /// Session ID
    pub session_id: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<String>,
    /// MFA verified
    pub mfa_verified: bool,
    /// Session metadata
    pub session_metadata: Option<SessionMetadata>,
}

impl AuthContext {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;

    #[tokio::test]
    async fn test_authentication_service_creation() {
        let config = SecurityConfig::default();
        let _service = AuthenticationService::new(
            config.auth,
            b"test-secret-key-32-bytes-long!!!",
        );
    }

    #[test]
    fn test_auth_context_role_checks() {
        let context = AuthContext {
            user_id: "user123".to_string(),
            session_id: Some("session123".to_string()),
            roles: vec!["admin".to_string(), "user".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            mfa_verified: true,
            session_metadata: None,
        };

        assert!(context.has_role("admin"));
        assert!(context.has_role("user"));
        assert!(!context.has_role("superadmin"));
        assert!(context.is_admin());
    }

    #[test]
    fn test_auth_context_permission_checks() {
        let context = AuthContext {
            user_id: "user123".to_string(),
            session_id: None,
            roles: vec![],
            permissions: vec!["read".to_string()],
            mfa_verified: false,
            session_metadata: None,
        };

        assert!(context.has_permission("read"));
        assert!(!context.has_permission("write"));
    }
}
