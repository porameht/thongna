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
bm25.fit("สวัสดีครับ");
bm25.fit("ลาก่อนครับ");

let embedding = bm25.embed("สวัสดี");
// SparseEmbedding { indices: [...], values: [...] }

let embeddings = bm25.embed_many(&["สวัสดี", "ลาก่อน"]);
```

## License

Apache-2.0
