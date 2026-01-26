//! Word segmentation algorithms for Thai text.
//!
//! This module provides:
//! - NewMM (New Multi-cut Maximum Matching) algorithm (`newmm`)
//! - Segmenter trait for implementing custom algorithms (`traits`)

pub mod newmm;
pub mod traits;

// Re-export commonly used types
pub use newmm::NewmmSegmenter;
pub use traits::Segmenter;
