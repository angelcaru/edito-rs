use crossterm::style::{Attribute, Color};

#[derive(Debug, Clone)]
pub struct Word {
    pub col: usize,
    pub text: String,
    pub color: Color,
    pub attr: Attribute,
}

pub trait Language {
    fn split_words(&self, code: &[u8]) -> Vec<Word>;
}

mod rust;
pub use rust::*;
