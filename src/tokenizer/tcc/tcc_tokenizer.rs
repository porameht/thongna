use super::tcc_rules::{LOOKAHEAD_TCC, NON_LOOKAHEAD_TCC};

use crate::bytes_str::custom_string::{
    CustomStringBytesSlice, FixedCharsLengthByteSlice, BYTES_PER_CHAR,
};
use rustc_hash::FxHashSet as HashSet;

/// Returns a set of "character" indices at the end of each token
#[inline]
pub fn tcc_pos(custom_text_type: &CustomStringBytesSlice) -> HashSet<usize> {
    let mut set = HashSet::with_capacity_and_hasher(custom_text_type.chars_len() / 10, Default::default());
    let mut txt = custom_text_type;
    let mut position: usize = 0;

    while !txt.is_empty() {
        if let Some(result) = NON_LOOKAHEAD_TCC.find(txt) {
            let matched = &txt[result.start()..result.end()];
            let match_length = matched.len();

            if LOOKAHEAD_TCC.is_match(matched) {
                let end_char_index = (match_length - BYTES_PER_CHAR) / BYTES_PER_CHAR;
                position += end_char_index;
                set.insert(position);
                txt = txt.slice_by_char_indice(end_char_index, txt.chars_len());
            } else {
                let segment_size = match_length / BYTES_PER_CHAR;
                position += segment_size;
                set.insert(position);
                txt = txt.slice_by_char_indice(segment_size, txt.chars_len());
            }
        } else {
            // not thai
            position += 1;
            set.insert(position);
            txt = txt.slice_by_char_indice(1, txt.chars_len());
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytes_str::custom_string::CustomString;

    #[test]
    fn test_cluster_karan() {
        let kr_result = tcc_pos(CustomString::new("พิสูจน์ได้ค่ะ").raw_content());
        assert!(kr_result.contains(&2));
        assert!(kr_result.contains(&7));
        assert!(kr_result.contains(&10));
        assert!(kr_result.contains(&13));
    }

    #[test]
    fn test_cluster_general_case() {
        let gen_result = tcc_pos(CustomString::new("เรือน้อยลอยอยู่").raw_content());
        assert!(gen_result.contains(&4));
        assert!(gen_result.contains(&6));
        assert!(gen_result.contains(&7));
        assert!(gen_result.contains(&8));
        assert!(gen_result.contains(&9));
        assert!(gen_result.contains(&10));
        assert!(gen_result.contains(&11));
        assert!(gen_result.contains(&12));
        assert!(gen_result.contains(&15));
    }
}
