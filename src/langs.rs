#![allow(clippy::type_complexity)]

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
    fn should_indent(&self, line: &[u8]) -> bool;
    fn should_dedent(&self, ch: char) -> bool;
}

impl<T: Language + ?Sized> Language for &T {
    fn split_words(&self, code: &[u8]) -> Vec<Word> {
        (**self).split_words(code)
    }

    fn should_indent(&self, line: &[u8]) -> bool {
        (**self).should_indent(line)
    }

    fn should_dedent(&self, ch: char) -> bool {
        (**self).should_dedent(ch)
    }
}

pub const fn rgb_color(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

macro_rules! extension {
    ($ext:literal) => {
        |name| name.split('.').last().filter(|ext| *ext == $ext).is_some()
    };
}

macro_rules! exact {
    ($name:literal) => {
        // TODO: windows strikes again
        |name| {
            name.split('/')
                .last()
                .filter(|name| *name == $name)
                .is_some()
        }
    };
}

mod commit;
mod plaintext;
mod python;
mod rust;

const LANGS: &[(&str, fn(&str) -> bool, &dyn Language)] = &[
    ("rust", extension!("rs"), &rust::Rust),
    ("python", extension!("py"), &python::Python),
    ("git-commit", exact!("COMMIT_EDITMSG"), &commit::Commit),
    ("plaintext", |_| true, &plaintext::Plaintext),
];
pub const DEFAULT_LANG: &str = "plaintext";

pub fn lang_from_name(name: &str) -> Option<&'static dyn Language> {
    LANGS
        .iter()
        .find(|(lang_name, _, _)| *lang_name == name)
        .map(|(_, _, lang)| *lang)
}

pub fn lang_from_filename(name: &str) -> Option<&'static dyn Language> {
    LANGS
        .iter()
        .find(|(_, validator, _)| validator(name))
        .map(|(_, _, lang)| *lang)
}
