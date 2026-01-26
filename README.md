# Thongna

A high-performance Thai text processing library built with Rust.

## Features

- Thai word segmentation (NewMM algorithm)
- Text normalization
- Custom dictionary support
- Parallel processing

## Installation

```toml
[dependencies]
thongna = "0.2.4"
```

## Usage

```rust
use thongna::{word_tokenize, normalize};

fn main() {
    // Word segmentation
    let tokens = word_tokenize("สวัสดีครับ");
    println!("{:?}", tokens);

    // Text normalization
    let text = normalize("เเปลก");
    println!("{}", text);
}
```

## License

Apache-2.0
