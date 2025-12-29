//! LDAP/Active Directory Provider Implementation

use crate::{SSOProvider, SSOUser, SSOError, SSOResult, AuthenticationResult, config::ProviderConfig};
use async_trait::async_trait;
use ldap3::{LdapConn, LdapConnAsync, Scope, SearchEntry};
use serde::{Deserialize, Serialize};

/// LDAP provider
pub struct LDAPProvider {
    name: String,
    url: String,
    bind_dn: String,
    bind_password: String,
    user_base_dn: String,
    user_filter: String,
    uid_attribute: String,
    email_attribute: String,
    name_attribute: String,
    use_tls: bool,
}

impl LDAPProvider {
    /// Create new LDAP provider
    pub fn new(name: &str, config: ProviderConfig) -> SSOResult<Self> {
        match config {
            ProviderConfig::LDAP {
                url,
                bind_dn,
                bind_password,
                user_base_dn,
                user_filter,
                uid_attribute,
                email_attribute,
                name_attribute,
                use_tls,
                ..
            } => Ok(Self {
                name: name.to_string(),
                url,
                bind_dn,
                bind_password,
                user_base_dn,
                user_filter,
                uid_attribute,
                email_attribute,
                name_attribute,
                use_tls,
            }),
            _ => Err(SSOError::ConfigError("Invalid LDAP configuration".to_string())),
        }
    }

    /// Connect to LDAP server
    async fn connect(&self) -> SSOResult<ldap3::Ldap> {
        let (conn, mut ldap) = LdapConnAsync::new(&self.url)
            .await
            .map_err(|e| SSOError::LDAPError(format!("Connection failed: {}", e)))?;

        // Drive the connection
        ldap3::drive!(conn);

        // Bind with service account
        ldap.simple_bind(&self.bind_dn, &self.bind_password)
            .await
            .map_err(|e| SSOError::LDAPError(format!("Bind failed: {}", e)))?;

        Ok(ldap)
    }

    /// Search for user by username
    async fn search_user(&self, username: &str) -> SSOResult<SearchEntry> {
        let mut ldap = self.connect().await?;

        // Build search filter
        let filter = format!("(&{}({}={}))", self.user_filter, self.uid_attribute, username);

        // Perform search
        let (rs, _res) = ldap
            .search(
                &self.user_base_dn,
                Scope::Subtree,
                &filter,
                vec![
                    &self.uid_attribute,
                    &self.email_attribute,
                    &self.name_attribute,
                    "givenName",
                    "sn",
                ],
            )
            .await
            .map_err(|e| SSOError::LDAPError(format!("Search failed: {}", e)))?
            .success()
            .map_err(|e| SSOError::LDAPError(format!("Search error: {}", e)))?;

        // Unbind
        ldap.unbind()
            .await
            .map_err(|e| SSOError::LDAPError(format!("Unbind failed: {}", e)))?;

        // Get first result
        let entry = rs
            .into_iter()
            .next()
            .ok_or_else(|| SSOError::InvalidCredentials)?;

        Ok(SearchEntry::construct(entry))
    }

    /// Authenticate user with LDAP
    async fn authenticate(&self, username: &str, password: &str) -> SSOResult<SSOUser> {
        // First, find the user
        let user_entry = self.search_user(username).await?;

        // Get user DN
        let user_dn = user_entry.dn.clone();

        // Try to bind with user credentials
        let (conn, mut ldap) = LdapConnAsync::new(&self.url)
            .await
            .map_err(|e| SSOError::LDAPError(format!("Connection failed: {}", e)))?;

        ldap3::drive!(conn);

        ldap.simple_bind(&user_dn, password)
            .await
            .map_err(|_| SSOError::InvalidCredentials)?;

        // Unbind
        ldap.unbind()
            .await
            .map_err(|e| SSOError::LDAPError(format!("Unbind failed: {}", e)))?;

        // Extract user attributes
        let id = user_entry.attrs.get(&self.uid_attribute)
            .and_then(|v| v.first())
            .ok_or_else(|| SSOError::LDAPError("Missing UID attribute".to_string()))?
            .clone();

        let email = user_entry.attrs.get(&self.email_attribute)
            .and_then(|v| v.first())
            .ok_or_else(|| SSOError::LDAPError("Missing email attribute".to_string()))?
            .clone();

        let name = user_entry.attrs.get(&self.name_attribute)
            .and_then(|v| v.first())
            .map(|s| s.clone());

        let given_name = user_entry.attrs.get("givenName")
            .and_then(|v| v.first())
            .map(|s| s.clone());

        let family_name = user_entry.attrs.get("sn")
            .and_then(|v| v.first())
            .map(|s| s.clone());

        Ok(SSOUser {
            id,
            email,
            name,
            given_name,
            family_name,
            picture: None,
            metadata: serde_json::json!({
                "provider": "ldap",
                "dn": user_dn
            }),
            provider: self.name.clone(),
        })
    }
}

#[async_trait]
impl SSOProvider for LDAPProvider {
    async fn get_authorization_url(&self, state: &str, _nonce: Option<&str>) -> SSOResult<String> {
        // LDAP doesn't use OAuth-style flows
        // Return a custom URL for LDAP login form
        Ok(format!("/auth/ldap/login?state={}", state))
    }

    async fn exchange_code(&self, code: &str, _state: &str) -> SSOResult<SSOUser> {
        // For LDAP, "code" is in format "username:password" (base64 encoded)
        let decoded = base64::decode(code)
            .map_err(|e| SSOError::LDAPError(format!("Invalid credentials format: {}", e)))?;

        let credentials = String::from_utf8(decoded)
            .map_err(|e| SSOError::LDAPError(format!("Invalid UTF-8: {}", e)))?;

        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(SSOError::InvalidCredentials);
        }

        let username = parts[0];
        let password = parts[1];

        self.authenticate(username, password).await
    }

    async fn validate_token(&self, token: &str) -> SSOResult<SSOUser> {
        // LDAP doesn't use tokens
        // This would validate a session token created after LDAP authentication
        Err(SSOError::LDAPError("Token validation not supported for LDAP".to_string()))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> SSOResult<AuthenticationResult> {
        // LDAP doesn't support token refresh
        Err(SSOError::LDAPError("Token refresh not supported for LDAP".to_string()))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// LDAP login credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct LDAPCredentials {
    pub username: String,
    pub password: String,
}

impl LDAPCredentials {
    /// Encode credentials for exchange
    pub fn encode(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        base64::encode(credentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldap_provider_creation() {
        let config = ProviderConfig::LDAP {
            url: "ldap://ldap.example.com".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "password".to_string(),
            user_base_dn: "ou=users,dc=example,dc=com".to_string(),
            user_filter: "(objectClass=person)".to_string(),
            uid_attribute: "uid".to_string(),
            email_attribute: "mail".to_string(),
            name_attribute: "cn".to_string(),
            use_tls: true,
            tls_ca_cert: None,
        };

        let provider = LDAPProvider::new("test-ldap", config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_ldap_credentials_encoding() {
        let creds = LDAPCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let encoded = creds.encode();
        assert!(!encoded.is_empty());
    }
}
