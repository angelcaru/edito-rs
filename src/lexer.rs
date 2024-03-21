use crossterm::style::{Attribute, Color};

#[derive(Debug, Clone)]
pub struct Word {
    pub col: usize,
    pub text: String,
    pub is_fn: bool,
    pub is_macro: bool,
}

fn is_quote(ch: u8) -> bool {
    ch == b'"' || ch == b'\''
}

fn is_comment(code: &[u8]) -> bool {
    code.len() >= 2 && code[0] == b'/' && code[1] == b'/'
}

fn is_ident(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_'
}

pub fn split_words(mut code: &[u8]) -> Vec<Word> {
    fn is_ch_usable(ch: u8) -> bool {
        is_ident(ch) || is_quote(ch)
    }

    let mut words = Vec::new();

    let mut pos = 0;
    while !code.is_empty() {
        while !code.is_empty() && !is_ch_usable(code[0]) {
            if is_comment(code) {
                let mut word = String::new();
                word.push(code[0] as char);
                pos += 1;
                code = &code[1..];
                // NOTE: the '\n' case will never show up because we only call this function
                // on individual lines. It's here just in case that changes.
                while !code.is_empty() && code[0] != b'\n' {
                    word.push(code[0] as char);
                    pos += 1;
                    code = &code[1..];
                }
                if !code.is_empty() {
                    word.push(code[0] as char);
                    pos += 1;
                    code = &code[1..];
                }
                words.push(Word {
                    col: pos - word.len(),
                    text: word,
                    is_fn: false,
                    is_macro: false,
                });
            }
            if !code.is_empty() {
                pos += 1;
                code = &code[1..];
            }
        }

        let mut word = String::new();

        if !code.is_empty() && is_quote(code[0]) {
            word.push(code[0] as char);
            pos += 1;
            code = &code[1..];
            while !code.is_empty() && !is_quote(code[0]) {
                word.push(code[0] as char);
                pos += 1;
                code = &code[1..];
            }
            if !code.is_empty() {
                word.push(code[0] as char);
                pos += 1;
                code = &code[1..];
            }
        } else {
            while !code.is_empty() && !is_ch_usable(code[0]) {
                pos += 1;
                code = &code[1..];
            }
            while !code.is_empty() && is_ident(code[0]) {
                word.push(code[0] as char);
                pos += 1;
                code = &code[1..];
            }
        }
        if code.get(0).filter(|ch| ch == &&b'!').is_some() {
            word.push(code[0] as char);
            pos += 1;
            code = &code[1..];
            words.push(Word {
                col: pos - word.len(),
                text: word,
                is_fn: false,
                is_macro: true,
            });
        } else {
            words.push(Word {
                col: pos - word.len(),
                text: word,
                is_fn: code.get(0).filter(|ch| ch == &&b'(').is_some(),
                is_macro: false,
            });
        }
    }

    words
}

fn is_keyword(word: &str) -> bool {
    match word {
        "as" => true,
        "break" => true,
        "const" => true,
        "continue" => true,
        "crate" => true,
        "else" => true,
        "enum" => true,
        "extern" => true,
        "false" => true,
        "fn" => true,
        "for" => true,
        "if" => true,
        "impl" => true,
        "in" => true,
        "let" => true,
        "loop" => true,
        "match" => true,
        "mod" => true,
        "move" => true,
        "mut" => true,
        "pub" => true,
        "ref" => true,
        "return" => true,
        "self" => true,
        "Self" => true,
        "static" => true,
        "struct" => true,
        "super" => true,
        "trait" => true,
        "true" => true,
        "type" => true,
        "unsafe" => true,
        "use" => true,
        "where" => true,
        "while" => true,
        "async" => true,
        "await" => true,
        "dyn" => true,
        "abstract" => true,
        "become" => true,
        "box" => true,
        "do" => true,
        "final" => true,
        "macro" => true,
        "override" => true,
        "priv" => true,
        "typeof" => true,
        "unsized" => true,
        "virtual" => true,
        "yield" => true,
        "try" => true,
        "macro_rules" => true,
        "union" => true,
        _ => false,
    }
}

fn is_type(word: &str) -> bool {
    match word {
        "i8" => true,
        "i16" => true,
        "i32" => true,
        "i64" => true,
        "i128" => true,
        "isize" => true,
        "u8" => true,
        "u16" => true,
        "u32" => true,
        "u64" => true,
        "u128" => true,
        "usize" => true,
        "f32" => true,
        "f64" => true,
        "char" => true,
        "bool" => true,
        "str" => true,
        w => w.chars().next().filter(char::is_ascii_uppercase).is_some() && !w.contains('_'),
    }
}

fn is_string(word: &str) -> bool {
    !word.is_empty() && is_quote(word.as_bytes()[0])
}

fn rgb_color(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

pub trait Styled {
    fn color(&self) -> Color;
    fn attr(&self) -> Attribute;
}

impl Styled for Word {
    fn color(&self) -> Color {
        if is_keyword(&self.text) {
            Color::Yellow
        } else if is_type(&self.text) {
            Color::Green
        } else if is_string(&self.text) {
            Color::DarkGreen
        } else if is_comment(self.text.as_bytes()) {
            rgb_color(100, 100, 100)
        } else if self.is_fn {
            rgb_color(140, 201, 26)
        } else if self.is_macro {
            Color::DarkGreen
        } else {
            rgb_color(76, 111, 217)
        }
    }

    fn attr(&self) -> Attribute {
        if is_keyword(&self.text) {
            Attribute::Bold
        } else if self.is_fn {
            Attribute::Bold
        } else {
            Attribute::Reset
        }
    }
}
