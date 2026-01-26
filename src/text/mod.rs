//! Text processing utilities for Thai language.
//!
//! This module provides:
//! - Fixed-width Unicode string handling (`unicode`)
//! - Regex pattern conversion (`pattern`)
//! - Thai character cluster detection (`cluster`)
//! - Text normalization (`normalize`)

pub mod cluster;
pub mod normalize;
pub mod pattern;
pub mod unicode;

// Re-export commonly used types
pub use normalize::normalize;
pub use unicode::{
    FixedCharsLengthByteSlice, FixedWidthBytesSlice, FixedWidthBytesVec, FixedWidthString,
    BYTES_PER_CHAR,
};
