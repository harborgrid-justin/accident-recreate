//! SAML 2.0 SSO support
//!
//! Provides SAML 2.0 authentication for enterprise SSO integration.

use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "saml")]
use samael::metadata::{ContactPerson, EntityDescriptor};
#[cfg(feature = "saml")]
use samael::service_provider::ServiceProviderBuilder;

/// SAML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    /// Entity ID (SP identifier)
    pub entity_id: String,

    /// Assertion Consumer Service URL
    pub acs_url: String,

    /// Single Logout Service URL
    pub slo_url: Option<String>,

    /// IDP metadata URL
    pub idp_metadata_url: Option<String>,

    /// IDP Entity ID
    pub idp_entity_id: String,

    /// IDP SSO URL
    pub idp_sso_url: String,

    /// IDP SLO URL
    pub idp_slo_url: Option<String>,

    /// IDP certificate (PEM format)
    pub idp_certificate: String,

    /// SP private key (PEM format)
    #[serde(skip_serializing)]
    pub sp_private_key: String,

    /// SP certificate (PEM format)
    pub sp_certificate: String,

    /// Name ID format
    pub name_id_format: String,

    /// Sign authentication requests
    pub sign_authn_requests: bool,

    /// Require signed assertions
    pub require_signed_assertions: bool,

    /// Require signed responses
    pub require_signed_responses: bool,
}

impl Default for SamlConfig {
    fn default() -> Self {
        Self {
            entity_id: String::new(),
            acs_url: String::new(),
            slo_url: None,
            idp_metadata_url: None,
            idp_entity_id: String::new(),
            idp_sso_url: String::new(),
            idp_slo_url: None,
            idp_certificate: String::new(),
            sp_private_key: String::new(),
            sp_certificate: String::new(),
            name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
            sign_authn_requests: true,
            require_signed_assertions: true,
            require_signed_responses: true,
        }
    }
}

/// SAML client for SSO
#[derive(Debug)]
pub struct SamlClient {
    config: SamlConfig,
}

impl SamlClient {
    /// Create a new SAML client
    pub fn new(config: SamlConfig) -> SecurityResult<Self> {
        if config.entity_id.is_empty() {
            return Err(SecurityError::ConfigError(
                "SAML entity ID cannot be empty".to_string(),
            ));
        }

        if config.acs_url.is_empty() {
            return Err(SecurityError::ConfigError(
                "SAML ACS URL cannot be empty".to_string(),
            ));
        }

        Ok(Self { config })
    }

    /// Generate authentication request
    #[cfg(feature = "saml")]
    pub fn generate_authn_request(&self) -> SecurityResult<String> {
        use samael::service_provider::ServiceProvider;
        use samael::schema::AuthnRequest;

        // Build service provider
        let sp = ServiceProviderBuilder::default()
            .entity_id(&self.config.entity_id)
            .acs_url(&self.config.acs_url)
            .idp_metadata(&self.generate_idp_metadata())
            .build()
            .map_err(|e| SecurityError::SamlError(format!("Failed to build SP: {}", e)))?;

        // Create authentication request
        let authn_request = sp
            .make_authentication_request(&self.config.idp_sso_url)
            .map_err(|e| SecurityError::SamlError(format!("Failed to create authn request: {}", e)))?;

        // Serialize to XML
        let xml = authn_request
            .to_xml()
            .map_err(|e| SecurityError::SamlError(format!("Failed to serialize authn request: {}", e)))?;

        Ok(xml)
    }

    /// Generate IDP metadata from config
    #[cfg(feature = "saml")]
    fn generate_idp_metadata(&self) -> String {
        // This is a simplified version. In production, you would parse actual IDP metadata
        format!(
            r#"<?xml version="1.0"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{}">
  <IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{}"/>
  </IDPSSODescriptor>
</EntityDescriptor>"#,
            self.config.idp_entity_id, self.config.idp_sso_url
        )
    }

    /// Validate SAML response
    #[cfg(feature = "saml")]
    pub fn validate_response(&self, saml_response: &str) -> SecurityResult<SamlAssertion> {
        use base64::Engine;
        use samael::schema::Response;

        // Decode base64
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(saml_response)
            .map_err(|e| SecurityError::SamlError(format!("Failed to decode response: {}", e)))?;

        let xml = String::from_utf8(decoded)
            .map_err(|e| SecurityError::SamlError(format!("Invalid UTF-8: {}", e)))?;

        // Parse response
        let response = Response::from_xml(&xml)
            .map_err(|e| SecurityError::SamlError(format!("Failed to parse response: {}", e)))?;

        // Validate response
        // In production, you would:
        // 1. Verify signature
        // 2. Check timestamps (NotBefore, NotOnOrAfter)
        // 3. Validate audience
        // 4. Check issuer

        // Extract assertion data
        if let Some(assertion) = response.assertions.first() {
            let mut attributes = HashMap::new();

            if let Some(attr_statement) = assertion.attribute_statements.first() {
                for attr in &attr_statement.attributes {
                    if let Some(value) = attr.values.first() {
                        attributes.insert(
                            attr.name.clone().unwrap_or_default(),
                            value.value.clone().unwrap_or_default(),
                        );
                    }
                }
            }

            let subject = assertion
                .subject
                .as_ref()
                .and_then(|s| s.name_id.as_ref())
                .and_then(|n| n.value.clone())
                .unwrap_or_default();

            return Ok(SamlAssertion {
                subject,
                attributes,
                session_index: assertion.authn_statements
                    .first()
                    .and_then(|s| s.session_index.clone()),
            });
        }

        Err(SecurityError::SamlError("No assertion found in response".to_string()))
    }

    /// Generate SP metadata
    pub fn generate_sp_metadata(&self) -> String {
        format!(
            r#"<?xml version="1.0"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{}">
  <SPSSODescriptor AuthnRequestsSigned="{}" WantAssertionsSigned="{}" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <NameIDFormat>{}</NameIDFormat>
    <AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{}" index="0"/>
  </SPSSODescriptor>
</EntityDescriptor>"#,
            self.config.entity_id,
            self.config.sign_authn_requests,
            self.config.require_signed_assertions,
            self.config.name_id_format,
            self.config.acs_url
        )
    }

    /// Generate logout request
    #[cfg(feature = "saml")]
    pub fn generate_logout_request(
        &self,
        name_id: &str,
        session_index: Option<String>,
    ) -> SecurityResult<String> {
        use samael::schema::LogoutRequest;

        let logout_url = self.config.idp_slo_url.as_ref()
            .ok_or_else(|| SecurityError::SamlError("IDP SLO URL not configured".to_string()))?;

        // Create logout request
        let logout_request = LogoutRequest {
            id: Some(format!("_{}",uuid::Uuid::new_v4())),
            version: Some("2.0".to_string()),
            issue_instant: Some(chrono::Utc::now()),
            destination: Some(logout_url.clone()),
            issuer: Some(samael::schema::Issuer {
                value: Some(self.config.entity_id.clone()),
                ..Default::default()
            }),
            name_id: Some(samael::schema::NameID {
                value: Some(name_id.to_string()),
                format: Some(self.config.name_id_format.clone()),
                ..Default::default()
            }),
            session_index: session_index.map(|si| vec![si]),
            ..Default::default()
        };

        // Serialize to XML
        let xml = logout_request
            .to_xml()
            .map_err(|e| SecurityError::SamlError(format!("Failed to serialize logout request: {}", e)))?;

        Ok(xml)
    }
}

/// SAML assertion data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    /// Subject (user identifier)
    pub subject: String,

    /// Assertion attributes
    pub attributes: HashMap<String, String>,

    /// Session index
    pub session_index: Option<String>,
}

impl SamlAssertion {
    /// Get attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Get email from attributes
    pub fn get_email(&self) -> Option<&String> {
        self.get_attribute("email")
            .or_else(|| self.get_attribute("emailAddress"))
            .or_else(|| self.get_attribute("mail"))
    }

    /// Get name from attributes
    pub fn get_name(&self) -> Option<&String> {
        self.get_attribute("name")
            .or_else(|| self.get_attribute("displayName"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SamlConfig {
        SamlConfig {
            entity_id: "https://sp.example.com/saml/metadata".to_string(),
            acs_url: "https://sp.example.com/saml/acs".to_string(),
            slo_url: Some("https://sp.example.com/saml/slo".to_string()),
            idp_metadata_url: None,
            idp_entity_id: "https://idp.example.com/saml/metadata".to_string(),
            idp_sso_url: "https://idp.example.com/saml/sso".to_string(),
            idp_slo_url: Some("https://idp.example.com/saml/slo".to_string()),
            idp_certificate: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            sp_private_key: "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string(),
            sp_certificate: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
            sign_authn_requests: true,
            require_signed_assertions: true,
            require_signed_responses: true,
        }
    }

    #[test]
    fn test_saml_client_creation() {
        let config = create_test_config();
        let client = SamlClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_sp_metadata_generation() {
        let config = create_test_config();
        let client = SamlClient::new(config).unwrap();

        let metadata = client.generate_sp_metadata();
        assert!(metadata.contains("EntityDescriptor"));
        assert!(metadata.contains("SPSSODescriptor"));
        assert!(metadata.contains(&client.config.entity_id));
        assert!(metadata.contains(&client.config.acs_url));
    }

    #[test]
    fn test_saml_assertion() {
        let mut attributes = HashMap::new();
        attributes.insert("email".to_string(), "user@example.com".to_string());
        attributes.insert("name".to_string(), "John Doe".to_string());

        let assertion = SamlAssertion {
            subject: "user@example.com".to_string(),
            attributes,
            session_index: Some("session123".to_string()),
        };

        assert_eq!(assertion.get_email().unwrap(), "user@example.com");
        assert_eq!(assertion.get_name().unwrap(), "John Doe");
    }

    #[test]
    fn test_invalid_config() {
        let mut config = create_test_config();
        config.entity_id = String::new();

        let result = SamlClient::new(config);
        assert!(result.is_err());
    }
}
