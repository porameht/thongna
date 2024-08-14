use std::{collections::VecDeque, error::Error, fmt::Display, path::PathBuf};

use super::{
    dict_reader::{create_dict_trie, DictSource},
    tcc::tcc_tokenizer,
    tokenizer_trait::Tokenizer,
    trie_char::TrieChar as Trie,
};
use crate::bytes_str::custom_string::{CustomStringBytesSlice, FixedCharsLengthByteSlice};
use crate::bytes_str::custom_regex::regex_pattern_to_custom_pattern;
use crate::bytes_str::custom_string::{rfind_space_char_index, CustomString, BYTES_PER_CHAR};

use anyhow::Result as AnyResult;
use binary_heap_plus::{BinaryHeap, MinComparator};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::bytes::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

const MAX_GRAPH_SIZE: usize = 50;

const TEXT_SCAN_POINT: usize = 120;
const TEXT_SCAN_LEFT: usize = 20;
const TEXT_SCAN_RIGHT: usize = 20;
const TEXT_SCAN_BEGIN: usize = TEXT_SCAN_POINT - TEXT_SCAN_LEFT;
const TEXT_SCAN_END: usize = TEXT_SCAN_POINT + TEXT_SCAN_RIGHT;

type CharacterIndex = usize;

const NON_THAI_READABLE_PATTERN: &[&str; 5] = &[
    r"(?x)^[-a-zA-Z]+",
    r"(?x)^[0-9]+([,\.][0-9]+)*",
    r"(?x)^[๐-๙]+([,\.][๐-๙]+)*",
    r"(?x)^[\ \t]+",
    r"(?x)^\r?\n",
];

lazy_static! {
    static ref NON_THAI_PATTERN: Regex = Regex::new(
        &NON_THAI_READABLE_PATTERN
            .iter()
            .map(|p| regex_pattern_to_custom_pattern(p).unwrap())
            .collect::<Vec<_>>()
            .join("|")
    )
    .unwrap();
}

lazy_static! {
    static ref THAI_TWOCHARS_PATTERN: Regex =
        Regex::new(&regex_pattern_to_custom_pattern(r"^[ก-ฮ]{0,2}$").unwrap()).unwrap();
}

#[derive(Clone, Debug)]
struct BFSSearchError {
    graph: HashMap<CharacterIndex, Vec<CharacterIndex>>,
    start: CharacterIndex,
    goal: CharacterIndex,
}

impl BFSSearchError {
    #[inline(always)]
    pub fn new(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
    ) -> Self {
        Self {
            graph: graph.clone(),
            start,
            goal,
        }
    }
}

impl Display for BFSSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot find goal position {} with start position {} with graph {:?}",
            self.goal, self.start, self.graph
        )
    }
}

impl Error for BFSSearchError {}

#[derive(Debug)]
pub struct NewmmTokenizer {
    dict: Box<Trie>,
}

impl NewmmTokenizer {
    /// Create a new tokenizer using a dictionary from a text file
    pub fn new(dict_path: &str) -> Self {
        NewmmTokenizer {
            dict: Box::from(
                create_dict_trie(DictSource::FilePath(PathBuf::from(dict_path))).unwrap(),
            ),
        }
    }

    /// Create a new tokenizer using a dictionary from a vector of Strings
    pub fn from_word_list(word_list: Vec<String>) -> Self {
        NewmmTokenizer {
            dict: Box::from(create_dict_trie(DictSource::WordList(word_list)).unwrap()),
        }
    }

    /// Add words to the tokenizer's dictionary
    pub fn add_word(&mut self, word_list: &[&str]) {
        word_list.iter().for_each(|word| {
            self.dict.add(&CustomString::new(word));
        });
    }

    /// Remove words from the tokenizer's dictionary
    pub fn remove_word(&mut self, word_list: &[&str]) {
        word_list.iter().for_each(|word| {
            self.dict.remove(&CustomString::new(word));
        });
    }

    #[inline(always)]
    fn bfs_paths_graph(
        graph: &HashMap<CharacterIndex, Vec<CharacterIndex>>,
        start: CharacterIndex,
        goal: CharacterIndex,
        current_queue: &mut VecDeque<(usize, Vec<usize>)>,
    ) -> AnyResult<Vec<CharacterIndex>> {
        current_queue.clear();

        let mut init_path = Vec::with_capacity(goal - start);
        init_path.push(start);
        current_queue.push_back((start, init_path));

        while let Some((vertex, path)) = current_queue.pop_front() {
            if let Some(idk) = graph.get(&vertex) {
                for position in idk {
                    if *position != goal {
                        let mut appended_path = path.clone();
                        appended_path.push(*position);
                        current_queue.push_back((*position, appended_path));
                    } else {
                        let mut appended_path = path;
                        appended_path.push(*position);
                        return Ok(appended_path);
                    };
                }
            };
        }

        Err(BFSSearchError::new(graph, start, goal).into())
    }

    #[inline(always)]
    fn one_cut<'a>(
        input: &'a CustomString,
        custom_dict: &Trie,
    ) -> AnyResult<Vec<&'a CustomStringBytesSlice>> {
        let text = input;
        let input_char_len = text.chars_len();
        let mut reused_queue: VecDeque<(usize, Vec<usize>)> = VecDeque::with_capacity(10);
        let mut graph_size: usize = 0;
        let mut graph: HashMap<CharacterIndex, Vec<CharacterIndex>> = HashMap::default();
        graph.reserve(input_char_len / 10);
        let mut result_str: Vec<&CustomStringBytesSlice> = Vec::with_capacity(input_char_len / 10);

        // all position should be refered as character index
        let valid_position = tcc_tokenizer::tcc_pos(text.raw_content());
        let text_length = input_char_len;
        let mut position_list: BinaryHeap<CharacterIndex, MinComparator> = BinaryHeap::new_min();
        let mut existing_candidate: HashSet<CharacterIndex> = HashSet::default();
        existing_candidate.reserve(input_char_len / 10);
        position_list.push(0);
        existing_candidate.insert(0);
        let mut end_position: CharacterIndex = 0;
        
        while let Some(begin_position) = position_list.pop() {
            if begin_position >= text_length {
                break;
            }
            
            let sub_text_prefix = text.substring(begin_position, text.chars_len());
            let prefixes = Trie::prefix_ref(&sub_text_prefix, custom_dict);
            
            for word in prefixes {
                let word_length = word.chars_len();
                let end_position_candidate = begin_position + word_length;
                if valid_position.contains(&end_position_candidate) {
                    graph.entry(begin_position)
                        .or_insert_with(Vec::new)
                        .push(end_position_candidate);

                    graph_size += 1;
                    if !existing_candidate.contains(&end_position_candidate) {
                        existing_candidate.insert(end_position_candidate);
                        position_list.push(end_position_candidate);
                    }
                    if graph_size > MAX_GRAPH_SIZE {
                        break;
                    }
                }
            }
            
            let position_list_length = position_list.len();
            if position_list_length == 1 {
                if let Some(first_position_list) = position_list.peek() {
                    let group_of_end_position_candidate = Self::bfs_paths_graph(
                        &graph,
                        end_position,
                        *first_position_list,
                        &mut reused_queue,
                    )?;
                    graph_size = 0; // reset our graph

                    for position in group_of_end_position_candidate.iter().skip(1) {
                        let token_bytes = text.substring_as_bytes(end_position, *position);
                        result_str.push(token_bytes);
                        end_position = *position;
                    }
                }
            } else if position_list_length == 0 {
                // no candidate, deal with non-dict word
                match NON_THAI_PATTERN.find(sub_text_prefix.raw_content()) {
                    Some(match_point) => {
                        let matched_start_char_index = match_point.start() / BYTES_PER_CHAR;
                        let matched_end_char_index = match_point.end() / BYTES_PER_CHAR;
                        end_position = begin_position
                            + sub_text_prefix
                                .raw_content()
                                .slice_by_char_indice(
                                    matched_start_char_index,
                                    matched_end_char_index,
                                )
                                .chars_len();
                    }
                    None => {
                        end_position = (begin_position + 1..text_length)
                            .find(|&position| {
                                if valid_position.contains(&position) {
                                    let prefix = text.substring(position, text_length);
                                    let list_of_prefixes = Trie::prefix_ref(&prefix, custom_dict);
                                    let valid_words: Vec<&[u8]> = list_of_prefixes
                                        .into_par_iter()
                                        .filter(|word| {
                                            let new_position = position + word.chars_len();
                                            valid_position.contains(&new_position) && !THAI_TWOCHARS_PATTERN.is_match(word)
                                        })
                                        .collect();

                                    if !valid_words.is_empty() {
                                        return true;
                                    }
                                    NON_THAI_PATTERN.is_match(prefix.raw_content())
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(text_length);
                    }
                }

                graph.entry(begin_position)
                    .or_insert_with(Vec::new)
                    .push(end_position);
                graph_size += 1;
                let token_bytes = text.substring_as_bytes(begin_position, end_position);
                result_str.push(token_bytes);
                position_list.push(end_position);
                existing_candidate.insert(end_position);
            }
        }
        Ok(result_str)
    }

    fn internal_segment(
        input: &CustomString,
        custom_dict: &Trie,
        safe: bool,
        parallel: bool,
    ) -> AnyResult<Vec<String>> {
        if input.is_empty() {
            return Ok(vec![]);
        }
        if !safe || input.chars_len() < TEXT_SCAN_END {
            let result = Self::one_cut(input, custom_dict)?;
            Ok(if parallel {
                result
                    .into_par_iter()
                    .map(CustomString::convert_raw_bytes_to_std_string)
                    .collect()
            } else {
                result
                    .into_iter()
                    .map(CustomString::convert_raw_bytes_to_std_string)
                    .collect()
            })
        } else {
            let mut txt = input.substring(0, input.chars_len());
            let mut txt_parts: Vec<CustomString> = Vec::with_capacity(txt.chars_len() / 10);
            while txt.chars_len() >= TEXT_SCAN_END {
                let sample = txt.substring(TEXT_SCAN_BEGIN, TEXT_SCAN_END);

                let cut_pos = rfind_space_char_index(sample.raw_content())
                    .map(|space_char_index| space_char_index + 1)
                    .unwrap_or_else(|| {
                        let word_tokens = Self::one_cut(&sample, custom_dict).unwrap();
                        let (token_max_index, _) = word_tokens
                            .iter()
                            .enumerate()
                            .max_by_key(|(_, token)| token.chars_len())
                            .unwrap();
                        
                        TEXT_SCAN_BEGIN + word_tokens[..token_max_index].iter().map(|token| token.chars_len()).sum::<usize>()
                    });

                txt_parts.push(txt.substring(0, cut_pos));
                txt = txt.substring(cut_pos, txt.chars_len());
            }
            if !txt.is_empty() {
                txt_parts.push(txt);
            }

            Ok(if parallel {
                txt_parts
                    .par_iter()
                    .flat_map(|part| -> AnyResult<_> {
                        let bind_part = &part.substring(0, part.chars_len());
                        let words = Self::one_cut(bind_part, custom_dict)?;
                        Ok(words
                            .into_par_iter()
                            .map(CustomString::convert_raw_bytes_to_std_string)
                            .collect::<Vec<String>>())
                    })
                    .flatten()
                    .collect()
            } else {
                txt_parts
                    .iter()
                    .flat_map(|part| -> AnyResult<_> {
                        Ok(
                            Self::one_cut(&part.substring(0, part.chars_len()), custom_dict)?
                                .iter()
                                .map(|word| CustomString::convert_raw_bytes_to_std_string(word))
                                .collect::<Vec<String>>(),
                        )
                    })
                    .flatten()
                    .collect()
            })
        }
    }
}

impl Tokenizer for NewmmTokenizer {
    fn segment(&self, text: &str, safe: bool, parallel: bool) -> AnyResult<Vec<String>> {
        Self::internal_segment(&CustomString::new(text), &self.dict, safe, parallel)
    }

    fn segment_to_string(&self, text: &str, safe: bool, parallel: bool) -> Vec<String> {
        self.segment(text, safe, parallel).unwrap_or_default()
    }
}
