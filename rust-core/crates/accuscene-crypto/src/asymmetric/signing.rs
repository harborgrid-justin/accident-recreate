//! Ed25519 digital signatures
//!
//! Provides message signing and verification using Ed25519.

use crate::asymmetric::keypair::{Ed25519KeyPair, Ed25519PublicKey};
use crate::error::{CryptoError, CryptoResult};
use ed25519_dalek::{Signer as DalekSigner, Verifier};
use serde::{Deserialize, Serialize};

/// Ed25519 signature (64 bytes)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    bytes: [u8; 64],
}

impl Signature {
    /// Create a signature from bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self { bytes }
    }

    /// Create a signature from a slice
    pub fn from_slice(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid signature size: expected 64, got {}",
                bytes.len()
            )));
        }
        let mut array = [0u8; 64];
        array.copy_from_slice(bytes);
        Ok(Self { bytes: array })
    }

    /// Get the signature bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.bytes
    }

    /// Convert to a byte array
    pub fn to_bytes(&self) -> [u8; 64] {
        self.bytes
    }

    /// Encode to base64
    pub fn to_base64(&self) -> String {
        base64::encode(self.bytes)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let bytes = base64::decode(encoded)?;
        Self::from_slice(&bytes)
    }

    /// Convert to Ed25519-dalek signature
    pub(crate) fn to_dalek_signature(&self) -> ed25519_dalek::Signature {
        ed25519_dalek::Signature::from_bytes(&self.bytes)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Ed25519 signer
pub struct Ed25519Signer {
    keypair: Ed25519KeyPair,
}

impl Ed25519Signer {
    /// Create a new signer with the given key pair
    pub fn new(keypair: Ed25519KeyPair) -> Self {
        Self { keypair }
    }

    /// Create a new signer with a newly generated key pair
    pub fn generate() -> CryptoResult<Self> {
        let keypair = Ed25519KeyPair::generate()?;
        Ok(Self { keypair })
    }

    /// Get the public key
    pub fn public_key(&self) -> &Ed25519PublicKey {
        self.keypair.public_key()
    }

    /// Get the key pair
    pub fn keypair(&self) -> &Ed25519KeyPair {
        &self.keypair
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> CryptoResult<Signature> {
        let secret = self.keypair.secret_key().to_dalek_secret_key();
        let public = self.keypair.public_key().to_dalek_public_key()?;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret.to_bytes());
        let signature = signing_key.sign(message);

        Ok(Signature::from_bytes(signature.to_bytes()))
    }

    /// Sign a message and return both signature and public key
    pub fn sign_detached(&self, message: &[u8]) -> CryptoResult<(Signature, Ed25519PublicKey)> {
        let signature = self.sign(message)?;
        Ok((signature, self.public_key().clone()))
    }
}

/// Sign a message with the given key pair
pub fn sign_message(keypair: &Ed25519KeyPair, message: &[u8]) -> CryptoResult<Signature> {
    let signer = Ed25519Signer::new(keypair.clone());
    signer.sign(message)
}

/// Verify a signature on a message
pub fn verify_signature(
    public_key: &Ed25519PublicKey,
    message: &[u8],
    signature: &Signature,
) -> CryptoResult<bool> {
    let dalek_public = public_key.to_dalek_public_key()?;
    let dalek_signature = signature.to_dalek_signature();

    match dalek_public.verify(message, &dalek_signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// A signed message containing the message and its signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    /// The original message
    pub message: Vec<u8>,
    /// The signature
    pub signature: Signature,
    /// The public key of the signer
    pub public_key: Ed25519PublicKey,
}

impl SignedMessage {
    /// Create a new signed message
    pub fn new(message: Vec<u8>, signature: Signature, public_key: Ed25519PublicKey) -> Self {
        Self {
            message,
            signature,
            public_key,
        }
    }

    /// Sign a message with the given signer
    pub fn sign(signer: &Ed25519Signer, message: Vec<u8>) -> CryptoResult<Self> {
        let signature = signer.sign(&message)?;
        Ok(Self {
            message,
            signature,
            public_key: signer.public_key().clone(),
        })
    }

    /// Verify the signature on this message
    pub fn verify(&self) -> CryptoResult<bool> {
        verify_signature(&self.public_key, &self.message, &self.signature)
    }

    /// Encode to base64
    pub fn to_base64(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json.as_bytes()))
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let json_bytes = base64::decode(encoded)?;
        let json_str = std::str::from_utf8(&json_bytes)
            .map_err(|e| CryptoError::DecodingError(e.to_string()))?;
        let data = serde_json::from_str(json_str)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Hello, World!";

        let signature = signer.sign(message).unwrap();
        let verified = verify_signature(signer.public_key(), message, &signature).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_wrong_message_fails() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Hello, World!";
        let wrong_message = b"Goodbye, World!";

        let signature = signer.sign(message).unwrap();
        let verified = verify_signature(signer.public_key(), wrong_message, &signature).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_verify_wrong_public_key_fails() {
        let signer1 = Ed25519Signer::generate().unwrap();
        let signer2 = Ed25519Signer::generate().unwrap();
        let message = b"Hello, World!";

        let signature = signer1.sign(message).unwrap();
        let verified = verify_signature(signer2.public_key(), message, &signature).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_signature_base64_roundtrip() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Test message";

        let signature = signer.sign(message).unwrap();
        let encoded = signature.to_base64();
        let decoded = Signature::from_base64(&encoded).unwrap();

        assert_eq!(signature.as_bytes(), decoded.as_bytes());
    }

    #[test]
    fn test_signed_message() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Secret message".to_vec();

        let signed = SignedMessage::sign(&signer, message.clone()).unwrap();
        assert_eq!(signed.message, message);
        assert!(signed.verify().unwrap());
    }

    #[test]
    fn test_signed_message_base64_roundtrip() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Test message".to_vec();

        let signed = SignedMessage::sign(&signer, message).unwrap();
        let encoded = signed.to_base64().unwrap();
        let decoded = SignedMessage::from_base64(&encoded).unwrap();

        assert_eq!(signed.message, decoded.message);
        assert_eq!(signed.signature.as_bytes(), decoded.signature.as_bytes());
        assert!(decoded.verify().unwrap());
    }

    #[test]
    fn test_tampered_signed_message() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Original message".to_vec();

        let mut signed = SignedMessage::sign(&signer, message).unwrap();
        signed.message = b"Tampered message".to_vec();

        assert!(!signed.verify().unwrap());
    }

    #[test]
    fn test_sign_detached() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"Test message";

        let (signature, public_key) = signer.sign_detached(message).unwrap();
        assert_eq!(&public_key, signer.public_key());
        assert!(verify_signature(&public_key, message, &signature).unwrap());
    }

    #[test]
    fn test_empty_message() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = b"";

        let signature = signer.sign(message).unwrap();
        assert!(verify_signature(signer.public_key(), message, &signature).unwrap());
    }

    #[test]
    fn test_large_message() {
        let signer = Ed25519Signer::generate().unwrap();
        let message = vec![42u8; 1024 * 1024]; // 1 MB

        let signature = signer.sign(&message).unwrap();
        assert!(verify_signature(signer.public_key(), &message, &signature).unwrap());
    }
}
