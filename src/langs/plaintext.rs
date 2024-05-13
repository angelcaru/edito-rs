use crate::{Language, Word};
use crossterm::style::{Attribute, Color};

pub struct Plaintext;
impl Language for Plaintext {
    fn split_words(&self, code: &str) -> Vec<Word> {
        vec![Word {
            col: 0,
            text: code.into(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }

    fn should_indent(&self, _line: &str) -> bool {
        false
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
