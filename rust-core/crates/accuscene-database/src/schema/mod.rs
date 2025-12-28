//! Database schema definitions using sea-query
//!
//! Provides type-safe table and index definitions for the AccuScene database.

pub mod tables;
pub mod indexes;

pub use tables::*;
pub use indexes::*;
