use crossterm::style::Color;

pub type Word = (usize, String);

pub fn split_words(mut code: &[u8]) -> Vec<Word> {
    let mut words = Vec::new();

    let mut pos = 0;
    while !code.is_empty() {
        while !code.is_empty() && !code[0].is_ascii_alphanumeric() {
            pos += 1;
            code = &code[1..];
        }

        let mut word = String::new();
        while !code.is_empty() && code[0].is_ascii_alphanumeric() {
            word.push(code[0] as char);
            pos += 1;
            code = &code[1..];
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

pub trait GetColor {
    fn color(&self) -> Color;
}

impl GetColor for Word {
    fn color(&self) -> Color {
        if is_keyword(&self.1) {
            Color::Yellow
        } else if is_type(&self.1) {
            Color::Green
        } else {
            Color::White
        }
    }
}
