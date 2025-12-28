//! Simple certificate handling
//!
//! Provides a lightweight certificate system for public key distribution and verification.

use crate::asymmetric::signing::{verify_signature, Ed25519Signer, Signature};
use crate::asymmetric::{Ed25519KeyPair, Ed25519PublicKey};
use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A simple certificate for public key distribution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    /// The subject of the certificate (e.g., user ID, domain name)
    pub subject: String,
    /// The public key being certified
    pub public_key: Ed25519PublicKey,
    /// When the certificate was issued (Unix timestamp)
    pub issued_at: u64,
    /// When the certificate expires (Unix timestamp)
    pub expires_at: u64,
    /// The issuer of the certificate
    pub issuer: String,
    /// Additional metadata
    pub metadata: CertificateMetadata,
    /// The signature from the issuer
    pub signature: Signature,
}

/// Metadata for certificates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateMetadata {
    /// Certificate serial number
    pub serial_number: String,
    /// Certificate purpose
    pub purpose: Option<String>,
    /// Custom attributes
    pub attributes: std::collections::HashMap<String, String>,
}

impl Default for CertificateMetadata {
    fn default() -> Self {
        Self {
            serial_number: uuid::Uuid::new_v4().to_string(),
            purpose: None,
            attributes: std::collections::HashMap::new(),
        }
    }
}

impl Certificate {
    /// Check if the certificate is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    /// Check if the certificate is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Get the remaining validity period
    pub fn remaining_validity(&self) -> Option<Duration> {
        if self.is_expired() {
            return None;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Some(Duration::from_secs(self.expires_at - now))
    }

    /// Verify the certificate signature
    pub fn verify_signature(&self, issuer_public_key: &Ed25519PublicKey) -> CryptoResult<bool> {
        let cert_data = self.serialize_for_signing()?;
        verify_signature(issuer_public_key, &cert_data, &self.signature)
    }

    /// Encode the certificate to base64
    pub fn to_base64(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json.as_bytes()))
    }

    /// Decode a certificate from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let json_bytes = base64::decode(encoded)?;
        let json_str = std::str::from_utf8(&json_bytes)
            .map_err(|e| CryptoError::DecodingError(e.to_string()))?;
        let cert = serde_json::from_str(json_str)?;
        Ok(cert)
    }

    /// Serialize certificate data for signing (excludes signature field)
    fn serialize_for_signing(&self) -> CryptoResult<Vec<u8>> {
        #[derive(Serialize)]
        struct CertDataForSigning<'a> {
            subject: &'a str,
            public_key: &'a Ed25519PublicKey,
            issued_at: u64,
            expires_at: u64,
            issuer: &'a str,
            metadata: &'a CertificateMetadata,
        }

        let data = CertDataForSigning {
            subject: &self.subject,
            public_key: &self.public_key,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            issuer: &self.issuer,
            metadata: &self.metadata,
        };

        serde_json::to_vec(&data).map_err(|e| CryptoError::SerializationError(e.to_string()))
    }
}

/// Certificate authority for issuing and verifying certificates
pub struct CertificateAuthority {
    /// The CA's signing key pair
    keypair: Ed25519KeyPair,
    /// The CA's name/identifier
    name: String,
}

impl CertificateAuthority {
    /// Create a new certificate authority
    pub fn new(name: String, keypair: Ed25519KeyPair) -> Self {
        Self { keypair, name }
    }

    /// Generate a new CA with a random key pair
    pub fn generate(name: String) -> CryptoResult<Self> {
        let keypair = Ed25519KeyPair::generate()?;
        Ok(Self::new(name, keypair))
    }

    /// Issue a certificate for a public key
    pub fn issue_certificate(
        &self,
        subject: String,
        public_key: Ed25519PublicKey,
        validity: Duration,
        metadata: CertificateMetadata,
    ) -> CryptoResult<Certificate> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = now + validity.as_secs();

        // Create unsigned certificate
        let mut cert = Certificate {
            subject,
            public_key,
            issued_at: now,
            expires_at,
            issuer: self.name.clone(),
            metadata,
            signature: Signature::from_bytes([0u8; 64]), // Placeholder
        };

        // Sign the certificate
        let cert_data = cert.serialize_for_signing()?;
        let signer = Ed25519Signer::new(self.keypair.clone());
        let signature = signer.sign(&cert_data)?;

        cert.signature = signature;
        Ok(cert)
    }

    /// Verify a certificate issued by this CA
    pub fn verify_certificate(&self, cert: &Certificate) -> CryptoResult<bool> {
        // Check if we issued this certificate
        if cert.issuer != self.name {
            return Err(CryptoError::InvalidCertificate(
                "Certificate not issued by this CA".to_string(),
            ));
        }

        // Check if expired
        if cert.is_expired() {
            return Err(CryptoError::CertificateExpired);
        }

        // Verify signature
        cert.verify_signature(self.keypair.public_key())
    }

    /// Get the CA's public key
    pub fn public_key(&self) -> &Ed25519PublicKey {
        self.keypair.public_key()
    }

    /// Get the CA's name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Certificate builder for creating certificates with custom options
pub struct CertificateBuilder {
    subject: String,
    public_key: Ed25519PublicKey,
    validity: Duration,
    metadata: CertificateMetadata,
}

impl CertificateBuilder {
    /// Create a new certificate builder
    pub fn new(subject: String, public_key: Ed25519PublicKey) -> Self {
        Self {
            subject,
            public_key,
            validity: Duration::from_secs(365 * 24 * 60 * 60), // 1 year default
            metadata: CertificateMetadata::default(),
        }
    }

    /// Set the validity period
    pub fn validity(mut self, validity: Duration) -> Self {
        self.validity = validity;
        self
    }

    /// Set the purpose
    pub fn purpose(mut self, purpose: String) -> Self {
        self.metadata.purpose = Some(purpose);
        self
    }

    /// Add a custom attribute
    pub fn attribute(mut self, key: String, value: String) -> Self {
        self.metadata.attributes.insert(key, value);
        self
    }

    /// Set custom metadata
    pub fn metadata(mut self, metadata: CertificateMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build and sign the certificate with a CA
    pub fn sign(self, ca: &CertificateAuthority) -> CryptoResult<Certificate> {
        ca.issue_certificate(self.subject, self.public_key, self.validity, self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ca() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        assert_eq!(ca.name(), "Test CA");
    }

    #[test]
    fn test_issue_certificate() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(3600),
                CertificateMetadata::default(),
            )
            .unwrap();

        assert_eq!(cert.subject, "user@example.com");
        assert_eq!(cert.issuer, "Test CA");
        assert!(cert.is_valid());
    }

    #[test]
    fn test_verify_certificate() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(3600),
                CertificateMetadata::default(),
            )
            .unwrap();

        assert!(ca.verify_certificate(&cert).unwrap());
    }

    #[test]
    fn test_expired_certificate() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(0),
                CertificateMetadata::default(),
            )
            .unwrap();

        std::thread::sleep(Duration::from_millis(10));
        assert!(cert.is_expired());
        assert!(!cert.is_valid());
        assert!(ca.verify_certificate(&cert).is_err());
    }

    #[test]
    fn test_certificate_base64_roundtrip() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(3600),
                CertificateMetadata::default(),
            )
            .unwrap();

        let encoded = cert.to_base64().unwrap();
        let decoded = Certificate::from_base64(&encoded).unwrap();

        assert_eq!(cert.subject, decoded.subject);
        assert_eq!(cert.issuer, decoded.issuer);
    }

    #[test]
    fn test_certificate_builder() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = CertificateBuilder::new(
            "user@example.com".to_string(),
            user_keypair.public_key().clone(),
        )
        .validity(Duration::from_secs(7200))
        .purpose("authentication".to_string())
        .attribute("role".to_string(), "admin".to_string())
        .sign(&ca)
        .unwrap();

        assert_eq!(cert.subject, "user@example.com");
        assert_eq!(
            cert.metadata.purpose,
            Some("authentication".to_string())
        );
        assert_eq!(
            cert.metadata.attributes.get("role"),
            Some(&"admin".to_string())
        );
    }

    #[test]
    fn test_verify_wrong_issuer() {
        let ca1 = CertificateAuthority::generate("CA 1".to_string()).unwrap();
        let ca2 = CertificateAuthority::generate("CA 2".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca1
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(3600),
                CertificateMetadata::default(),
            )
            .unwrap();

        let result = ca2.verify_certificate(&cert);
        assert!(result.is_err());
    }

    #[test]
    fn test_remaining_validity() {
        let ca = CertificateAuthority::generate("Test CA".to_string()).unwrap();
        let user_keypair = Ed25519KeyPair::generate().unwrap();

        let cert = ca
            .issue_certificate(
                "user@example.com".to_string(),
                user_keypair.public_key().clone(),
                Duration::from_secs(3600),
                CertificateMetadata::default(),
            )
            .unwrap();

        let remaining = cert.remaining_validity().unwrap();
        assert!(remaining.as_secs() > 3590);
        assert!(remaining.as_secs() <= 3600);
    }
}
