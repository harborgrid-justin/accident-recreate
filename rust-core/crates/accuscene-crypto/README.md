# AccuScene Cryptographic Security Module

Enterprise-grade cryptographic operations for the AccuScene accident recreation platform.

## Overview

This crate provides a comprehensive cryptographic security module implementing modern, audited algorithms with a focus on security, performance, and ease of use.

## Features

### Core Cryptography
- **Secure Random Generation**: ChaCha20-based CSPRNG
- **Hashing**: SHA-256, SHA-512, BLAKE3
- **Password Hashing**: Argon2id (recommended), scrypt
- **Symmetric Encryption**: AES-256-GCM, ChaCha20-Poly1305
- **Asymmetric Cryptography**: Ed25519 signatures, X25519 key exchange
- **Key Derivation**: HKDF, PBKDF2

### High-Level Features
- **Envelope Encryption**: DEK/KEK pattern for large data
- **Secure Vault**: In-memory key storage with automatic zeroization
- **Token System**: Secure token generation and validation with expiration
- **Integrity Verification**: HMAC and digital signature-based file verification
- **Certificate System**: Simple PKI for public key distribution

## Security Properties

- All keys and sensitive data automatically zeroized on drop
- Constant-time comparisons for all security-critical operations
- Only authenticated encryption (AEAD) algorithms
- Modern, audited cryptographic primitives
- Memory-safe implementation (no unsafe code)

## Usage Examples

### Symmetric Encryption

```rust
use accuscene_crypto::prelude::*;

// Generate a key
let key = SymmetricKey::generate()?;

// Encrypt data
let plaintext = b"Secret data";
let encrypted = encrypt_aes256gcm(&key, plaintext, None)?;

// Decrypt data
let decrypted = decrypt_aes256gcm(&key, &encrypted, None)?;
```

### Password Hashing

```rust
use accuscene_crypto::hash::{hash_password, verify_password};

// Hash a password
let password = "user_password";
let hash = hash_password(password)?;

// Verify password
let is_valid = verify_password(password, &hash)?;
```

### Digital Signatures

```rust
use accuscene_crypto::asymmetric::*;

// Generate keypair
let signer = Ed25519Signer::generate()?;

// Sign a message
let message = b"Important message";
let signature = signer.sign(message)?;

// Verify signature
let verified = verify_signature(signer.public_key(), message, &signature)?;
```

### Key Exchange

```rust
use accuscene_crypto::asymmetric::*;

// Alice generates keypair
let alice = X25519KeyPair::generate()?;

// Bob generates keypair
let bob = X25519KeyPair::generate()?;

// Both derive the same shared secret
let alice_shared = alice.exchange(bob.public_key())?;
let bob_shared = bob.exchange(alice.public_key())?;
```

### Secure Vault

```rust
use accuscene_crypto::vault::Vault;

// Create a vault
let mut vault = Vault::generate()?;

// Store secrets
vault.store("api_key".to_string(), b"secret_key_123")?;

// Retrieve secrets
let secret = vault.retrieve("api_key")?;
```

## Module Structure

```
accuscene-crypto/
├── src/
│   ├── lib.rs                 - Public API and prelude
│   ├── error.rs               - Error types
│   ├── random.rs              - Secure random generation
│   ├── secure_memory.rs       - Memory protection
│   ├── hash/
│   │   ├── mod.rs
│   │   ├── sha.rs             - SHA-256/512
│   │   ├── blake3.rs          - BLAKE3
│   │   └── password.rs        - Password hashing
│   ├── symmetric/
│   │   ├── mod.rs
│   │   ├── aes.rs             - AES-256-GCM
│   │   ├── chacha.rs          - ChaCha20-Poly1305
│   │   └── key.rs             - Key management
│   ├── asymmetric/
│   │   ├── mod.rs
│   │   ├── keypair.rs         - Ed25519 keypairs
│   │   ├── signing.rs         - Ed25519 signatures
│   │   └── ecdh.rs            - X25519 key exchange
│   ├── kdf.rs                 - Key derivation
│   ├── envelope.rs            - Envelope encryption
│   ├── vault.rs               - Secure vault
│   ├── token.rs               - Token management
│   ├── integrity.rs           - File integrity
│   └── certificate.rs         - Certificate system
└── Cargo.toml
```

## Dependencies

### Cryptographic Libraries
- `ring` - Core cryptographic primitives
- `aes-gcm` - AES-256-GCM authenticated encryption
- `chacha20poly1305` - ChaCha20-Poly1305 authenticated encryption
- `argon2` - Argon2id password hashing
- `scrypt` - scrypt password hashing
- `ed25519-dalek` - Ed25519 signatures
- `x25519-dalek` - X25519 key exchange
- `sha2` - SHA-2 family hashing
- `blake3` - BLAKE3 hashing

### Utilities
- `zeroize` - Secure memory zeroing
- `subtle` - Constant-time operations
- `rand` / `rand_chacha` - Secure random generation
- `serde` - Serialization
- `base64` - Base64 encoding

## Security Considerations

1. **Key Management**: All keys are automatically zeroized when dropped
2. **Constant-Time**: All comparisons use constant-time algorithms
3. **Authenticated Encryption**: Only AEAD ciphers are used
4. **Modern Algorithms**: Using current best practices (Argon2id, Ed25519, etc.)
5. **No Unsafe Code**: Pure safe Rust implementation

## Testing

```bash
cargo test --package accuscene-crypto
```

## Benchmarks

```bash
cargo bench --package accuscene-crypto
```

## License

MIT OR Apache-2.0

## Version

0.1.5
