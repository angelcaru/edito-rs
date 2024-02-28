use crossterm::style::Color;

pub type Word = (usize, String);

fn is_quote(ch: u8) -> bool {
    ch == b'"' || ch == b'\''
}

fn is_comment(code: &[u8]) -> bool {
    code.len() >= 2 && code[0] == b'/' && code[1] == b'/'
}

pub fn split_words(mut code: &[u8]) -> Vec<Word> {
    fn is_ch_usable(ch: u8) -> bool {
        ch.is_ascii_alphanumeric() || is_quote(ch) 
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
                words.push((pos - word.len(), word));
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
            while !code.is_empty() && code[0].is_ascii_alphanumeric() {
                word.push(code[0] as char);
                pos += 1;
                code = &code[1..];
            }
        }
        words.push((pos - word.len(), word));
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

pub trait GetColor {
    fn color(&self) -> Color;
}

impl GetColor for Word {
    fn color(&self) -> Color {
        if is_keyword(&self.1) {
            Color::Yellow
        } else if is_type(&self.1) {
            Color::Green
        } else if is_string(&self.1) {
            Color::DarkGreen
        } else if is_comment(self.1.as_bytes()) {
            Color::Grey
        } else {
            Color::White
        }
    }
}
