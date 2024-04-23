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

impl<T: Language + ?Sized> Language for &T {
    fn split_words(&self, code: &[u8]) -> Vec<Word> {
        (**self).split_words(code)
    }
}

pub const fn rgb_color(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

mod rust;
mod plaintext;

const LANGS: &[(&str, &str, &dyn Language)] = &[
    ("rust", "rs", &rust::Rust),
    ("plaintext", "txt", &plaintext::Plaintext),
];
pub const DEFAULT_LANG: &str = "plaintext";

pub fn lang_from_name(name: &str) -> Option<&'static dyn Language> {
    LANGS
        .iter()
        .find(|(lang_name, _, _)| *lang_name == name)
        .map(|(_, _, lang)| *lang)
}

pub fn lang_from_extension(ext: &str) -> Option<&'static dyn Language> {
    LANGS
        .iter()
        .find(|(_, ext_name, _)| *ext_name == ext)
        .map(|(_, _, lang)| *lang)
}

