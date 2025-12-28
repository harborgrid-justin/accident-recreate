//! Core traits for AccuScene types
//!
//! This module defines the fundamental traits that all AccuScene types
//! should implement for consistency and interoperability.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for types that can be serialized to/from JSON
pub trait Serializable: Serialize + for<'de> Deserialize<'de> {
    /// Serialize to JSON string
    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Serialize to pretty JSON string
    fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize from JSON string
    fn from_json(json: &str) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Serialize to JSON bytes
    fn to_json_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(Into::into)
    }

    /// Deserialize from JSON bytes
    fn from_json_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_slice(bytes).map_err(Into::into)
    }
}

/// Trait for types that can validate their internal state
pub trait Validatable {
    /// Validate the object's state
    ///
    /// # Errors
    /// Returns `AccuSceneError::ValidationError` if validation fails
    fn validate(&self) -> Result<()>;

    /// Check if the object is valid without returning an error
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Get validation warnings (non-critical issues)
    fn validation_warnings(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Trait for types that have a unique identifier
pub trait Identifiable {
    /// Type of the identifier
    type Id: Clone + Debug + PartialEq + Eq;

    /// Get the unique identifier
    fn id(&self) -> &Self::Id;

    /// Set the unique identifier
    fn set_id(&mut self, id: Self::Id);

    /// Create a new instance with a generated ID
    fn with_new_id(self) -> Self
    where
        Self: Sized;
}

/// Trait for types that track creation and modification timestamps
pub trait Timestamped {
    /// Get the creation timestamp
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the last modification timestamp
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;

    /// Update the modification timestamp to now
    fn touch(&mut self);
}

/// Trait for types that can be cloned in a thread-safe manner
pub trait ThreadSafeClone: Clone + Send + Sync {}

/// Automatically implement ThreadSafeClone for types that meet the requirements
impl<T> ThreadSafeClone for T where T: Clone + Send + Sync {}

/// Trait for types that can calculate their memory footprint
pub trait MemoryFootprint {
    /// Calculate approximate memory usage in bytes
    fn memory_footprint(&self) -> usize;

    /// Get a human-readable memory size
    fn memory_size_string(&self) -> String {
        let bytes = self.memory_footprint();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Trait for types that support versioning
pub trait Versioned {
    /// Get the version number
    fn version(&self) -> u32;

    /// Increment the version
    fn increment_version(&mut self);

    /// Check if this version is compatible with another
    fn is_compatible_with(&self, other_version: u32) -> bool {
        self.version() == other_version
    }
}

/// Trait for types that can be merged with another instance
pub trait Mergeable {
    /// Merge another instance into this one
    ///
    /// # Errors
    /// Returns error if merge is not possible
    fn merge(&mut self, other: &Self) -> Result<()>;

    /// Check if two instances can be merged
    fn can_merge(&self, other: &Self) -> bool;
}

/// Trait for types that can provide metadata
pub trait WithMetadata {
    /// Type of metadata
    type Metadata: Clone + Debug;

    /// Get metadata
    fn metadata(&self) -> &Self::Metadata;

    /// Set metadata
    fn set_metadata(&mut self, metadata: Self::Metadata);

    /// Update metadata using a function
    fn update_metadata<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::Metadata);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestStruct {
        value: i32,
    }

    impl Serializable for TestStruct {}

    impl Validatable for TestStruct {
        fn validate(&self) -> Result<()> {
            if self.value < 0 {
                Err(crate::error::AccuSceneError::validation(
                    "Value must be non-negative",
                ))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_serializable() {
        let test = TestStruct { value: 42 };
        let json = test.to_json().unwrap();
        let deserialized = TestStruct::from_json(&json).unwrap();
        assert_eq!(test.value, deserialized.value);
    }

    #[test]
    fn test_validatable() {
        let valid = TestStruct { value: 42 };
        assert!(valid.is_valid());

        let invalid = TestStruct { value: -1 };
        assert!(!invalid.is_valid());
    }
}
