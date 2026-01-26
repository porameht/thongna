//! Dictionary management for Thai word segmentation.
//!
//! This module provides:
//! - Trie data structure for efficient word lookups (`trie`)
//! - Dictionary loading utilities (`source`)

pub mod source;
pub mod trie;

// Re-export commonly used types
pub use source::{create_trie, DictSource};
pub use trie::DictionaryTrie;
