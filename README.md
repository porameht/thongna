# Thongna 🌾

**Thongna** (ท้องนา) is a high-performance text processing library for the Thai language, built with Rust and exposed as a Python package. Inspired by [PyThaiNLP](https://github.com/PyThaiNLP/pythainlp), Thongna aims to provide efficient Thai language processing tools with the speed and reliability of Rust.

## Features
- **Efficient Thai word segmentation**: Break Thai text into meaningful tokens using the NewMM algorithm.
- **Fast and reliable**: Built with Rust, Thongna offers the performance you need for large-scale text processing.
- **Python integration**: Easily use Thongna in your Python projects with its simple and intuitive API.
- **Custom dictionary support**: Load and use custom dictionaries for specialized segmentation tasks.
- **Text normalization**: Standardize Thai text by handling common inconsistencies and variations.
- **Parallel processing**: Utilize multi-core processors for faster processing of large texts.
- **Safe mode**: Prevent infinite loops in tokenization for extra reliability.

## Project Details
- **Version**: 0.2.2 (as of the latest release)
- **License**: Apache-2.0
- **Supported Python versions**: 3.8+
- **Rust edition**: 2021
- **Key dependencies**:
  - PyO3 for Rust-Python interoperability
  - Rayon for parallel processing
  - Regex for text manipulation
- **CI/CD**: Utilizes GitHub Actions for automated testing and building on multiple platforms (Linux, macOS, Windows)
- **Package distribution**: Available on PyPI, with pre-built wheels for various platforms and architectures

## Installation

To install Thongna, ensure you have Python 3.8+ installed, then use `pip`:


## Why Thongna? 🌾

The name "Thongna" (ท้องนา) means "rice field" in Thai, symbolizing growth, nourishment, and the foundational aspects of life. Just like a rice field sustains life, Thongna provides the essential tools for working with Thai text, ensuring that your applications can grow and thrive.

## Contributing
We welcome contributions from the community! If you’d like to contribute to Thongna, please follow these steps:

- Fork the repository.
- Create a new branch for your feature or bugfix.
- Submit a pull request with a clear explanation of your changes.

## License
Thongna is licensed under the Apache License. See the LICENSE file for more details.

## Contact
For any questions, suggestions, or issues, feel free to open an issue or contact the maintainers directly.