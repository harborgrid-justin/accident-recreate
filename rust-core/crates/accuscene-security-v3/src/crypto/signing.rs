//! Ed25519 digital signatures
//!
//! Provides digital signature generation and verification using Ed25519.

use crate::error::{SecurityError, SecurityResult};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Ed25519 signer for digital signatures
#[derive(Debug)]
pub struct Ed25519Signer {
    signing_key: SigningKey,
}

impl Ed25519Signer {
    /// Generate a new signing key pair
    pub fn generate() -> SecurityResult<(Self, PublicKey)> {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        Ok((
            Self { signing_key },
            PublicKey {
                key: verifying_key.to_bytes().to_vec(),
            },
        ))
    }

    /// Create from existing private key bytes
    pub fn from_bytes(bytes: &[u8]) -> SecurityResult<Self> {
        if bytes.len() != 32 {
            return Err(SecurityError::InvalidKey(
                "Ed25519 private key must be 32 bytes".to_string(),
            ));
        }

        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| SecurityError::InvalidKey("Invalid key length".to_string()))?;

        let signing_key = SigningKey::from_bytes(&key_bytes);

        Ok(Self { signing_key })
    }

    /// Get the private key bytes (use with caution!)
    pub fn to_bytes(&self) -> Zeroizing<Vec<u8>> {
        Zeroizing::new(self.signing_key.to_bytes().to_vec())
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            key: self.signing_key.verifying_key().to_bytes().to_vec(),
        }
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signature = self.signing_key.sign(message);
        Signature {
            bytes: signature.to_bytes().to_vec(),
        }
    }

    /// Sign and encode as base64
    pub fn sign_to_base64(&self, message: &[u8]) -> String {
        let signature = self.sign(message);
        base64::encode(&signature.bytes)
    }
}

/// Public key for signature verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    /// Public key bytes
    pub key: Vec<u8>,
}

impl PublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> SecurityResult<Self> {
        if bytes.len() != 32 {
            return Err(SecurityError::InvalidKey(
                "Ed25519 public key must be 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            key: bytes.to_vec(),
        })
    }

    /// Get bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.key
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> SecurityResult<bool> {
        if signature.bytes.len() != 64 {
            return Err(SecurityError::InvalidInput(
                "Invalid signature length".to_string(),
            ));
        }

        let key_bytes: [u8; 32] = self.key.as_slice().try_into()
            .map_err(|_| SecurityError::InvalidKey("Invalid key length".to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| SecurityError::InvalidKey(format!("Invalid public key: {}", e)))?;

        let sig_bytes: [u8; 64] = signature.bytes.as_slice().try_into()
            .map_err(|_| SecurityError::InvalidInput("Invalid signature length".to_string()))?;

        let signature = Ed25519Signature::from_bytes(&sig_bytes);

        match verifying_key.verify(message, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify a base64-encoded signature
    pub fn verify_base64(&self, message: &[u8], signature_b64: &str) -> SecurityResult<bool> {
        let sig_bytes = base64::decode(signature_b64)
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid base64: {}", e)))?;

        let signature = Signature { bytes: sig_bytes };
        self.verify(message, &signature)
    }

    /// Encode to base64
    pub fn to_base64(&self) -> String {
        base64::encode(&self.key)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> SecurityResult<Self> {
        let bytes = base64::decode(encoded)
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid base64: {}", e)))?;

        Self::from_bytes(&bytes)
    }
}

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature bytes
    pub bytes: Vec<u8>,
}

impl Signature {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Get bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Encode to base64
    pub fn to_base64(&self) -> String {
        base64::encode(&self.bytes)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> SecurityResult<Self> {
        let bytes = base64::decode(encoded)
            .map_err(|e| SecurityError::InvalidInput(format!("Invalid base64: {}", e)))?;

        Ok(Self { bytes })
    }
}

/// Signed data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedData {
    /// The data
    pub data: Vec<u8>,

    /// The signature
    pub signature: Signature,

    /// The public key that signed it
    pub public_key: PublicKey,
}

impl SignedData {
    /// Create signed data
    pub fn new(data: Vec<u8>, signer: &Ed25519Signer) -> Self {
        let signature = signer.sign(&data);
        let public_key = signer.public_key();

        Self {
            data,
            signature,
            public_key,
        }
    }

    /// Verify the signature
    pub fn verify(&self) -> SecurityResult<bool> {
        self.public_key.verify(&self.data, &self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let (signer, public_key) = Ed25519Signer::generate().unwrap();
        let message = b"Test message";

        let signature = signer.sign(message);
        assert_eq!(signature.bytes.len(), 64);

        assert!(public_key.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_signature_verification_failure() {
        let (signer, public_key) = Ed25519Signer::generate().unwrap();
        let message = b"Test message";

        let signature = signer.sign(message);

        // Verify with wrong message
        assert!(!public_key.verify(b"Wrong message", &signature).unwrap());
    }

    #[test]
    fn test_base64_encoding() {
        let (signer, public_key) = Ed25519Signer::generate().unwrap();
        let message = b"Test message";

        let signature_b64 = signer.sign_to_base64(message);
        assert!(public_key.verify_base64(message, &signature_b64).unwrap());
    }

    #[test]
    fn test_key_serialization() {
        let (signer, public_key) = Ed25519Signer::generate().unwrap();

        // Serialize and deserialize private key
        let private_bytes = signer.to_bytes();
        let signer2 = Ed25519Signer::from_bytes(&private_bytes).unwrap();

        // Verify they produce the same signatures
        let message = b"Test message";
        let sig1 = signer.sign(message);
        let sig2 = signer2.sign(message);

        assert_eq!(sig1.bytes, sig2.bytes);
    }

    #[test]
    fn test_public_key_base64() {
        let (_, public_key) = Ed25519Signer::generate().unwrap();

        let encoded = public_key.to_base64();
        let decoded = PublicKey::from_base64(&encoded).unwrap();

        assert_eq!(public_key.key, decoded.key);
    }

    #[test]
    fn test_signed_data() {
        let (signer, _) = Ed25519Signer::generate().unwrap();
        let data = b"Important data".to_vec();

        let signed = SignedData::new(data.clone(), &signer);

        assert!(signed.verify().unwrap());
        assert_eq!(signed.data, data);
    }

    #[test]
    fn test_tampered_signed_data() {
        let (signer, _) = Ed25519Signer::generate().unwrap();
        let data = b"Important data".to_vec();

        let mut signed = SignedData::new(data, &signer);

        // Tamper with data
        signed.data[0] ^= 1;

        // Verification should fail
        assert!(!signed.verify().unwrap());
    }

    #[test]
    fn test_invalid_key_length() {
        let result = Ed25519Signer::from_bytes(&[0u8; 16]);
        assert!(result.is_err());

        let result = PublicKey::from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }
}
