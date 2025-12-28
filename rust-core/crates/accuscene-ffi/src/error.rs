//! Error handling for FFI boundary
//!
//! This module provides error conversion from Rust errors to JavaScript errors
//! across the FFI boundary.

use accuscene_core::error::AccuSceneError;
use napi::{Error as NapiError, Status};

/// Convert AccuSceneError to NAPI Error for JavaScript
pub fn to_napi_error(error: AccuSceneError) -> NapiError {
    let status = match &error {
        AccuSceneError::ValidationError { .. } => Status::InvalidArg,
        AccuSceneError::NotFound { .. } => Status::GenericFailure,
        AccuSceneError::PermissionDenied(_) => Status::GenericFailure,
        AccuSceneError::SerializationError(_) => Status::InvalidArg,
        AccuSceneError::ConfigError(_) => Status::InvalidArg,
        AccuSceneError::PhysicsError(_) => Status::GenericFailure,
        AccuSceneError::MathError(_) => Status::InvalidArg,
        AccuSceneError::IntegrityError(_) => Status::GenericFailure,
        AccuSceneError::ConcurrencyError(_) => Status::GenericFailure,
        AccuSceneError::IoError(_) => Status::GenericFailure,
        AccuSceneError::InternalError(_) => Status::GenericFailure,
        AccuSceneError::InvalidState(_) => Status::InvalidArg,
    };

    NapiError::new(status, error.to_string())
}

/// Result type for FFI operations
pub type FfiResult<T> = Result<T, NapiError>;

/// Convert core Result to FFI Result
pub fn to_ffi_result<T>(result: accuscene_core::error::Result<T>) -> FfiResult<T> {
    result.map_err(to_napi_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let error = AccuSceneError::validation("Test error");
        let napi_error = to_napi_error(error);
        assert_eq!(napi_error.status, Status::InvalidArg);
    }
}
