use crate::{Language, Word};
use crossterm::style::{Attribute, Color};

pub struct Plaintext;
impl Language for Plaintext {
    fn split_words(&self, code: &[u8]) -> Vec<Word> {
        vec![Word {
            col: 0,
            text: String::from_utf8_lossy(code).to_string(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }
}
