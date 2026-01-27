//! Thai-specific BM25 search using NewMM tokenizer.

use super::bm25::{Tokenizer, BM25};
use crate::segmentation::{NewmmSegmenter, Segmenter};

/// Thai tokenizer using NewMM algorithm.
pub struct ThaiTokenizer {
    segmenter: NewmmSegmenter,
}

impl ThaiTokenizer {
    /// Create a new Thai tokenizer with dictionary file.
    pub fn new(dict_path: &str) -> Self {
        ThaiTokenizer {
            segmenter: NewmmSegmenter::new(dict_path),
        }
    }

    /// Create a new Thai tokenizer with word list.
    pub fn from_word_list(words: Vec<String>) -> Self {
        ThaiTokenizer {
            segmenter: NewmmSegmenter::from_word_list(words),
        }
    }
}

impl Tokenizer for ThaiTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        self.segmenter.segment_to_string(text, false, false)
    }
}

/// Create a Thai BM25 search index.
pub fn thai_bm25(dict_path: &str) -> BM25<ThaiTokenizer> {
    BM25::with_tokenizer(ThaiTokenizer::new(dict_path))
}
