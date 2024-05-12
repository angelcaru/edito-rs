use crate::*;

pub struct Commit;

impl Language for Commit {
    fn split_words(&self, code: &[char]) -> Vec<Word> {
        if code.first().filter(|&&ch| ch == '#').is_some() {
            return vec![Word {
                col: 0,
                text: code.into_iter().collect(),
                color: rgb_color(100, 100, 100),
                attr: Attribute::Italic,
            }];
        }
        vec![Word {
            col: 0,
            text: code.into_iter().collect(),
            color: Color::White,
            attr: Attribute::Reset,
        }]
    }

    fn should_indent(&self, _line: &[char]) -> bool {
        false
    }

    fn should_dedent(&self, _ch: char) -> bool {
        false
    }
}
