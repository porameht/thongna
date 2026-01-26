# thainlp-rs

Thai NLP library built with Rust.

- Word segmentation (NewMM algorithm)
- Text normalization
- Custom dictionary support
- Parallel processing

## Installation

```toml
[dependencies]
thainlp-rs = "0.3.0"
```

## Usage

```rust
use thainlp_rs::{load_dict, segment, normalize};

// Load dictionary
load_dict("data/dictionaries/default.txt", "default").unwrap();

// Word segmentation
let tokens = segment("สวัสดีครับ", "default", false, false).unwrap();
// ["สวัสดี", "ครับ"]

// Text normalization
let text = normalize("เเปลก", true);
// "แปลก"
```

## License

Apache-2.0
