//! BM25 sparse embedding for Thai text.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

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
}

fn hash_term(term: &str) -> u32 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
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

/// BM25 sparse embedder.
pub struct BM25<T: Tokenizer = WhitespaceTokenizer> {
    tokenizer: T,
    k1: f64,
    b: f64,
    doc_freqs: HashMap<String, usize>,
    doc_count: usize,
    avg_doc_length: f64,
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
            doc_freqs: HashMap::new(),
            doc_count: 0,
            avg_doc_length: 0.0,
        }
    }

    pub fn k1(mut self, k1: f64) -> Self {
        self.k1 = k1;
        self
    }

    pub fn b(mut self, b: f64) -> Self {
        self.b = b;
        self
    }

    /// Fit on a document to update statistics.
    pub fn fit(&mut self, text: &str) {
        let tokens = self.tokenizer.tokenize(text);
        self.fit_tokens(&tokens);
    }

    /// Fit on tokens.
    pub fn fit_tokens(&mut self, tokens: &[String]) {
        let unique_terms: HashSet<_> = tokens.iter().collect();
        for term in unique_terms {
            *self.doc_freqs.entry(term.clone()).or_insert(0) += 1;
        }

        let total = self.avg_doc_length * self.doc_count as f64 + tokens.len() as f64;
        self.doc_count += 1;
        self.avg_doc_length = total / self.doc_count as f64;
    }

    /// Fit on multiple documents.
    pub fn fit_many(&mut self, texts: &[&str]) {
        for text in texts {
            self.fit(text);
        }
    }

    fn idf(&self, term: &str) -> f64 {
        let n = self.doc_count as f64;
        let df = *self.doc_freqs.get(term).unwrap_or(&0) as f64;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Generate sparse embedding.
    pub fn embed(&self, text: &str) -> SparseEmbedding {
        let tokens = self.tokenizer.tokenize(text);
        self.embed_tokens(&tokens)
    }

    /// Generate sparse embedding from tokens.
    pub fn embed_tokens(&self, tokens: &[String]) -> SparseEmbedding {
        let mut term_freqs: HashMap<&str, usize> = HashMap::new();
        for term in tokens {
            *term_freqs.entry(term.as_str()).or_insert(0) += 1;
        }

        let doc_len = tokens.len() as f64;
        let avg_len = if self.avg_doc_length > 0.0 {
            self.avg_doc_length
        } else {
            doc_len.max(1.0)
        };

        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (term, &tf) in &term_freqs {
            let tf = tf as f64;
            let idf = self.idf(term);
            let numerator = tf * (self.k1 + 1.0);
            let denominator = tf + self.k1 * (1.0 - self.b + self.b * (doc_len / avg_len));
            let weight = idf * (numerator / denominator);

            if weight > 0.0 {
                indices.push(hash_term(term));
                values.push(weight as f32);
            }
        }

        SparseEmbedding::new(indices, values)
    }

    /// Embed multiple texts.
    pub fn embed_many(&self, texts: &[&str]) -> Vec<SparseEmbedding> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed() {
        let mut bm25 = BM25::new();
        bm25.fit("hello world");
        bm25.fit("hello rust");

        let embedding = bm25.embed("hello");
        assert!(!embedding.is_empty());
        assert_eq!(embedding.indices.len(), embedding.values.len());
    }

    #[test]
    fn test_embed_many() {
        let mut bm25 = BM25::new();
        bm25.fit_many(&["hello world", "hello rust"]);

        let embeddings = bm25.embed_many(&["hello", "world"]);
        assert_eq!(embeddings.len(), 2);
    }
}
