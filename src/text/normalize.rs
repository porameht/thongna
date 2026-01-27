//! Thai text normalization utilities.
//!
//! This module provides functions to normalize Thai text by standardizing
//! character representations and handling common text variations.

use once_cell::sync::Lazy;
use regex::Regex;

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
/// use thainlp_rs::text::normalize::normalize;
///
/// let text = "สวัสดี123ครับ";
/// let normalized = normalize(text, true);
/// ```
pub fn normalize(text: &str, whitespace_number: bool) -> String {
    let mut result = if whitespace_number {
        WHITESPACE_NUMBER_RE.replace_all(text, " $1 ").into_owned()
    } else {
        text.to_string()
    };

    result = result.replace('\u{200b}', "");
    result = MULTIPLE_SPACES_RE.replace_all(&result, " ").into_owned();
    result = MULTIPLE_TABS_RE.replace_all(&result, "\t").into_owned();
    result = MULTIPLE_NEWLINES_RE.replace_all(&result, "\n").into_owned();

    for (re, replacement) in NORMALIZE_RULE2_COMPILED.iter() {
        result = re.replace_all(&result, *replacement).into_owned();
    }

    for (re, rule) in NORMALIZE_RULE1_COMPILED.iter() {
        result = re.replace_all(&result, *rule).into_owned();
    }

    result
}
