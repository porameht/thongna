# thainlp-rs

Thai NLP library built with Rust.

- Word segmentation (NewMM algorithm)
- Text normalization
- BM25 sparse embedding (fastembed compatible)
- Custom dictionary support

## Installation

```toml
[dependencies]
thainlp-rs = "0.3.0"
```

## Usage

```rust
use thainlp_rs::{load_dict, segment, normalize};
use thainlp_rs::search::thai_bm25;

// Word segmentation
load_dict("data/dictionaries/default.txt", "default").unwrap();
let tokens = segment("สวัสดีครับ", "default", false, false).unwrap();
// ["สวัสดี", "ครับ"]

// Text normalization
let text = normalize("เเปลก", true);
// "แปลก"

// BM25 sparse embedding
let mut bm25 = thai_bm25("data/dictionaries/default.txt");

// Fit documents (auto-selects seq/par based on size)
bm25.fit_batch(&["สวัสดีครับ", "ลาก่อนครับ"]);
bm25.build_cache(); // Pre-compute IDF for 8% speedup

// Embed queries
let embedding = bm25.embed("สวัสดี");
// SparseEmbedding { indices: [...], values: [...] }

// Batch embed (auto-selects seq/par)
let embeddings = bm25.embed_batch(&["สวัสดี", "ลาก่อน"]);

// Explicit control when needed
bm25.fit_batch_seq(&docs);  // Force sequential
bm25.fit_batch_par(&docs);  // Force parallel
```

## Benchmarks

```bash
cargo bench
```

See [benches/BENCHMARK_RESULTS.md](benches/BENCHMARK_RESULTS.md) for detailed results.

## License

Apache-2.0
