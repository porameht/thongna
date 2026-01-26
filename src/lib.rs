//! Thongna - High-performance Thai text processing library
//!
//! A blazing-fast Thai text processing library for word segmentation and normalization.
//!
//! # Example
//!
//! ```ignore
//! use thongna::{load_dict, newmm, normalize};
//!
//! // Normalize Thai text
//! let normalized = normalize("สวัสดี123ครับ", true);
//!
//! // Load a dictionary and segment text
//! load_dict("path/to/dict.txt", "my_dict").unwrap();
//! let tokens = newmm("สวัสดีครับ", "my_dict", false, false).unwrap();
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::tokenizer::newmm::NewmmTokenizer;
use crate::tokenizer::traits::Tokenizer;

pub mod encoding;
pub mod tokenizer;

static DICT_COLLECTION: Lazy<RwLock<HashMap<String, NewmmTokenizer>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static NORMALIZE_RULE1: [&str; 23] = [
    "ะ", "ั", "็", "า", "ิ", "ี", "ึ", "่", "ํ", "ุ", "ู", "ใ", "ไ", "โ", "ื", "่", "้", "๋", "๊", "ึ", "์", "๋",
    "ำ",
];

static NORMALIZE_RULE2: [(&str, &str); 9] = [
    ("เเ", "แ"),
    ("ู้", "ู้"),
    ("ํา", "ำ"),
    ("ํ(t)า", "\\1ำ"),
    ("ํา(t)", "\\1ำ"),
    ("([่-๋])([ัิ-ื])", "\\2\\1"),
    ("([่-๋])([ูุ])", "\\2\\1"),
    ("ำ([่-๋])", "\\1ำ"),
    ("(์)([ัิ-ู])", "\\2\\1"),
];

static WHITESPACE_NUMBER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9]+)").unwrap());
static MULTIPLE_SPACES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r" {2,}").unwrap());
static MULTIPLE_TABS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\t{2,}").unwrap());
static MULTIPLE_NEWLINES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{2,}").unwrap());

static NORMALIZE_RULE2_COMPILED: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    let tone_marks = "[่้๊๋]";
    NORMALIZE_RULE2
        .iter()
        .map(|(pattern, replacement)| {
            let compiled_pattern = pattern.replace("t", tone_marks);
            (
                Regex::new(&compiled_pattern).expect("Invalid NORMALIZE_RULE2 pattern"),
                *replacement,
            )
        })
        .collect()
});

static NORMALIZE_RULE1_COMPILED: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    let tone_marks = "[่้๊๋]";
    NORMALIZE_RULE1
        .iter()
        .map(|rule| {
            let pattern = format!("{}+", rule.replace("t", tone_marks));
            (
                Regex::new(&pattern).expect("Invalid NORMALIZE_RULE1 pattern"),
                *rule,
            )
        })
        .collect()
});

/// Error type for thongna operations
#[derive(Debug)]
pub enum ThongnaError {
    /// Dictionary not found
    DictionaryNotFound(String),
    /// Lock acquisition failed
    LockError(String),
}

impl std::fmt::Display for ThongnaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThongnaError::DictionaryNotFound(name) => {
                write!(f, "Dictionary name {} does not exist.", name)
            }
            ThongnaError::LockError(msg) => {
                write!(f, "Failed to acquire dictionary lock: {}", msg)
            }
        }
    }
}

impl std::error::Error for ThongnaError {}

/// Normalize Thai text.
///
/// This function normalizes Thai text by applying various rules to standardize
/// the text representation.
///
/// # Arguments
///
/// * `text` - Input text to be normalized
/// * `whitespace_number` - If true, adds spaces around numbers
///
/// # Returns
///
/// Normalized text as a String
///
/// # Example
///
/// ```
/// use thongna::normalize;
///
/// let text = "สวัสดี123ครับ";
/// let normalized = normalize(text, true);
/// ```
pub fn normalize(text: &str, whitespace_number: bool) -> String {
    let mut text = text.to_string();

    if whitespace_number {
        text = WHITESPACE_NUMBER_RE.replace_all(&text, " $1 ").into_owned();
    }

    text = text.replace('\u{200b}', "");
    text = MULTIPLE_SPACES_RE.replace_all(&text, " ").into_owned();
    text = MULTIPLE_TABS_RE.replace_all(&text, "\t").into_owned();
    text = MULTIPLE_NEWLINES_RE.replace_all(&text, "\n").into_owned();

    for (re, replacement) in NORMALIZE_RULE2_COMPILED.iter() {
        text = re.replace_all(&text, *replacement).into_owned();
    }

    for (re, rule) in NORMALIZE_RULE1_COMPILED.iter() {
        text = re.replace_all(&text, *rule).into_owned();
    }

    text
}

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
/// use thongna::{load_dict, newmm};
///
/// load_dict("dict.txt", "default").unwrap();
/// let tokens = newmm("สวัสดีครับ", "default", false, false).unwrap();
/// ```
pub fn newmm(
    text: &str,
    dict_name: &str,
    safe: bool,
    parallel: bool,
) -> Result<Vec<String>, ThongnaError> {
    let dict_collection = DICT_COLLECTION
        .read()
        .map_err(|e| ThongnaError::LockError(e.to_string()))?;

    if let Some(loaded_dict) = dict_collection.get(dict_name) {
        let result = loaded_dict.segment_to_string(text, safe, parallel);
        Ok(result)
    } else {
        Err(ThongnaError::DictionaryNotFound(dict_name.to_string()))
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
pub fn load_dict(file_path: &str, dict_name: &str) -> Result<(String, bool), ThongnaError> {
    let mut dict_col_lock = DICT_COLLECTION
        .write()
        .map_err(|e| ThongnaError::LockError(e.to_string()))?;

    if dict_col_lock.get(dict_name).is_some() {
        Ok((
            format!(
                "Failed: dictionary name {} already exists, please use another name.",
                dict_name
            ),
            false,
        ))
    } else {
        let tokenizer = NewmmTokenizer::new(file_path);
        dict_col_lock.insert(dict_name.to_owned(), tokenizer);

        Ok((
            format!(
                "Successful: file {} has been successfully loaded to dictionary name {}.",
                file_path, dict_name
            ),
            true,
        ))
    }
}
