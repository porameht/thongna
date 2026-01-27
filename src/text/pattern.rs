//! Regex pattern conversion for fixed-width Unicode strings.
//!
//! This module translates normal, human-readable Thai regex patterns
//! into 4-byte zero-left-padded byte regex pattern strings.

use anyhow::{Error as AnyError, Result};
use regex_syntax::{
    hir::{Anchor, Class, Group, Literal as LiteralEnum, Repetition},
    hir::{ClassUnicodeRange, Hir, HirKind},
    is_meta_character, Parser,
};
use std::{error::Error, fmt::Display};

trait ToFixedWidthRepr {
    fn to_fixed_width_repr(&self) -> Result<String>;
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants reserved for future regex feature support
enum UnsupportedPatternError {
    ByteLiteral,
    ByteClass,
    DifferentRanges(char, char),
    RepetitionRange,
    AnchorStartLine,
    AnchorEndLine,
}

enum IterableHirKind {
    Alternation(Vec<Hir>),
    Concat(Vec<Hir>),
}

impl Display for UnsupportedPatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByteLiteral => {
                write!(f, "Byte literal is not supported")
            }
            UnsupportedPatternError::ByteClass => {
                write!(f, "Byte class is not supported")
            }
            UnsupportedPatternError::DifferentRanges(a, b) => {
                write!(
                    f,
                    "Different byte length range is not supported {} {}",
                    a, b
                )
            }
            UnsupportedPatternError::RepetitionRange => {
                write!(f, "Repetition range is not supported")
            }
            UnsupportedPatternError::AnchorStartLine => {
                write!(
                    f,
                    "Start line anchor (^) is not supported in multiline mode"
                )
            }
            UnsupportedPatternError::AnchorEndLine => {
                write!(f, "End line anchor ($) is not supported in multiline mode")
            }
        }
    }
}
impl Error for UnsupportedPatternError {}

impl ToFixedWidthRepr for Hir {
    fn to_fixed_width_repr(&self) -> Result<String> {
        self.kind().to_fixed_width_repr()
    }
}

impl ToFixedWidthRepr for HirKind {
    fn to_fixed_width_repr(&self) -> Result<String> {
        match self {
            HirKind::Empty => todo!(),
            HirKind::Literal(l) => l.to_fixed_width_repr(),
            HirKind::Class(c) => c.to_fixed_width_repr(),
            HirKind::Anchor(a) => a.to_fixed_width_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(r) => r.to_fixed_width_repr(),
            HirKind::Group(g) => g.to_fixed_width_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_fixed_width_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_fixed_width_repr()
            }
        }
    }
}

impl ToFixedWidthRepr for Anchor {
    fn to_fixed_width_repr(&self) -> Result<String> {
        match self {
            Anchor::StartLine => todo!(),
            Anchor::EndLine => todo!(),
            Anchor::StartText => Ok("^".to_string()),
            Anchor::EndText => Ok("$".to_string()),
        }
    }
}

impl ToFixedWidthRepr for LiteralEnum {
    fn to_fixed_width_repr(&self) -> Result<String> {
        match self {
            LiteralEnum::Unicode(a) => Ok(a.to_four_byte_string()),
            LiteralEnum::Byte(_) => Err(AnyError::new(UnsupportedPatternError::ByteLiteral)),
        }
    }
}

impl ToFixedWidthRepr for Class {
    fn to_fixed_width_repr(&self) -> Result<String> {
        match self {
            Class::Unicode(u) => Ok(u.ranges().to_four_byte_string()),
            Class::Bytes(_) => Err(AnyError::from(UnsupportedPatternError::ByteClass)),
        }
    }
}

impl ToFixedWidthRepr for Repetition {
    fn to_fixed_width_repr(&self) -> Result<String> {
        use regex_syntax::hir::{RepetitionKind, RepetitionRange};

        let symbol = match &self.kind {
            RepetitionKind::ZeroOrOne => "?".to_string(),
            RepetitionKind::ZeroOrMore => "*".to_string(),
            RepetitionKind::OneOrMore => "+".to_string(),
            RepetitionKind::Range(r) => match r {
                RepetitionRange::Exactly(e) => format!("{{{}}}", e),
                RepetitionRange::AtLeast(l) => format!("{{{},}}", l),
                RepetitionRange::Bounded(start, end) => format!("{{{},{}}}", start, end),
            },
        };

        let repeated_expression = match &self.hir.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(l) => l.to_fixed_width_repr(),
            HirKind::Class(c) => c.to_fixed_width_repr(),
            HirKind::Anchor(a) => a.to_fixed_width_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(r) => r.to_fixed_width_repr(),
            HirKind::Group(g) => g.to_fixed_width_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_fixed_width_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_fixed_width_repr()
            }
        }?;

        if matches!(self.hir.kind(), HirKind::Group(_)) {
            Ok(format!("{}{}", repeated_expression, symbol))
        } else {
            Ok(format!("({}){}", repeated_expression, symbol))
        }
    }
}

/// Convert a single HIR member to its fixed-width representation.
fn member_to_fixed_width(member: &Hir) -> Result<String> {
    match member.kind() {
        HirKind::Empty => todo!(),
        HirKind::Literal(lit) => lit.to_fixed_width_repr(),
        HirKind::Class(c) => c.to_fixed_width_repr(),
        HirKind::Anchor(a) => a.to_fixed_width_repr(),
        HirKind::WordBoundary(_) => todo!(),
        HirKind::Repetition(r) => r.to_fixed_width_repr(),
        HirKind::Group(g) => g.to_fixed_width_repr(),
        HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_fixed_width_repr(),
        HirKind::Alternation(a) => IterableHirKind::Alternation(a.to_vec()).to_fixed_width_repr(),
    }
}

impl ToFixedWidthRepr for IterableHirKind {
    fn to_fixed_width_repr(&self) -> Result<String> {
        match self {
            IterableHirKind::Alternation(members) => {
                let mut result = String::new();
                for member in members {
                    let repr = member_to_fixed_width(member)?;

                    if matches!(member.kind(), HirKind::Alternation(_)) {
                        result.push_str(&repr);
                    } else if result.is_empty() && matches!(member.kind(), HirKind::Concat(_)) {
                        result = repr;
                    } else if result.is_empty() {
                        result = format!("({})", repr);
                    } else {
                        result.push_str(&format!("|({})", repr));
                    }
                }
                Ok(result)
            }
            IterableHirKind::Concat(members) => {
                let mut result = String::new();
                for member in members {
                    result.push_str(&member_to_fixed_width(member)?);
                }
                Ok(result)
            }
        }
    }
}

impl ToFixedWidthRepr for Group {
    fn to_fixed_width_repr(&self) -> Result<String> {
        let inner = match self.hir.kind() {
            HirKind::Empty => todo!(),
            HirKind::Literal(lit) => lit.to_fixed_width_repr(),
            HirKind::Class(c) => c.to_fixed_width_repr(),
            HirKind::Anchor(a) => a.to_fixed_width_repr(),
            HirKind::WordBoundary(_) => todo!(),
            HirKind::Repetition(_) => todo!(),
            HirKind::Group(g) => g.to_fixed_width_repr(),
            HirKind::Concat(c) => IterableHirKind::Concat(c.to_vec()).to_fixed_width_repr(),
            HirKind::Alternation(a) => {
                IterableHirKind::Alternation(a.to_vec()).to_fixed_width_repr()
            }
        }?;
        Ok(format!("({})", inner))
    }
}

fn get_char_range_byte_class(class_range: &ClassUnicodeRange) -> Option<UTFBytesLength> {
    let start_class = char_class(class_range.start());
    let end_class = char_class(class_range.end());
    if start_class == end_class {
        Some(start_class)
    } else {
        None
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum UTFBytesLength {
    One,
    Two,
    Three,
    Four,
}

fn char_class(character: char) -> UTFBytesLength {
    let mut bytes_buffer: [u8; 4] = [0; 4];

    character.encode_utf8(&mut bytes_buffer);
    match bytes_buffer {
        [_a, 0, 0, 0] => UTFBytesLength::One,
        [_a, _b, 0, 0] => UTFBytesLength::Two,
        [_a, _b, _c, 0] => UTFBytesLength::Three,
        _ => UTFBytesLength::Four,
    }
}

trait PadLeftZeroFourBytesRep {
    fn to_four_byte_string(&self) -> String;
}

fn escape_meta_character(c: char) -> String {
    if is_meta_character(c) {
        format!(r"\{}", c)
    } else if c.is_whitespace() {
        format!("{:?}", c).replace('\'', "")
    } else {
        c.to_string()
    }
}

impl PadLeftZeroFourBytesRep for &[ClassUnicodeRange] {
    fn to_four_byte_string(&self) -> String {
        let urange = self;
        let char_classes = urange
            .iter()
            .map(get_char_range_byte_class)
            .collect::<Vec<_>>();

        if char_classes.iter().all(|elem| elem.is_some()) {
            let the_class = char_classes.first().unwrap().unwrap();

            if char_classes.iter().all(|elem| elem.unwrap() == the_class) {
                let pad_left_0 = match the_class {
                    UTFBytesLength::One => r"\x00\x00\x00",
                    UTFBytesLength::Two => r"\x00\x00",
                    UTFBytesLength::Three => r"\x00",
                    UTFBytesLength::Four => r"",
                };
                let mut output_four_bytes_rep: Vec<String> = vec![];
                for regex_range in urange.iter() {
                    let (start, end) = (regex_range.start(), regex_range.end());
                    if start == end {
                        output_four_bytes_rep
                            .push(escape_meta_character(end).to_string().replace("'", ""));
                    } else {
                        output_four_bytes_rep.push(
                            format!(
                                r"{}-{}",
                                escape_meta_character(start),
                                escape_meta_character(end)
                            )
                            .replace('\'', ""),
                        );
                    }
                }
                format!(r"{}[{}]", pad_left_0, output_four_bytes_rep.join(""))
            } else {
                println!("{:?}", self);
                todo!()
            }
        } else {
            todo!()
        }
    }
}

impl PadLeftZeroFourBytesRep for char {
    fn to_four_byte_string(&self) -> String {
        let mut bytes_buffer: [u8; 4] = [0; 4];
        self.encode_utf8(&mut bytes_buffer);

        match bytes_buffer {
            [_, 0, 0, 0] => {
                if self.is_alphanumeric() || *self == ' ' {
                    format!(r"\x00\x00\x00{}", self)
                } else {
                    format!(r"\x00\x00\x00{:?}", self).replace('\'', "")
                }
            }
            [_, _, 0, 0] => format!(r"\x00\x00{}", self),
            [_, _, _, 0] => format!(r"\x00{}", self),
            _ => self.to_string(),
        }
    }
}

/// Converts a standard regex pattern to a fixed-width compatible pattern.
pub fn to_fixed_width_pattern(regex_pattern: &str) -> Result<String> {
    let hir = Parser::new().parse(regex_pattern)?;
    hir.to_fixed_width_repr()
}
