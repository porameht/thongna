# Thongna 🌾

**Thongna** (ท้องนา) is a high-performance text processing library for the Thai language, built with Rust and exposed as a Python package. Designed to handle the complexities of Thai text with the speed and efficiency that Rust provides, Thongna is perfect for developers looking to integrate advanced text processing features into their applications.

## Features
- **Efficient Thai text normalization**: Clean and standardize Thai text by removing or replacing special characters, whitespace, and more.
- **Fast and reliable**: Built with Rust, Thongna offers the performance you need for large-scale text processing.
- **Python integration**: Easily use Thongna in your Python projects with its simple and intuitive API.

## Installation

To install Thongna, ensure you have Python and Rust installed, then use `pip`:

```bash
pip install thongna
```

Usage
Here's a quick example of how to use Thongna for basic text processing:

```python
import thongna

# Example text
thai_text = "สวัสดีค่ะ! นี่คือทดสอบการใช้งาน Thongna 🌾"

# Normalize the text
normalized_text = thongna.normalize_text(thai_text)

print("Normalized Text:", normalized_text)
```

## Functions

- normalize_text(text: str) -> str: Normalize Thai text by cleaning up unwanted characters and ensuring consistent formatting.
- replace_characters(text: str, replacements: dict) -> str: Replace specific characters in the text based on a given dictionary of replacements.
- More features to come...

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