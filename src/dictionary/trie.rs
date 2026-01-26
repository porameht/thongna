//! Trie data structure for dictionary storage.
//!
//! This module provides an efficient trie implementation for storing
//! and querying Thai words in the dictionary.

use crate::text::unicode::{
    FixedWidthBytesSlice, FixedWidthBytesVec, FixedWidthString, FixedCharsLengthByteSlice,
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::BorrowMut;

#[derive(Debug)]
struct TrieNode {
    children: HashMap<char, Self>,
    end: bool,
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::default(),
            end: false,
        }
    }

    fn find_child(&self, word: &char) -> Option<&Self> {
        self.children.get(word)
    }

    #[allow(dead_code)]
    fn remove_child(&mut self, word: &char) {
        self.children.remove(word);
    }

    #[allow(dead_code)]
    fn find_mut_child(&mut self, word: &char) -> Option<&mut Self> {
        self.children.get_mut(word)
    }

    #[allow(dead_code)]
    fn set_not_end(&mut self) {
        self.end = false;
    }

    fn add_word(&mut self, input_word: &FixedWidthString) {
        if input_word.is_empty() {
            self.end = true;
            return;
        }
        self.children
            .entry(*input_word.get_chars_content().first().unwrap())
            .or_default()
            .add_word(&input_word.substring(1, input_word.chars_len()));
    }

    fn remove_word(&mut self, input_word: &FixedWidthString) {
        let mut word = input_word;
        let char_count = word.chars_len();
        if char_count >= 1 {
            let character = word.get_chars_content().first().unwrap();
            if let Some(child) = self.find_mut_child(character) {
                let substring_of_word = word.substring(1, word.chars_len());
                if char_count == 1 {
                    child.set_not_end();
                }
                word = &substring_of_word;
                child.remove_word(word);
                if !child.end && child.children.is_empty() {
                    self.remove_child(character);
                }
            };
        }
    }
}

/// Character-based Trie for efficient dictionary lookups.
///
/// This version of Trie stores fixed-width bytes vector as words,
/// but prefix operation and its node uses char instead.
#[derive(Debug)]
pub struct DictionaryTrie {
    words: HashSet<FixedWidthBytesVec>,
    root: TrieNode,
}

impl DictionaryTrie {
    pub fn new(words: &[FixedWidthString]) -> Self {
        let mut instance = Self {
            words: HashSet::default(),
            root: TrieNode::new(),
        };
        for word in words.iter() {
            instance.add(word);
        }
        instance
    }

    #[allow(dead_code)]
    fn remove_word_from_set(&mut self, word: &FixedWidthString) {
        self.words.remove(word.raw_content());
    }

    pub fn add(&mut self, word: &FixedWidthString) {
        let stripped_word = word.trim();
        if !stripped_word.is_empty() {
            self.words.insert(stripped_word.raw_content().into());
            let current_cursor = self.root.borrow_mut();
            current_cursor.add_word(&stripped_word);
        }
    }

    pub fn remove(&mut self, word: &FixedWidthString) {
        let stripped_word = word.trim();
        if !stripped_word.is_empty() && self.words.contains(stripped_word.raw_content()) {
            self.remove_word_from_set(&stripped_word);
            self.root.remove_word(&stripped_word);
        }
    }

    #[allow(dead_code)]
    pub fn contains(&self, word: &FixedWidthString) -> bool {
        self.words.contains(word.raw_content())
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Vec<u8>> {
        self.words.iter()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Returns a vec of substring (as reference) as produced by words stored in dict_trie.
    pub fn find_prefixes<'p>(
        prefix: &'p FixedWidthString,
        dict_trie: &Self,
    ) -> Vec<&'p FixedWidthBytesSlice> {
        let mut result: Vec<&[u8]> = vec![];
        let prefix_cpy = prefix;
        let mut current_index = 0;
        let mut current_node_wrap = Some(&dict_trie.root);
        while current_index < prefix_cpy.chars_len() {
            let character = prefix_cpy.get_char_at(current_index);
            if let Some(current_node) = current_node_wrap {
                if let Some(child) = current_node.find_child(&character) {
                    if child.end {
                        let substring_of_prefix = prefix_cpy
                            .raw_content()
                            .slice_by_char_indice(0, current_index + 1);
                        result.push(substring_of_prefix);
                    }
                    current_node_wrap = Some(child);
                } else {
                    break;
                }
            }
            current_index += 1;
        }
        result
    }
}

#[test]
fn test_add_and_remove_word() {
    let mut trie = DictionaryTrie::new(&[FixedWidthString::new("ศาล")]);
    assert_eq!(trie.len(), 1);
    trie.add(&FixedWidthString::new("ศาล"));
    assert_eq!(trie.len(), 1);
    trie.add(&FixedWidthString::new("  ศาล "));
    assert_eq!(trie.len(), 1);
    trie.add(&FixedWidthString::new("ศาลา"));
    assert_eq!(trie.len(), 2);
    trie.remove(&FixedWidthString::new("ศาลา"));
    assert_eq!(trie.len(), 1);
    trie.remove(&FixedWidthString::new("ลา"));
    assert_eq!(trie.len(), 1);
    trie.remove(&FixedWidthString::new("ศาล"));
    assert_eq!(trie.len(), 0);
    trie.remove(&FixedWidthString::new(""));
    assert_eq!(trie.len(), 0);
}
