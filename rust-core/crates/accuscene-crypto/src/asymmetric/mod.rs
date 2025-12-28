//! Asymmetric cryptography
//!
//! Provides Ed25519 digital signatures and X25519 key exchange.

pub mod ecdh;
pub mod keypair;
pub mod signing;

pub use self::ecdh::{perform_key_exchange, X25519KeyPair, X25519PublicKey, X25519SecretKey};
pub use self::keypair::{Ed25519KeyPair, Ed25519PublicKey, Ed25519SecretKey};
pub use self::signing::{sign_message, verify_signature, Ed25519Signer, Signature};
