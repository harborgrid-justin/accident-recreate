//! AccuScene Enterprise Security v0.3.0
//!
//! Enterprise-grade security, RBAC, authentication, cryptography, auditing, and compliance.
//!
//! # Features
//!
//! - **RBAC**: Role-based access control with hierarchical inheritance
//! - **Authentication**: Multi-factor authentication, JWT, OAuth 2.0, SAML 2.0
//! - **Cryptography**: AES-256-GCM, Ed25519, secure key derivation
//! - **Audit**: Immutable audit logging with analysis
//! - **Compliance**: GDPR, data retention, export capabilities
//!
//! # Security Guarantees
//!
//! - Zero-copy sensitive data handling where possible
//! - Constant-time comparisons to prevent timing attacks
//! - Automatic memory zeroization for sensitive data
//! - SOC2/ISO27001 compliance support
//!
//! # Example
//!
//! ```rust
//! use accuscene_security_v3::rbac::{Role, Permission, PolicyEvaluator};
//! use accuscene_security_v3::auth::password::PasswordHasher;
//!
//! // Create a role with permissions
//! let mut role = Role::new("admin", "System Administrator");
//! role.add_permission(Permission::all());
//!
//! // Hash a password securely
//! let hasher = PasswordHasher::new();
//! let hash = hasher.hash("secure_password").unwrap();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod rbac;
pub mod auth;
pub mod crypto;
pub mod audit;
pub mod compliance;
pub mod config;
pub mod error;

// Re-export common types
pub use error::{SecurityError, SecurityResult};
pub use config::SecurityConfig;
