use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use regex::Regex;

static NORMALIZE_RULE1: [&str; 23] = [
    "ะ", "ั", "็", "า", "ิ", "ี", "ึ", "่", "ํ", "ุ", "ู", "ใ", "ไ", "โ", "ื", "่", "้", "๋", "๊", "ึ", "์", "๋", "ำ"
];

static NORMALIZE_RULE2: [(&str, &str); 9] = [
    ("เเ", "แ"),
    ("ู้", "ู้"),
    ("ํา", "ำ"),
    ("ํ(t)า", "\\1ำ"),
    ("ํา(t)", "\\1ำ"),
    ("([่-๋])([ัิ-ื])", "\\2\\1"),
    ("([่-๋])([ูุ])", "\\2\\1"),
    ("ำ([่-๋])", "\\1ำ"),
    ("(์)([ัิ-ู])", "\\2\\1")
];

#[pyfunction]
pub fn normalize(text: &str, whitespace_number: bool) -> PyResult<String> {
    let mut text = text.to_string();

    if whitespace_number {
        let re = Regex::new(r"([0-9]+)").unwrap();
        text = re.replace_all(&text, " $1 ").to_string();
    }

    text = text.replace("\u{200b}", "");
    text = Regex::new(r" {2,}").unwrap().replace_all(&text, " ").to_string();
    text = Regex::new(r"\t{2,}").unwrap().replace_all(&text, "\t").to_string();
    text = Regex::new(r"\n{2,}").unwrap().replace_all(&text, "\n").to_string();

    for (pattern, replacement) in &NORMALIZE_RULE2 {
        let pattern = pattern.replace("t", "[่้๊๋]");
        let re = Regex::new(&pattern).unwrap();
        text = re.replace_all(&text, *replacement).to_string();
    }

    for &rule in &NORMALIZE_RULE1 {
        let pattern = format!("{}+", rule.replace("t", "[่้๊๋]"));
        let re = Regex::new(&pattern).unwrap();
        text = re.replace_all(&text, rule).to_string();
    }

    Ok(text)
}

#[pymodule]
fn thongna(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    Ok(())
}
