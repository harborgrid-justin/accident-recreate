//! File integrity verification
//!
//! Provides HMAC and digital signature-based file integrity verification.

use crate::asymmetric::signing::{verify_signature, Ed25519Signer, Signature};
use crate::asymmetric::Ed25519PublicKey;
use crate::error::{CryptoError, CryptoResult};
use crate::hash::blake3::blake3_hash_file;
use crate::hash::sha::{sha256_file, sha512_file};
use crate::symmetric::key::SymmetricKey;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// File integrity verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityProof {
    /// The hash of the file
    pub hash: String,
    /// The algorithm used
    pub algorithm: IntegrityAlgorithm,
    /// Optional HMAC for authenticated integrity
    pub hmac: Option<String>,
    /// Optional digital signature for non-repudiation
    pub signature: Option<Signature>,
    /// Public key used for signature (if signed)
    pub public_key: Option<Ed25519PublicKey>,
}

/// Integrity verification algorithm
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrityAlgorithm {
    /// SHA-256 hash
    Sha256,
    /// SHA-512 hash
    Sha512,
    /// BLAKE3 hash
    Blake3,
    /// HMAC-SHA256
    HmacSha256,
    /// Ed25519 signature
    Ed25519,
}

impl IntegrityProof {
    /// Encode to base64 for storage
    pub fn to_base64(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json.as_bytes()))
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let json_bytes = base64::decode(encoded)?;
        let json_str = std::str::from_utf8(&json_bytes)
            .map_err(|e| CryptoError::DecodingError(e.to_string()))?;
        let proof = serde_json::from_str(json_str)?;
        Ok(proof)
    }
}

/// File integrity verifier
pub struct IntegrityVerifier;

impl IntegrityVerifier {
    /// Compute a SHA-256 hash of a file
    pub fn hash_sha256<P: AsRef<Path>>(path: P) -> CryptoResult<IntegrityProof> {
        let hash = sha256_file(path)?;
        Ok(IntegrityProof {
            hash: hex::encode(hash),
            algorithm: IntegrityAlgorithm::Sha256,
            hmac: None,
            signature: None,
            public_key: None,
        })
    }

    /// Compute a SHA-512 hash of a file
    pub fn hash_sha512<P: AsRef<Path>>(path: P) -> CryptoResult<IntegrityProof> {
        let hash = sha512_file(path)?;
        Ok(IntegrityProof {
            hash: hex::encode(hash),
            algorithm: IntegrityAlgorithm::Sha512,
            hmac: None,
            signature: None,
            public_key: None,
        })
    }

    /// Compute a BLAKE3 hash of a file
    pub fn hash_blake3<P: AsRef<Path>>(path: P) -> CryptoResult<IntegrityProof> {
        let hash = blake3_hash_file(path)?;
        Ok(IntegrityProof {
            hash: hex::encode(hash),
            algorithm: IntegrityAlgorithm::Blake3,
            hmac: None,
            signature: None,
            public_key: None,
        })
    }

    /// Compute an HMAC-SHA256 of a file
    pub fn hmac_sha256<P: AsRef<Path>>(path: P, key: &SymmetricKey) -> CryptoResult<IntegrityProof> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);

        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key.as_bytes());
        let mut context = hmac::Context::with_key(&hmac_key);

        let mut buffer = [0u8; 8192];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }

        let tag = context.sign();
        let hash = hex::encode(tag.as_ref());

        Ok(IntegrityProof {
            hash: hash.clone(),
            algorithm: IntegrityAlgorithm::HmacSha256,
            hmac: Some(hash),
            signature: None,
            public_key: None,
        })
    }

    /// Sign a file with Ed25519
    pub fn sign_file<P: AsRef<Path>>(
        path: P,
        signer: &Ed25519Signer,
    ) -> CryptoResult<IntegrityProof> {
        // First compute the hash
        let hash = blake3_hash_file(path.as_ref())?;
        let hash_hex = hex::encode(hash);

        // Sign the hash
        let signature = signer.sign(&hash)?;

        Ok(IntegrityProof {
            hash: hash_hex,
            algorithm: IntegrityAlgorithm::Ed25519,
            hmac: None,
            signature: Some(signature),
            public_key: Some(signer.public_key().clone()),
        })
    }

    /// Verify a file against an integrity proof
    pub fn verify<P: AsRef<Path>>(path: P, proof: &IntegrityProof) -> CryptoResult<bool> {
        match proof.algorithm {
            IntegrityAlgorithm::Sha256 => {
                let current = Self::hash_sha256(path)?;
                Ok(current.hash == proof.hash)
            }
            IntegrityAlgorithm::Sha512 => {
                let current = Self::hash_sha512(path)?;
                Ok(current.hash == proof.hash)
            }
            IntegrityAlgorithm::Blake3 => {
                let current = Self::hash_blake3(path)?;
                Ok(current.hash == proof.hash)
            }
            IntegrityAlgorithm::HmacSha256 => {
                Err(CryptoError::IntegrityCheckFailed(
                    "Cannot verify HMAC without key".to_string(),
                ))
            }
            IntegrityAlgorithm::Ed25519 => {
                let hash = blake3_hash_file(path)?;
                let signature = proof
                    .signature
                    .as_ref()
                    .ok_or_else(|| CryptoError::IntegrityCheckFailed("No signature".to_string()))?;
                let public_key = proof.public_key.as_ref().ok_or_else(|| {
                    CryptoError::IntegrityCheckFailed("No public key".to_string())
                })?;

                verify_signature(public_key, &hash, signature)
            }
        }
    }

    /// Verify a file against an HMAC proof
    pub fn verify_hmac<P: AsRef<Path>>(
        path: P,
        proof: &IntegrityProof,
        key: &SymmetricKey,
    ) -> CryptoResult<bool> {
        if proof.algorithm != IntegrityAlgorithm::HmacSha256 {
            return Err(CryptoError::IntegrityCheckFailed(
                "Proof is not an HMAC".to_string(),
            ));
        }

        let current = Self::hmac_sha256(path, key)?;
        Ok(current.hmac == proof.hmac)
    }
}

/// Compute a checksum of data in memory
pub fn checksum_sha256(data: &[u8]) -> String {
    use crate::hash::sha::sha256;
    hex::encode(sha256(data))
}

/// Compute a BLAKE3 checksum of data in memory
pub fn checksum_blake3(data: &[u8]) -> String {
    use crate::hash::blake3::blake3_hash;
    hex::encode(blake3_hash(data))
}

/// Compute an HMAC of data in memory
pub fn hmac_sha256(data: &[u8], key: &SymmetricKey) -> String {
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key.as_bytes());
    let tag = hmac::sign(&hmac_key, data);
    hex::encode(tag.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hash_sha256_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hash_sha256(temp_file.path()).unwrap();
        assert!(!proof.hash.is_empty());
        assert_eq!(proof.algorithm, IntegrityAlgorithm::Sha256);
    }

    #[test]
    fn test_hash_blake3_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hash_blake3(temp_file.path()).unwrap();
        assert!(!proof.hash.is_empty());
        assert_eq!(proof.algorithm, IntegrityAlgorithm::Blake3);
    }

    #[test]
    fn test_verify_hash() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hash_sha256(temp_file.path()).unwrap();
        let verified = IntegrityVerifier::verify(temp_file.path(), &proof).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_modified_file_fails() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hash_sha256(temp_file.path()).unwrap();

        // Modify the file
        temp_file.write_all(b" modified").unwrap();
        temp_file.flush().unwrap();

        let verified = IntegrityVerifier::verify(temp_file.path(), &proof).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_hmac_file() {
        let key = SymmetricKey::generate().unwrap();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hmac_sha256(temp_file.path(), &key).unwrap();
        assert!(proof.hmac.is_some());
        assert_eq!(proof.algorithm, IntegrityAlgorithm::HmacSha256);
    }

    #[test]
    fn test_verify_hmac() {
        let key = SymmetricKey::generate().unwrap();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hmac_sha256(temp_file.path(), &key).unwrap();
        let verified = IntegrityVerifier::verify_hmac(temp_file.path(), &proof, &key).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_hmac_wrong_key_fails() {
        let key1 = SymmetricKey::generate().unwrap();
        let key2 = SymmetricKey::generate().unwrap();

        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::hmac_sha256(temp_file.path(), &key1).unwrap();
        let verified = IntegrityVerifier::verify_hmac(temp_file.path(), &proof, &key2).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_sign_and_verify_file() {
        let signer = Ed25519Signer::generate().unwrap();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test data").unwrap();
        temp_file.flush().unwrap();

        let proof = IntegrityVerifier::sign_file(temp_file.path(), &signer).unwrap();
        assert!(proof.signature.is_some());
        assert!(proof.public_key.is_some());

        let verified = IntegrityVerifier::verify(temp_file.path(), &proof).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_checksum_sha256() {
        let data = b"test data";
        let checksum = checksum_sha256(data);
        assert_eq!(checksum.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_checksum_blake3() {
        let data = b"test data";
        let checksum = checksum_blake3(data);
        assert_eq!(checksum.len(), 64); // BLAKE3 hex = 64 chars
    }

    #[test]
    fn test_hmac_sha256_data() {
        let key = SymmetricKey::generate().unwrap();
        let data = b"test data";
        let hmac = hmac_sha256(data, &key);
        assert_eq!(hmac.len(), 64); // HMAC-SHA256 hex = 64 chars
    }

    #[test]
    fn test_integrity_proof_base64_roundtrip() {
        let proof = IntegrityProof {
            hash: "abc123".to_string(),
            algorithm: IntegrityAlgorithm::Sha256,
            hmac: None,
            signature: None,
            public_key: None,
        };

        let encoded = proof.to_base64().unwrap();
        let decoded = IntegrityProof::from_base64(&encoded).unwrap();

        assert_eq!(proof.hash, decoded.hash);
        assert_eq!(proof.algorithm, decoded.algorithm);
    }
}
