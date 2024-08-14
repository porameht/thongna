use crate::bytes_str::custom_string::CustomString;

use super::trie_char::TrieChar as Trie;
use std::io::{BufRead, BufReader};
use std::{error::Error, fs::File, path::PathBuf};

pub enum DictSource {
    FilePath(PathBuf),
    WordList(Vec<String>),
}

pub fn create_dict_trie(source: DictSource) -> Result<Trie, Box<dyn Error>> {
    match source {
        DictSource::FilePath(file_path) => {
            let file = File::open(file_path)?;
            let reader = BufReader::with_capacity(8192, file);
            let dict: Vec<CustomString> = reader
                .lines()
                .filter_map(Result::ok)
                .map(|line| CustomString::new(&line))
                .collect();
            Ok(Trie::new(&dict))
        }
        DictSource::WordList(word_list) => {
            let custom_word_list: Vec<CustomString> = word_list
                .into_iter()
                .map(|word| CustomString::new(&word))
                .collect();
            Ok(Trie::new(&custom_word_list))
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
    let trie = create_dict_trie(DictSource::WordList(test_word_list)).unwrap();
    assert!(trie.contain(&CustomString::new("กาแฟ")));
    assert_eq!(trie.amount_of_words(), 5);
}
