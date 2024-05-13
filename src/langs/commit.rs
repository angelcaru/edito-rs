use crate::*;

pub struct Commit;

impl Language for Commit {
    fn split_words(&self, code: &str) -> Vec<Word> {
        if code.chars().next().filter(|&ch| ch == '#').is_some() {
            return vec![Word {
                col: 0,
                text: code.into(),
                color: rgb_color(100, 100, 100),
                attr: Attribute::Italic,
            }];
        }
        vec![Word {
            col: 0,
            text: code.into(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }

    fn should_indent(&self, _line: &str) -> bool {
        false
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
