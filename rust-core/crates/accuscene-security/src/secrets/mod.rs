//! Secrets management

pub mod rotation;
pub mod vault;

pub use rotation::{needs_rotation, RotationPolicy};
pub use vault::SecretVault;
