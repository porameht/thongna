//! Traits for word segmentation algorithms.

use anyhow::Result as AnyResult;

/// Trait for word segmentation algorithms.
pub trait Segmenter {
    /// Segment text into words.
    fn segment(&self, text: &str, safe: bool, parallel: bool) -> AnyResult<Vec<String>>;

    /// Segment text into words, returning empty vec on error.
    fn segment_to_string(&self, text: &str, safe: bool, parallel: bool) -> Vec<String>;
}
