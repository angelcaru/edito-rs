use crate::*;

pub struct Commit;

impl Language for Commit {
    fn split_words(&self, code: &[u8]) -> Vec<Word> {
        if code.first().filter(|&&ch| ch == b'#').is_some() {
            return vec![Word {
                col: 0,
                text: String::from_utf8_lossy(code).to_string(),
                color: rgb_color(100, 100, 100),
                attr: Attribute::Italic,
            }];
        }
        vec![Word {
            col: 0,
            text: String::from_utf8_lossy(code).to_string(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }

    fn should_indent(&self, _line: &[u8]) -> bool {
        false
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
