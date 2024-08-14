use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::RwLock;
use crate::tokenizer::newmm::NewmmTokenizer;
use crate::tokenizer::tokenizer_trait::Tokenizer;

use pyo3::{exceptions, wrap_pyfunction};
use regex::Regex;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;

pub mod tokenizer;
pub mod bytes_str;

static DICT_COLLECTION: Lazy<RwLock<HashMap<String, Box<NewmmTokenizer>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

static NORMALIZE_RULE1: [&str; 23] = [
    "ะ", "ั", "็", "า", "ิ", "ี", "ึ", "่", "ํ", "ุ", "ู", "ใ", "ไ", "โ", "ื", "่", "้", "๋", "๊", "ึ", "์", "๋", "ำ"
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
    ("(์)([ัิ-ู])", "\\2\\1")
];

lazy_static! {
    static ref WHITESPACE_NUMBER_RE: Regex = Regex::new(r"([0-9]+)").unwrap();
    static ref MULTIPLE_SPACES_RE: Regex = Regex::new(r" {2,}").unwrap();
    static ref MULTIPLE_TABS_RE: Regex = Regex::new(r"\t{2,}").unwrap();
    static ref MULTIPLE_NEWLINES_RE: Regex = Regex::new(r"\n{2,}").unwrap();
}

#[pyfunction]
#[pyo3(text_signature = "(text, whitespace_number=True)")]
pub fn normalize(text: &str, whitespace_number: bool) -> PyResult<String> {
    // Normalize Thai text.
    //
    // This function normalizes Thai text by applying various rules to standardize
    // the text representation.
    //
    // Args:
    //     text (str): Input text to be normalized
    //     whitespace_number (bool, optional): If True, adds spaces around numbers. 
    //                                         Defaults to True.
    //
    // Returns:
    //     str: Normalized text
    let mut text = text.to_string();

    if whitespace_number {
        text = WHITESPACE_NUMBER_RE.replace_all(&text, " $1 ").into_owned();
    }

    text = text.replace('\u{200b}', "");
    text = MULTIPLE_SPACES_RE.replace_all(&text, " ").into_owned();
    text = MULTIPLE_TABS_RE.replace_all(&text, "\t").into_owned();
    text = MULTIPLE_NEWLINES_RE.replace_all(&text, "\n").into_owned();

    for (pattern, replacement) in &NORMALIZE_RULE2 {
        let pattern = pattern.replace("t", "[่้๊๋]");
        let re = Regex::new(&pattern).unwrap();
        text = re.replace_all(&text, *replacement).into_owned();
    }

    for &rule in &NORMALIZE_RULE1 {
        let pattern = format!("{}+", rule.replace("t", "[่้๊๋]"));
        let re = Regex::new(&pattern).unwrap();
        text = re.replace_all(&text, rule).into_owned();
    }

    Ok(text)
}

#[pyfunction]
#[pyo3(text_signature = "(text, dict_name, safe=False, parallel=False)")]
fn newmm(text: &str, dict_name: &str, safe: bool, parallel: bool) -> PyResult<Vec<String>> {
    // Break text into tokens.
    //
    // This method is an implementation of newmm segmentation.
    // Supports multithread mode - set by parallel flag.
    //
    // Args:
    //     text (str): Input text
    //     dict_name (str): Dictionary name, as assigned in load_dict()
    //     safe (bool, optional): Use safe mode to avoid long waiting time in
    //         a text with lots of ambiguous word boundaries. Defaults to False.
    //     parallel (bool, optional): Use multithread mode. Defaults to False.
    //
    // Returns:
    //     List[str]: List of tokens
    if let Some(loaded_dict) = DICT_COLLECTION.read().unwrap().get(dict_name) {
        let result = loaded_dict.segment_to_string(text, safe, parallel);
        Ok(result)
    } else {
        Err(exceptions::PyRuntimeError::new_err(format!(
            "Dictionary name {} does not exist.",
            dict_name
        )))
    }
}

#[pyfunction]
#[pyo3(text_signature = "(file_path, dict_name)")]
fn load_dict(file_path: &str, dict_name: &str) -> PyResult<(String, bool)> {
    // Load dictionary from a file.
    //
    // Load a dictionary file into an in-memory dictionary collection,
    // and assign dict_name to it.
    // This function does not override an existing dict name.
    //
    // Args:
    //     file_path (str): Path to a dictionary file
    //     dict_name (str): A unique dictionary name, used for reference
    //
    // Returns:
    //     Tuple[str, bool]: A tuple containing a human-readable result string and a boolean
    let mut dict_col_lock = DICT_COLLECTION.write().unwrap();
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
        dict_col_lock.insert(dict_name.to_owned(), Box::new(tokenizer));

        Ok((
            format!(
                "Successful: file {} has been successfully loaded to dictionary name {}.",
                file_path, dict_name
            ),
            true,
        ))
    }
}

#[pymodule]
fn thongna(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(newmm, m)?)?;
    m.add_function(wrap_pyfunction!(load_dict, m)?)?;
    Ok(())
}
