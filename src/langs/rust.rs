use crate::*;
use crossterm::style::{Attribute, Color};

fn is_quote(ch: char) -> bool {
    ch == '"' || ch == '\''
}

fn is_comment(code: &str) -> bool {
    code.len() >= 2 && code.chars().next().unwrap() == '/' && code.chars().nth(1).unwrap() == '/'
}

fn is_ident(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn is_number(word: &str) -> bool {
    for ch in word.chars() {
        if !ch.is_ascii_digit() {
            return false;
        }
    }
    true
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
            | "macro_rules"
            | "union"
    )
}

fn is_type(word: &str) -> bool {
    match word {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" | "char" | "bool" | "str" => true,
        w => w.chars().next().filter(char::is_ascii_uppercase).is_some() && !w.contains('_'),
    }
}

pub struct Rust;

impl Language for Rust {
    fn split_words(&self, mut code: &str) -> Vec<Word> {
        fn is_ch_usable(ch: char) -> bool {
            is_ident(ch) || is_quote(ch)
        }

        let mut words = Vec::new();

        let mut pos = 0;
        while !code.is_empty() {
            while !code.is_empty() && !is_ch_usable(code.chars().next().unwrap()) {
                if is_comment(code) {
                    let mut word = String::new();
                    word.push(code.chars().next().unwrap());
                    pos += 1;
                    code = &code[1..];
                    // NOTE: the '\n' case will never show up because we only call this function
                    // on individual lines. It's here just in case that changes.
                    while !code.is_empty() && code.chars().next().unwrap() != '\n' {
                        word.push(code.chars().next().unwrap());
                        pos += 1;
                        code = &code[1..];
                    }
                    if !code.is_empty() {
                        word.push(code.chars().next().unwrap());
                        pos += 1;
                        code = &code[1..];
                    }
                    words.push(Word {
                        col: pos - word.len(),
                        text: word,
                        color: rgb_color(100, 100, 100),
                        attr: Attribute::Italic,
                    });
                }
                if !code.is_empty() {
                    pos += 1;
                    code = &code[1..];
                }
            }

            let mut word = String::new();
            let color;
            let attr;

            if !code.is_empty() && is_quote(code.chars().next().unwrap()) {
                let quote = code.chars().next().unwrap();
                word.push(code.chars().next().unwrap());
                pos += 1;
                code = &code[1..];
                while !code.is_empty() && code.chars().next().unwrap() != quote {
                    word.push(code.chars().next().unwrap());
                    pos += 1;
                    code = &code[1..];
                }
                if !code.is_empty() {
                    word.push(code.chars().next().unwrap());
                    pos += 1;
                    code = &code[1..];
                }
                color = Color::DarkGreen;
                attr = Attribute::Reset;
            } else {
                while !code.is_empty() && !is_ch_usable(code.chars().next().unwrap()) {
                    pos += 1;
                    code = &code[1..];
                }
                while !code.is_empty() && is_ident(code.chars().next().unwrap()) {
                    word.push(code.chars().next().unwrap());
                    pos += 1;
                    code = &code[1..];
                }
                if code.chars().next().filter(|&ch| ch == '!').is_some() {
                    color = Color::DarkGreen;
                    attr = Attribute::Reset;
                } else if code.chars().next().filter(|&ch| ch == '(').is_some() {
                    color = rgb_color(140, 201, 26);
                    attr = Attribute::Bold;
                } else if is_keyword(&word) {
                    color = Color::Yellow;
                    attr = Attribute::Bold;
                } else if is_type(&word) {
                    color = Color::Green;
                    attr = Attribute::Reset;
                } else if is_number(&word) {
                    color = rgb_color(201, 146, 18);
                    attr = Attribute::Reset;
                } else {
                    color = rgb_color(76, 111, 217);
                    attr = Attribute::Reset;
                }
            }
            words.push(Word {
                col: pos - word.len(),
                text: word,
                color,
                attr,
            });
        }

        words
    }

    fn should_indent(&self, line: &str) -> bool {
        line.chars().last() == Some('{')
    }

    fn should_dedent(&self, ch: char) -> bool {
        ch == '}'
    }
}
