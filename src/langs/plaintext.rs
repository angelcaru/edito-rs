use crate::{Language, Word};
use crossterm::style::{Attribute, Color};

pub struct Plaintext;
impl Language for Plaintext {
    fn split_words(&self, code: &[char]) -> Vec<Word> {
        vec![Word {
            col: 0,
            text: code.iter().collect::<String>(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }

    fn should_indent(&self, _line: &[char]) -> bool {
        false
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
