//! Dictionary loading utilities.
//!
//! This module provides functions to load dictionaries from various sources.

use super::trie::DictionaryTrie;
use crate::text::unicode::FixedWidthString;

use std::io::{BufRead, BufReader};
use std::{error::Error, fs::File, path::PathBuf};

/// Source for dictionary data.
pub enum DictSource {
    /// Load dictionary from a file path
    FilePath(PathBuf),
    /// Load dictionary from a list of words
    WordList(Vec<String>),
}

/// Create a dictionary trie from a given source.
pub fn create_trie(source: DictSource) -> Result<DictionaryTrie, Box<dyn Error>> {
    match source {
        DictSource::FilePath(file_path) => {
            let file = File::open(file_path)?;
            let reader = BufReader::with_capacity(8192, file);
            let dict: Vec<FixedWidthString> = reader
                .lines()
                .map_while(Result::ok)
                .map(|line| FixedWidthString::new(&line))
                .collect();
            Ok(DictionaryTrie::new(&dict))
        }
        DictSource::WordList(word_list) => {
            let fixed_width_list: Vec<FixedWidthString> = word_list
                .into_iter()
                .map(|word| FixedWidthString::new(&word))
                .collect();
            Ok(DictionaryTrie::new(&fixed_width_list))
        }
    }
}

#[test]
fn test_trie() {
    let test_word_list = vec![
        "กากบาท".to_string(),
        "กาแฟ".to_string(),
        "กรรม".to_string(),
        "42".to_string(),
        "aง|.%".to_string(),
    ];
    let trie = create_trie(DictSource::WordList(test_word_list)).unwrap();
    assert!(trie.contains(&FixedWidthString::new("กาแฟ")));
    assert_eq!(trie.len(), 5);
}
