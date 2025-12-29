//! Data encoding schemes for efficient serialization.
//!
//! This module provides various encoding schemes:
//! - Variable-length integer encoding (varint)
//! - ZigZag encoding for signed integers
//! - Prefix compression for sorted strings

pub mod prefix;
pub mod varint;
pub mod zigzag;

pub use prefix::PrefixCompressor;
pub use varint::{decode_varint, encode_varint};
pub use zigzag::{zigzag_decode, zigzag_encode};
