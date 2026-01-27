//! Optimized BM25 sparse embedding for Thai text.
//!
//! Optimizations:
//! - BM25+ variant (fixes long document penalty)
//! - Pre-computed IDF cache
//! - SIMD-friendly memory layout
//! - Adaptive parallel/sequential processing based on data size
//!
//! # Parallel vs Sequential
//!
//! Based on benchmarks, the optimal strategy depends on data size:
//! - **fit**: Parallel is faster for 500+ documents
//! - **embed**: Sequential is faster for most cases (each embed is ~100ns)
//!
//! Use `fit_batch`/`embed_batch` for auto-selection, or explicit
//! `*_seq`/`*_par` methods for manual control.

use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

/// Threshold for parallel fit (based on benchmarks: parallel wins at ~300+ docs)
const FIT_PARALLEL_THRESHOLD: usize = 300;

/// Threshold for parallel embed (based on benchmarks: parallel wins at ~3000+ queries)
const EMBED_PARALLEL_THRESHOLD: usize = 3_000;

/// Sparse embedding (fastembed compatible).
#[derive(Debug, Clone, PartialEq)]
pub struct SparseEmbedding {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

impl SparseEmbedding {
    pub fn new(indices: Vec<u32>, values: Vec<f32>) -> Self {
        Self { indices, values }
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Dot product with another sparse embedding.
    pub fn dot(&self, other: &SparseEmbedding) -> f32 {
        let other_map: FxHashMap<u32, f32> = other
            .indices
            .iter()
            .zip(other.values.iter())
            .map(|(&i, &v)| (i, v))
            .collect();

        self.indices
            .iter()
            .zip(self.values.iter())
            .filter_map(|(&idx, &val)| other_map.get(&idx).map(|&other_val| val * other_val))
            .sum()
    }
}

#[inline(always)]
fn hash_term(term: &str) -> u32 {
    let mut hasher = rustc_hash::FxHasher::default();
    term.hash(&mut hasher);
    hasher.finish() as u32
}

/// Tokenizer trait.
pub trait Tokenizer: Send + Sync {
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// Whitespace tokenizer.
#[derive(Clone)]
pub struct WhitespaceTokenizer;

impl Tokenizer for WhitespaceTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect()
    }
}

/// BM25+ optimized sparse embedder.
///
/// Improvements over standard BM25:
/// - BM25+ variant: adds delta (δ=1) to fix long document penalty
/// - Pre-computed IDF cache for fast lookups
/// - FxHashMap for faster hashing
/// - Parallel batch embedding with Rayon
pub struct BM25<T: Tokenizer = WhitespaceTokenizer> {
    tokenizer: T,
    k1: f32,
    b: f32,
    delta: f32, // BM25+ parameter
    doc_count: usize,
    total_doc_length: usize,
    doc_freqs: FxHashMap<String, usize>,
    idf_cache: FxHashMap<String, f32>,
}

impl BM25<WhitespaceTokenizer> {
    pub fn new() -> Self {
        Self::with_tokenizer(WhitespaceTokenizer)
    }
}

impl Default for BM25<WhitespaceTokenizer> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Tokenizer> BM25<T> {
    pub fn with_tokenizer(tokenizer: T) -> Self {
        BM25 {
            tokenizer,
            k1: 1.2,
            b: 0.75,
            delta: 1.0, // BM25+ default
            doc_count: 0,
            total_doc_length: 0,
            doc_freqs: FxHashMap::default(),
            idf_cache: FxHashMap::default(),
        }
    }

    /// Set k1 parameter (term frequency saturation). Default: 1.2
    pub fn k1(mut self, k1: f32) -> Self {
        self.k1 = k1;
        self
    }

    /// Set b parameter (document length normalization). Default: 0.75
    pub fn b(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    /// Set delta parameter (BM25+ lower bound). Default: 1.0
    /// Set to 0.0 for standard BM25 behavior.
    pub fn delta(mut self, delta: f32) -> Self {
        self.delta = delta;
        self
    }

    #[inline(always)]
    fn avg_doc_length(&self) -> f32 {
        if self.doc_count == 0 {
            1.0
        } else {
            self.total_doc_length as f32 / self.doc_count as f32
        }
    }

    /// Fit on a document to update statistics.
    pub fn fit(&mut self, text: &str) {
        let tokens = self.tokenizer.tokenize(text);
        self.fit_tokens(&tokens);
    }

    /// Fit on tokens.
    pub fn fit_tokens(&mut self, tokens: &[String]) {
        // Update document frequencies (unique terms only)
        let mut seen = rustc_hash::FxHashSet::default();
        for term in tokens {
            if seen.insert(term) {
                *self.doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }
        }

        self.total_doc_length += tokens.len();
        self.doc_count += 1;

        // Invalidate IDF cache
        self.idf_cache.clear();
    }

    /// Fit on multiple documents (auto-selects parallel or sequential).
    ///
    /// Uses parallel processing for 500+ documents, sequential otherwise.
    /// For explicit control, use `fit_batch_seq` or `fit_batch_par`.
    pub fn fit_batch(&mut self, texts: &[&str]) {
        if texts.len() >= FIT_PARALLEL_THRESHOLD {
            self.fit_batch_par(texts);
        } else {
            self.fit_batch_seq(texts);
        }
    }

    /// Fit on multiple documents (sequential).
    ///
    /// Best for small batches (<500 documents).
    pub fn fit_batch_seq(&mut self, texts: &[&str]) {
        for text in texts {
            self.fit(text);
        }
    }

    /// Fit on multiple documents (parallel tokenization).
    ///
    /// Best for large batches (500+ documents).
    /// Tokenizes in parallel, then updates stats sequentially.
    pub fn fit_batch_par(&mut self, texts: &[&str]) {
        // Tokenize in parallel
        let token_lists: Vec<Vec<String>> = texts
            .par_iter()
            .map(|text| self.tokenizer.tokenize(text))
            .collect();

        // Update stats sequentially (required for consistency)
        for tokens in token_lists {
            self.fit_tokens(&tokens);
        }
    }

    /// Alias for `fit_batch_par` (backward compatibility).
    #[deprecated(since = "0.3.1", note = "Use `fit_batch` or `fit_batch_par` instead")]
    pub fn fit_many(&mut self, texts: &[&str]) {
        self.fit_batch_par(texts);
    }

    /// Pre-compute and cache IDF for all known terms.
    pub fn build_cache(&mut self) {
        let n = self.doc_count as f32;
        self.idf_cache = self
            .doc_freqs
            .iter()
            .map(|(term, &df)| {
                let df = df as f32;
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                (term.clone(), idf)
            })
            .collect();
    }

    #[inline(always)]
    fn idf(&self, term: &str) -> f32 {
        // Check cache first
        if let Some(&cached) = self.idf_cache.get(term) {
            return cached;
        }

        // Compute on-the-fly
        let n = self.doc_count as f32;
        let df = *self.doc_freqs.get(term).unwrap_or(&0) as f32;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Generate sparse embedding using BM25+.
    pub fn embed(&self, text: &str) -> SparseEmbedding {
        let tokens = self.tokenizer.tokenize(text);
        self.embed_tokens(&tokens)
    }

    /// Generate sparse embedding from tokens using BM25+.
    ///
    /// BM25+ formula:
    /// score = IDF * ((tf * (k1 + 1)) / (tf + k1 * (1 - b + b * (dl / avgdl))) + delta)
    pub fn embed_tokens(&self, tokens: &[String]) -> SparseEmbedding {
        if tokens.is_empty() {
            return SparseEmbedding::new(vec![], vec![]);
        }

        // Count term frequencies
        let mut term_freqs: FxHashMap<&str, u32> = FxHashMap::default();
        for term in tokens {
            *term_freqs.entry(term.as_str()).or_insert(0) += 1;
        }

        let doc_len = tokens.len() as f32;
        let len_norm = 1.0 - self.b + self.b * (doc_len / self.avg_doc_length());

        let mut indices = Vec::with_capacity(term_freqs.len());
        let mut values = Vec::with_capacity(term_freqs.len());

        for (term, &tf) in &term_freqs {
            let tf = tf as f32;
            let idf = self.idf(term);

            // BM25+ formula with delta
            let tf_component = (tf * (self.k1 + 1.0)) / (tf + self.k1 * len_norm);
            let weight = idf * (tf_component + self.delta);

            if weight > 0.0 {
                indices.push(hash_term(term));
                values.push(weight);
            }
        }

        SparseEmbedding::new(indices, values)
    }

    /// Embed multiple texts (auto-selects parallel or sequential).
    ///
    /// Uses sequential for most cases (each embed is ~100ns).
    /// Parallel only for 10,000+ texts.
    /// For explicit control, use `embed_batch_seq` or `embed_batch_par`.
    pub fn embed_batch(&self, texts: &[&str]) -> Vec<SparseEmbedding> {
        if texts.len() >= EMBED_PARALLEL_THRESHOLD {
            self.embed_batch_par(texts)
        } else {
            self.embed_batch_seq(texts)
        }
    }

    /// Embed multiple texts (sequential).
    ///
    /// Best for most use cases due to low per-embed cost (~100ns).
    pub fn embed_batch_seq(&self, texts: &[&str]) -> Vec<SparseEmbedding> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Embed multiple texts (parallel).
    ///
    /// Best for very large batches (10,000+ texts) or heavy tokenizers.
    pub fn embed_batch_par(&self, texts: &[&str]) -> Vec<SparseEmbedding> {
        texts.par_iter().map(|t| self.embed(t)).collect()
    }

    /// Alias for `embed_batch_par` (backward compatibility).
    #[deprecated(since = "0.3.1", note = "Use `embed_batch` or `embed_batch_par` instead")]
    pub fn embed_many(&self, texts: &[&str]) -> Vec<SparseEmbedding> {
        self.embed_batch_par(texts)
    }

    /// Embed multiple pre-tokenized texts (auto-selects parallel or sequential).
    pub fn embed_tokens_batch(&self, token_lists: &[Vec<String>]) -> Vec<SparseEmbedding> {
        if token_lists.len() >= EMBED_PARALLEL_THRESHOLD {
            self.embed_tokens_batch_par(token_lists)
        } else {
            self.embed_tokens_batch_seq(token_lists)
        }
    }

    /// Embed multiple pre-tokenized texts (sequential).
    pub fn embed_tokens_batch_seq(&self, token_lists: &[Vec<String>]) -> Vec<SparseEmbedding> {
        token_lists
            .iter()
            .map(|tokens| self.embed_tokens(tokens))
            .collect()
    }

    /// Embed multiple pre-tokenized texts (parallel).
    pub fn embed_tokens_batch_par(&self, token_lists: &[Vec<String>]) -> Vec<SparseEmbedding> {
        token_lists
            .par_iter()
            .map(|tokens| self.embed_tokens(tokens))
            .collect()
    }

    /// Alias for `embed_tokens_batch_par` (backward compatibility).
    #[deprecated(since = "0.3.1", note = "Use `embed_tokens_batch` instead")]
    pub fn embed_many_tokens(&self, token_lists: &[Vec<String>]) -> Vec<SparseEmbedding> {
        self.embed_tokens_batch_par(token_lists)
    }

    /// Get corpus statistics.
    pub fn stats(&self) -> BM25Stats {
        BM25Stats {
            doc_count: self.doc_count,
            vocab_size: self.doc_freqs.len(),
            avg_doc_length: self.avg_doc_length(),
            cache_size: self.idf_cache.len(),
        }
    }
}

/// BM25 corpus statistics.
#[derive(Debug, Clone)]
pub struct BM25Stats {
    pub doc_count: usize,
    pub vocab_size: usize,
    pub avg_doc_length: f32,
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed() {
        let mut bm25 = BM25::new();
        bm25.fit("hello world");
        bm25.fit("hello rust");
        bm25.build_cache();

        let embedding = bm25.embed("hello");
        assert!(!embedding.is_empty());
        assert_eq!(embedding.indices.len(), embedding.values.len());
    }

    #[test]
    fn test_fit_batch_seq() {
        let mut bm25 = BM25::new();
        bm25.fit_batch_seq(&["hello world", "hello rust", "goodbye world"]);
        bm25.build_cache();

        assert_eq!(bm25.stats().doc_count, 3);
    }

    #[test]
    fn test_fit_batch_par() {
        let mut bm25 = BM25::new();
        bm25.fit_batch_par(&["hello world", "hello rust", "goodbye world"]);
        bm25.build_cache();

        assert_eq!(bm25.stats().doc_count, 3);
    }

    #[test]
    fn test_fit_batch_auto() {
        let mut bm25 = BM25::new();
        // Small batch - uses sequential
        bm25.fit_batch(&["hello world", "hello rust"]);
        assert_eq!(bm25.stats().doc_count, 2);
    }

    #[test]
    fn test_embed_batch_seq() {
        let mut bm25 = BM25::new();
        bm25.fit_batch(&["hello world", "hello rust", "goodbye world"]);
        bm25.build_cache();

        let embeddings = bm25.embed_batch_seq(&["hello", "world", "rust"]);
        assert_eq!(embeddings.len(), 3);
    }

    #[test]
    fn test_embed_batch_par() {
        let mut bm25 = BM25::new();
        bm25.fit_batch(&["hello world", "hello rust", "goodbye world"]);
        bm25.build_cache();

        let embeddings = bm25.embed_batch_par(&["hello", "world", "rust"]);
        assert_eq!(embeddings.len(), 3);
    }

    #[test]
    fn test_embed_batch_auto() {
        let mut bm25 = BM25::new();
        bm25.fit_batch(&["hello world", "hello rust", "goodbye world"]);
        bm25.build_cache();

        // Small batch - uses sequential
        let embeddings = bm25.embed_batch(&["hello", "world", "rust"]);
        assert_eq!(embeddings.len(), 3);
    }

    #[test]
    fn test_seq_par_consistency() {
        let docs = ["hello world", "hello rust", "goodbye world"];
        let queries = ["hello", "world"];

        // Sequential
        let mut bm25_seq = BM25::new();
        bm25_seq.fit_batch_seq(&docs);
        bm25_seq.build_cache();
        let emb_seq = bm25_seq.embed_batch_seq(&queries);

        // Parallel
        let mut bm25_par = BM25::new();
        bm25_par.fit_batch_par(&docs);
        bm25_par.build_cache();
        let emb_par = bm25_par.embed_batch_par(&queries);

        // Results should be identical
        assert_eq!(bm25_seq.stats().doc_count, bm25_par.stats().doc_count);
        assert_eq!(emb_seq.len(), emb_par.len());
        for (seq, par) in emb_seq.iter().zip(emb_par.iter()) {
            assert_eq!(seq.len(), par.len());
        }
    }

    #[test]
    fn test_bm25_plus_delta() {
        let mut bm25_std = BM25::new().delta(0.0); // Standard BM25
        let mut bm25_plus = BM25::new().delta(1.0); // BM25+

        bm25_std.fit("test document with many words");
        bm25_plus.fit("test document with many words");

        let emb_std = bm25_std.embed("test");
        let emb_plus = bm25_plus.embed("test");

        // BM25+ should have higher scores due to delta
        assert!(emb_plus.values[0] > emb_std.values[0]);
    }

    #[test]
    fn test_dot_product() {
        let mut bm25 = BM25::new();
        bm25.fit("hello world");
        bm25.build_cache();

        let emb1 = bm25.embed("hello world");
        let emb2 = bm25.embed("hello");

        let score = emb1.dot(&emb2);
        assert!(score > 0.0);
    }

    #[test]
    fn test_stats() {
        let mut bm25 = BM25::new();
        bm25.fit_batch(&["hello world", "hello rust"]);
        bm25.build_cache();

        let stats = bm25.stats();
        assert_eq!(stats.doc_count, 2);
        assert_eq!(stats.vocab_size, 3); // hello, world, rust
        assert!(stats.cache_size > 0);
    }
}
