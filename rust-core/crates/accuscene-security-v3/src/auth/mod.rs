//! Authentication module
//!
//! Provides comprehensive authentication mechanisms including:
//! - Password hashing with Argon2id
//! - JWT token generation and validation
//! - Session management with refresh tokens
//! - Multi-factor authentication (TOTP, WebAuthn)
//! - OAuth 2.0 / OpenID Connect
//! - SAML 2.0 SSO

pub mod password;
pub mod jwt;
pub mod session;
pub mod mfa;
pub mod oauth;

#[cfg(feature = "saml")]
pub mod saml;

pub use password::PasswordHasher;
pub use jwt::{JwtManager, Claims};
pub use session::{SessionManager, Session};
pub use mfa::{TotpManager, WebAuthnManager};
pub use oauth::OAuthClient;

#[cfg(feature = "saml")]
pub use saml::SamlClient;
