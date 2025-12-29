//! Error types for the AccuScene physics engine.
//!
//! This module provides comprehensive error handling for all physics operations,
//! including numerical instabilities, convergence failures, and invalid configurations.

use thiserror::Error;

/// Result type alias for physics operations.
pub type PhysicsResult<T> = Result<T, PhysicsError>;

/// Comprehensive error types for physics engine operations.
#[derive(Error, Debug, Clone)]
pub enum PhysicsError {
    /// Numerical instability detected in simulation.
    #[error("Numerical instability: {details}")]
    NumericalInstability { details: String },

    /// Solver failed to converge within iteration limit.
    #[error("Solver convergence failure: {0} iterations exceeded tolerance {1}")]
    ConvergenceFailure(usize, f64),

    /// Invalid configuration parameter.
    #[error("Invalid configuration: {parameter} = {value}, expected {constraint}")]
    InvalidConfiguration {
        parameter: String,
        value: String,
        constraint: String,
    },

    /// Singular matrix encountered (non-invertible).
    #[error("Singular matrix in {operation}: determinant = {determinant}")]
    SingularMatrix { operation: String, determinant: f64 },

    /// Invalid physical state (e.g., negative mass, infinite velocity).
    #[error("Invalid physical state: {0}")]
    InvalidPhysicalState(String),

    /// Collision detection failure.
    #[error("Collision detection error: {0}")]
    CollisionDetectionError(String),

    /// Constraint violation beyond tolerance.
    #[error("Constraint violation: {constraint_type}, error = {error}")]
    ConstraintViolation { constraint_type: String, error: f64 },

    /// Deformation energy calculation error.
    #[error("Deformation energy error: {0}")]
    DeformationError(String),

    /// Vehicle dynamics error.
    #[error("Vehicle dynamics error: {0}")]
    VehicleDynamicsError(String),

    /// Thread synchronization error.
    #[error("Thread synchronization error: {0}")]
    SyncError(String),

    /// Generic physics error.
    #[error("Physics error: {0}")]
    Generic(String),
}

impl PhysicsError {
    /// Creates a new numerical instability error.
    pub fn numerical_instability(details: impl Into<String>) -> Self {
        Self::NumericalInstability {
            details: details.into(),
        }
    }

    /// Creates a new invalid physical state error.
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidPhysicalState(message.into())
    }

    /// Creates a new generic error.
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = PhysicsError::numerical_instability("Integration step too large");
        assert!(error.to_string().contains("Numerical instability"));
    }

    #[test]
    fn test_convergence_failure() {
        let error = PhysicsError::ConvergenceFailure(100, 1e-6);
        assert!(error.to_string().contains("100"));
        assert!(error.to_string().contains("0.000001"));
    }
}
