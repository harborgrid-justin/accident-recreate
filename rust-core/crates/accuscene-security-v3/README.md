# AccuScene Enterprise Security v0.3.0

Enterprise-grade security, RBAC, authentication, cryptography, auditing, and compliance for AccuScene.

## Features

### Role-Based Access Control (RBAC)
- Hierarchical role inheritance
- Fine-grained permissions (CRUD + custom actions)
- Policy-based access control with rules
- Context-aware policy evaluation
- Predefined system roles (Super Admin, Admin, Manager, Editor, Viewer)

### Authentication
- **Password Hashing**: Argon2id with configurable parameters
- **JWT Tokens**: Access and refresh token management
- **Session Management**: With fingerprinting and concurrent session limits
- **Multi-Factor Authentication (MFA)**:
  - TOTP (Time-based One-Time Passwords)
  - WebAuthn (Hardware keys and biometrics)
  - Backup codes
- **OAuth 2.0 / OpenID Connect**: Support for Google, Microsoft, GitHub, Okta
- **SAML 2.0**: Enterprise SSO integration

### Cryptography
- **AES-256-GCM**: Authenticated encryption
- **Ed25519**: Digital signatures
- **Key Derivation**: HKDF and PBKDF2
- **Secure Random**: Cryptographically secure RNG
- **Envelope Encryption**: For large data
- Zero-copy sensitive data handling
- Constant-time comparisons

### Audit Logging
- Immutable audit log with tamper detection
- Chain integrity verification
- Comprehensive event types
- Real-time analysis and alerting
- Pattern detection (failed logins, suspicious activity, etc.)
- Configurable log retention

### Compliance
- **GDPR**:
  - Consent management
  - Data subject requests (Access, Rectification, Erasure, etc.)
  - Right to be forgotten
  - Data portability
- **Data Retention**: Configurable policies with legal hold support
- **Data Export**: JSON, CSV, XML formats with checksums
- SOC2/ISO27001 compliance support

## Security Guarantees

- Zero-copy sensitive data handling where possible
- Constant-time comparisons to prevent timing attacks
- Automatic memory zeroization for sensitive data (using `zeroize` crate)
- Comprehensive logging without leaking secrets
- Tamper-evident audit logs

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
accuscene-security-v3 = { path = "../accuscene-security-v3" }
```

### Features

- `default`: Includes async support
- `async`: Async runtime support (Tokio)
- `saml`: SAML 2.0 SSO support
- `full`: All features enabled

## Quick Start

### Password Hashing

```rust
use accuscene_security_v3::auth::password::PasswordHasher;

let hasher = PasswordHasher::new();
let hash = hasher.hash("SecurePassword123!").unwrap();
let is_valid = hasher.verify("SecurePassword123!", &hash).unwrap();
```

### JWT Token Management

```rust
use accuscene_security_v3::auth::jwt::JwtManager;
use accuscene_security_v3::config::JwtConfig;

let config = JwtConfig {
    secret: "your-secret-key".to_string(),
    issuer: "accuscene".to_string(),
    audience: "accuscene-api".to_string(),
    access_token_expiry_secs: 900,
    refresh_token_expiry_secs: 604800,
    algorithm: "HS256".to_string(),
};

let manager = JwtManager::new(config).unwrap();
let token_pair = manager.generate_token_pair(
    "user123",
    vec!["admin".to_string()]
).unwrap();
```

### RBAC

```rust
use accuscene_security_v3::rbac::{
    PolicyEvaluator, Action, Resource,
    AccessContext, UserContext, ResourceContext
};

// Create evaluator with system roles
let evaluator = PolicyEvaluator::with_system_roles();

// Create access context
let user = UserContext::new("user123", "john.doe")
    .with_role("admin");

let resource = ResourceContext::new("scene")
    .with_id("scene456");

let context = AccessContext::new(user, resource);

// Evaluate permission
let allowed = evaluator.evaluate(
    &context,
    &Action::Delete,
    &Resource::new("scene")
).unwrap();
```

### Encryption

```rust
use accuscene_security_v3::crypto::encryption::AesGcmEncryptor;

let (encryptor, key) = AesGcmEncryptor::new().unwrap();
let plaintext = b"Secret message";

let encrypted = encryptor.encrypt(plaintext).unwrap();
let decrypted = encryptor.decrypt(&encrypted).unwrap();

assert_eq!(&decrypted[..], plaintext);
```

### Audit Logging

```rust
use accuscene_security_v3::audit::{AuditLogger, AuditEvent, EventType};
use accuscene_security_v3::config::AuditConfig;

let logger = AuditLogger::new(AuditConfig::default());

// Log an event
let event = AuditEvent::login_success("user123", "192.168.1.1");
logger.log(event).unwrap();

// Verify integrity
assert!(logger.verify_integrity().unwrap());
```

### GDPR Compliance

```rust
use accuscene_security_v3::compliance::gdpr::{
    GdprCompliance, ConsentPurpose, DataSubjectRequest, RequestType
};

let compliance = GdprCompliance::new();

// Record consent
compliance.consent_manager()
    .record_consent("user123", ConsentPurpose::Marketing, true)
    .unwrap();

// Submit data subject request
let request = DataSubjectRequest::new(
    "user123",
    RequestType::Access,
    Some("I want to see my data".to_string())
);

let request_id = compliance.submit_request(request).unwrap();
```

## Configuration

All security features are configurable through `SecurityConfig`:

```rust
use accuscene_security_v3::config::SecurityConfig;

let mut config = SecurityConfig::default();

// Customize password policy
config.password_policy.min_length = 16;
config.password_policy.require_special_chars = true;

// Configure MFA
config.mfa.require_mfa = true;
config.mfa.totp_window = 2;

// Configure audit logging
config.audit.enable_audit_log = true;
config.audit.retention_days = 365;

// Configure compliance
config.compliance.enable_gdpr = true;
config.compliance.default_retention_days = 2555;
```

## Architecture

```
accuscene-security-v3/
├── src/
│   ├── lib.rs                 # Main library exports
│   ├── error.rs               # Error types
│   ├── config.rs              # Configuration
│   ├── rbac/                  # Role-Based Access Control
│   │   ├── mod.rs
│   │   ├── role.rs            # Roles with inheritance
│   │   ├── permission.rs      # Fine-grained permissions
│   │   ├── policy.rs          # Policy engine
│   │   ├── context.rs         # Access control context
│   │   └── evaluator.rs       # Policy evaluation
│   ├── auth/                  # Authentication
│   │   ├── mod.rs
│   │   ├── password.rs        # Argon2id password hashing
│   │   ├── jwt.rs             # JWT tokens
│   │   ├── session.rs         # Session management
│   │   ├── mfa.rs             # Multi-factor authentication
│   │   ├── oauth.rs           # OAuth 2.0 / OIDC
│   │   └── saml.rs            # SAML 2.0 SSO
│   ├── crypto/                # Cryptography
│   │   ├── mod.rs
│   │   ├── encryption.rs      # AES-256-GCM
│   │   ├── signing.rs         # Ed25519 signatures
│   │   ├── key_derivation.rs  # HKDF, PBKDF2
│   │   └── secure_random.rs   # Secure RNG
│   ├── audit/                 # Audit logging
│   │   ├── mod.rs
│   │   ├── logger.rs          # Immutable audit log
│   │   ├── events.rs          # Event types
│   │   └── analyzer.rs        # Log analysis
│   └── compliance/            # Compliance
│       ├── mod.rs
│       ├── data_retention.rs  # Retention policies
│       ├── gdpr.rs            # GDPR compliance
│       └── export.rs          # Data export
└── Cargo.toml
```

## Security Best Practices

1. **Key Management**:
   - Store encryption keys in secure key management systems (KMS)
   - Use environment variables for secrets, never hardcode
   - Rotate keys regularly
   - Use envelope encryption for large data

2. **Password Policies**:
   - Enforce minimum length of 12+ characters
   - Require complexity (uppercase, lowercase, digits, special chars)
   - Implement password history to prevent reuse
   - Use Argon2id with recommended parameters

3. **Session Management**:
   - Use secure session fingerprinting
   - Implement idle timeouts
   - Limit concurrent sessions
   - Invalidate sessions on logout

4. **Audit Logging**:
   - Log all authentication attempts
   - Log authorization decisions
   - Log data access and modifications
   - Verify log integrity regularly
   - Retain logs per compliance requirements

5. **Compliance**:
   - Obtain explicit consent for data processing
   - Honor data subject requests within legal timeframes
   - Implement data retention policies
   - Enable data export for portability

## Testing

Run tests:

```bash
cargo test
```

Run with all features:

```bash
cargo test --all-features
```

## Performance

All cryptographic operations use optimized implementations:
- Argon2id is tuned for balance between security and performance
- AES-256-GCM uses hardware acceleration when available
- Ed25519 is one of the fastest signature schemes
- Zero-copy operations minimize memory allocations

## Compliance Certifications

This crate is designed to support:
- SOC 2 Type II
- ISO 27001
- GDPR
- HIPAA (with proper configuration)
- PCI DSS Level 1

## License

MIT

## Contributing

Security contributions are welcome! Please report security vulnerabilities privately.

## Changelog

### v0.3.0 (2024)
- Initial release
- Complete RBAC system with hierarchical roles
- Multi-factor authentication (TOTP, WebAuthn)
- OAuth 2.0 and SAML 2.0 support
- AES-256-GCM encryption and Ed25519 signatures
- Immutable audit logging with tamper detection
- GDPR compliance utilities
- Data retention policies
