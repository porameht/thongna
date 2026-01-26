//! # thainlp-rs
//!
//! High-performance Thai NLP library for word segmentation, text normalization, and more.
//!
//! ## Example
//!
//! ```ignore
//! use thainlp_rs::{load_dict, segment, normalize};
//!
//! // Normalize Thai text
//! let normalized = normalize("สวัสดี123ครับ", true);
//!
//! // Load a dictionary and segment text
//! load_dict("path/to/dict.txt", "my_dict").unwrap();
//! let tokens = segment("สวัสดีครับ", "my_dict", false, false).unwrap();
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;

pub mod dictionary;
pub mod error;
pub mod segmentation;
pub mod text;

use segmentation::{NewmmSegmenter, Segmenter};

pub use error::{Error, Result};
pub use text::normalize::normalize;

static DICT_COLLECTION: Lazy<RwLock<HashMap<String, NewmmSegmenter>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Break text into tokens using NewMM algorithm.
///
/// This method is an implementation of newmm segmentation.
/// Supports multithread mode - set by parallel flag.
///
/// # Arguments
///
/// * `text` - Input text
/// * `dict_name` - Dictionary name, as assigned in `load_dict()`
/// * `safe` - Use safe mode to avoid long waiting time in a text with lots of ambiguous word boundaries
/// * `parallel` - Use multithread mode
///
/// # Returns
///
/// Result containing a list of tokens or an error
///
/// # Example
///
/// ```ignore
/// use thongna::{load_dict, segment};
///
/// load_dict("dict.txt", "default").unwrap();
/// let tokens = segment("สวัสดีครับ", "default", false, false).unwrap();
/// ```
pub fn segment(
    text: &str,
    dict_name: &str,
    safe: bool,
    parallel: bool,
) -> Result<Vec<String>> {
    let dict_collection = DICT_COLLECTION
        .read()
        .map_err(|e| Error::LockError(e.to_string()))?;

    if let Some(loaded_dict) = dict_collection.get(dict_name) {
        let result = loaded_dict.segment_to_string(text, safe, parallel);
        Ok(result)
    } else {
        Err(Error::DictionaryNotFound(dict_name.to_string()))
    }
}

/// Load dictionary from a file.
///
/// Load a dictionary file into an in-memory dictionary collection,
/// and assign dict_name to it.
/// This function does not override an existing dict name.
///
/// # Arguments
///
/// * `file_path` - Path to a dictionary file
/// * `dict_name` - A unique dictionary name, used for reference
///
/// # Returns
///
/// Result containing a tuple of (message, success) or an error
///
/// # Example
///
/// ```ignore
/// use thongna::load_dict;
///
/// let (msg, success) = load_dict("dict.txt", "my_dict").unwrap();
/// ```
pub fn load_dict(file_path: &str, dict_name: &str) -> Result<(String, bool)> {
    let mut dict_col_lock = DICT_COLLECTION
        .write()
        .map_err(|e| Error::LockError(e.to_string()))?;

    if dict_col_lock.get(dict_name).is_some() {
        Ok((
            format!(
                "Failed: dictionary name {} already exists, please use another name.",
                dict_name
            ),
            false,
        ))
    } else {
        let segmenter = NewmmSegmenter::new(file_path);
        dict_col_lock.insert(dict_name.to_owned(), segmenter);

        Ok((
            format!(
                "Successful: file {} has been successfully loaded to dictionary name {}.",
                file_path, dict_name
            ),
            true,
        ))
    }
}

// Legacy API compatibility
#[deprecated(since = "0.3.0", note = "Use `segment` instead")]
pub fn newmm(
    text: &str,
    dict_name: &str,
    safe: bool,
    parallel: bool,
) -> Result<Vec<String>> {
    segment(text, dict_name, safe, parallel)
}
