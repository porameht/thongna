//! Search functionality using BM25 algorithm.
//!
//! # Example
//!
//! ```ignore
//! use thainlp_rs::search::{BM25, thai_bm25};
//!
//! // Basic BM25 (whitespace tokenizer)
//! let mut bm25 = BM25::new();
//! bm25.add("hello world");
//!
//! // Thai BM25 (NewMM tokenizer)
//! let mut thai = thai_bm25("data/dictionaries/default.txt");
//! thai.add("สวัสดีครับ");
//! ```

pub mod bm25;
pub mod thai;

pub use bm25::{SparseEmbedding, Tokenizer, WhitespaceTokenizer, BM25};
pub use thai::{thai_bm25, ThaiTokenizer};
