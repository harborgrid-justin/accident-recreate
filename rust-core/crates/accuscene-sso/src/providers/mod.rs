//! SSO Provider Implementations
//!
//! Supports multiple SSO protocols:
//! - SAML 2.0
//! - OpenID Connect
//! - OAuth 2.0
//! - LDAP/Active Directory

pub mod saml;
pub mod oidc;
pub mod oauth2;
pub mod ldap;

use crate::{SSOProvider, SSOError, SSOResult, config::{SSOConfig, ProviderConfig}};
use std::sync::Arc;

/// Get provider instance by name
pub fn get_provider(config: &SSOConfig, name: &str) -> SSOResult<Box<dyn SSOProvider>> {
    let provider_config = config.providers.get(name)
        .ok_or_else(|| SSOError::ProviderNotFound(name.to_string()))?;

    match provider_config {
        ProviderConfig::SAML { .. } => {
            Ok(Box::new(saml::SAMLProvider::new(name, provider_config.clone())?))
        },
        ProviderConfig::OIDC { .. } => {
            Ok(Box::new(oidc::OIDCProvider::new(name, provider_config.clone())?))
        },
        ProviderConfig::OAuth2 { .. } => {
            Ok(Box::new(oauth2::OAuth2Provider::new(name, provider_config.clone())?))
        },
        ProviderConfig::LDAP { .. } => {
            Ok(Box::new(ldap::LDAPProvider::new(name, provider_config.clone())?))
        },
    }
}

/// Provider metadata
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    /// Provider name
    pub name: String,

    /// Provider type
    pub provider_type: String,

    /// Display name
    pub display_name: String,

    /// Icon URL
    pub icon_url: Option<String>,

    /// Enabled status
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_provider_not_found() {
        let config = SSOConfig::default();
        let result = get_provider(&config, "nonexistent");
        assert!(matches!(result, Err(SSOError::ProviderNotFound(_))));
    }
}
