use crate::*;
use crossterm::style::{Attribute, Color};

fn is_quote(ch: char) -> bool {
    ch == '"' || ch == '\''
}

fn is_comment(code: &[char]) -> bool {
    !code.is_empty() && code[0] == '#'
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
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

fn is_type(word: &str) -> bool {
    match word {
        "bool" | "bytearray" | "bytes" | "complex" | "dict" | "float" | "frozenset" | "int"
        | "list" | "memoryview" | "object" | "range" | "set" | "slice" | "str" | "super"
        | "tuple" | "type" => true,
        w => w.chars().next().filter(char::is_ascii_uppercase).is_some() && !w.contains('_'),
    }
}

pub struct Python;

impl Language for Python {
    fn split_words(&self, mut code: &[char]) -> Vec<Word> {
        fn is_ch_usable(ch: char) -> bool {
            is_ident(ch) || is_quote(ch)
        }

        let mut words = Vec::new();

        let mut pos = 0;
        while !code.is_empty() {
            while !code.is_empty() && !is_ch_usable(code[0]) {
                if is_comment(code) {
                    let mut word = String::new();
                    word.push(code[0]);
                    pos += 1;
                    code = &code[1..];
                    // NOTE: the '\n' case will never show up because we only call this function
                    // on individual lines. It's here just in case that changes.
                    while !code.is_empty() && code[0] != '\n' {
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

            if !code.is_empty() && is_quote(code[0]) {
                let quote = code[0];
                word.push(quote as char);
                pos += 1;
                code = &code[1..];
                while !code.is_empty() && code[0] != quote {
                    word.push(code[0] as char);
                    pos += 1;
                    code = &code[1..];
                }
                if !code.is_empty() {
                    word.push(code[0] as char);
                    pos += 1;
                    code = &code[1..];
                }
                color = Color::DarkGreen;
                attr = Attribute::Reset;
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
                if code.first().filter(|ch| ch == &&'(').is_some() {
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

    fn should_indent(&self, code: &[char]) -> bool {
        matches!(code.last(), Some(&':') | Some(&'(') | Some(&'{') | Some(&'['))
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
