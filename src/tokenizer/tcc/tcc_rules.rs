use crate::bytes_str::custom_regex::regex_pattern_to_custom_pattern;
use lazy_static::lazy_static;
use regex::bytes::Regex;

#[inline(always)]
pub fn replace_tcc_symbol(tcc_pattern: &str) -> String {
    tcc_pattern
        .replace('k', "(cc?[dิ]?[์])?")
        .replace('c', "[ก-ฮ]")
        .replace('t', "[่-๋]?")
        .replace('d', "ูุ")
}

lazy_static! {
    pub static ref NON_LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^เc็ck",
            r"^เcctาะk",
            r"^เccีtยะk",
            r"^เcc็ck",
            r"^เcิc์ck",
            r"^เcิtck",
            r"^เcีtยะ?k",
            r"^เcืtอะ?k",
            r"^เctา?ะ?k",
            r"^cัtวะk",
            r"^c[ัื]tc[ุิะ]?k",
            r"^c[ิุู]์k",
            r"^c[ะ-ู]tk",
            r"^cรรc์ ็",
            r"^c็",
            r"^ct[ะาำ]?k",
            r"^ck",
            r"^แc็c",
            r"^แcc์",
            r"^แctะ",
            r"^แcc็c",
            r"^แccc์",
            r"^โctะ",
            r"^[เ-ไ]ct",
            r"^ก็",
            r"^อึ",
            r"^หึ",
            r"^(เccีtย)[เ-ไก-ฮ]k",
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k",
        ]
        .iter()
        .map(|&pattern| regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap())
        .collect::<Vec<_>>()
        .join("|")
    )
    .unwrap();

    pub static ref LOOKAHEAD_TCC: Regex = Regex::new(
        &[
            r"^(เccีtย)[เ-ไก-ฮ]k",
            r"^(เc[ิีุู]tย)[เ-ไก-ฮ]k"
        ]
        .iter()
        .map(|&pattern| regex_pattern_to_custom_pattern(&replace_tcc_symbol(pattern)).unwrap())
        .collect::<Vec<_>>()
        .join("|")
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcc_regex_test_cases() {
        let test_cases = [
            ("^เc็ck", r"^\x00เ\x00[ก-ฮ]\x00็\x00[ก-ฮ](\x00[ก-ฮ](\x00[ก-ฮ])?(\x00[ิุ-ู])?\x00[์])?"),
            ("^เcctาะ", r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ](\x00[่-๋])?\x00า\x00ะ"),
            ("^เccีtยะ", r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย\x00ะ"),
            ("^เcc็c", r"^\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]"),
            ("^เcิc์c", r"^\x00เ\x00[ก-ฮ]\x00ิ\x00[ก-ฮ]\x00์\x00[ก-ฮ]"),
            ("^เcิtc", r"^\x00เ\x00[ก-ฮ]\x00ิ(\x00[่-๋])?\x00[ก-ฮ]"),
            ("^เcีtยะ?", r"^\x00เ\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย(\x00ะ)?"),
            ("^เcืtอะ?", r"^\x00เ\x00[ก-ฮ]\x00ื(\x00[่-๋])?\x00อ(\x00ะ)?"),
            ("^เctา?ะ?", r"^\x00เ\x00[ก-ฮ](\x00[่-๋])?(\x00า)?(\x00ะ)?"),
            ("^cัtวะ", r"^\x00[ก-ฮ]\x00ั(\x00[่-๋])?\x00ว\x00ะ"),
            ("^c[ัื]tc[ุิะ]?", r"^\x00[ก-ฮ]\x00[ัื](\x00[่-๋])?\x00[ก-ฮ](\x00[ะิุ])?"),
            ("^c[ิุู]์", r"^\x00[ก-ฮ]\x00[ิุ-ู]\x00์"),
            ("^c[ะ-ู]t", r"^\x00[ก-ฮ]\x00[ะ-ู](\x00[่-๋])?"),
            ("^c็", r"^\x00[ก-ฮ]\x00็"),
            ("^ct[ะาำ]?", r"^\x00[ก-ฮ](\x00[่-๋])?(\x00[ะา-ำ])?"),
            ("^แc็c", r"^\x00แ\x00[ก-ฮ]\x00็\x00[ก-ฮ]"),
            ("^แcc์", r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00์"),
            ("^แctะ", r"^\x00แ\x00[ก-ฮ](\x00[่-๋])?\x00ะ"),
            ("^แcc็c", r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00็\x00[ก-ฮ]"),
            ("^แccc์", r"^\x00แ\x00[ก-ฮ]\x00[ก-ฮ]\x00[ก-ฮ]\x00์"),
            ("^โctะ", r"^\x00โ\x00[ก-ฮ](\x00[่-๋])?\x00ะ"),
            ("^[เ-ไ]ct", r"^\x00[เ-ไ]\x00[ก-ฮ](\x00[่-๋])?"),
        ];

        for (input, expected) in test_cases.iter() {
            let result = regex_pattern_to_custom_pattern(&replace_tcc_symbol(input)).unwrap();
            assert_eq!(&result, expected, "Failed for input: {}", input);
        }

        let look_ahead_cases = [
            (r"^(เccีtย)[เ-ไก-ฮ]", r"^(\x00เ\x00[ก-ฮ]\x00[ก-ฮ]\x00ี(\x00[่-๋])?\x00ย)\x00[ก-ฮเ-ไ]"),
            (r"^(เc[ิีุู]tย)[เ-ไก-ฮ]", r"^(\x00เ\x00[ก-ฮ]\x00[ิ-ีุ-ู](\x00[่-๋])?\x00ย)\x00[ก-ฮเ-ไ]"),
        ];

        for (input, expected) in look_ahead_cases.iter() {
            let result = regex_pattern_to_custom_pattern(&replace_tcc_symbol(input)).unwrap();
            assert_eq!(&result, expected, "Failed for look-ahead input: {}", input);
        }
    }

    #[test]
    fn newmm_exception_match_cases() {
        let test_cases = [
            (r"(?x)^\r?\n", r"^(\x00\x00\x00\r)?\x00\x00\x00\n"),
            (r"^[ \t]+", r"^(\x00\x00\x00[\t ])+"),
            (r"(?x)^[-a-zA-Z]+", r"^(\x00\x00\x00[\-A-Za-z])+"),
            (r"(?x)^[๐-๙]+([,\.][๐-๙]+)*", r"^(\x00[๐-๙])+(\x00\x00\x00[,\.](\x00[๐-๙])+)*"),
            (r"(?x)^[0-9]+([,\.][0-9]+)*", r"^(\x00\x00\x00[0-9])+(\x00\x00\x00[,\.](\x00\x00\x00[0-9])+)*"),
            (r"^[ก-ฮ]{0,2}$", r"^(\x00[ก-ฮ]){0,2}$"),
        ];

        for (input, expected) in test_cases.iter() {
            let result = regex_pattern_to_custom_pattern(input).unwrap();
            assert_eq!(&result, expected, "Failed for input: {}", input);
        }
    }
}
