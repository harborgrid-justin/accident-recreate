//! Role-Based Access Control (RBAC) module
//!
//! This module provides a comprehensive RBAC implementation with:
//! - Hierarchical role inheritance
//! - Fine-grained permissions
//! - Policy-based access control
//! - Context-aware evaluation

pub mod role;
pub mod permission;
pub mod policy;
pub mod context;
pub mod evaluator;

pub use role::{Role, RoleHierarchy};
pub use permission::{Permission, Action, Resource};
pub use policy::{Policy, PolicyRule, Effect};
pub use context::AccessContext;
pub use evaluator::PolicyEvaluator;
