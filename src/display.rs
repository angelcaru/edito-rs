use crate::WriteChar;
use crossterm::{
    cursor,
    style::{self, Attribute, Color},
    terminal, QueueableCommand,
};
use std::io::Write;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Cell {
    pub ch: u8,
    pub fg: Color,
    pub bg: Color,
    pub attr: Attribute,
}

impl Cell {
    pub fn empty() -> Self {
        Self {
            ch: b' ',
            fg: Color::White,
            bg: Color::Black,
            attr: Attribute::Reset,
        }
    }

    pub fn render<T: Write>(&self, q: &mut T) -> Result<(), std::io::Error> {
        q.queue(style::SetAttribute(self.attr))?;
        q.queue(style::SetForegroundColor(self.fg))?;
        q.queue(style::SetBackgroundColor(self.bg))?;
        q.write_ch(self.ch)?;
        Ok(())
    }
}

pub struct TerminalDisplay {
    pub stdout: std::io::Stdout,
    pub prev_chars: Option<Vec<Vec<Cell>>>,
    pub chars: Vec<Vec<Cell>>,
    pub w: u16,
    pub h: u16,
}

impl TerminalDisplay {
    pub fn new() -> Result<Self, std::io::Error> {
        let (w, h) = terminal::size()?;
        Ok(Self {
            stdout: std::io::stdout(),
            prev_chars: None,
            chars: Self::init_chars(w, h),
            w,
            h,
        })
    }

    pub fn init_chars(w: u16, h: u16) -> Vec<Vec<Cell>> {
        let mut chars = Vec::with_capacity(h.into());
        for _ in 0..h {
            let mut row = Vec::with_capacity(w.into());
            for _ in 0..w {
                row.push(Cell::empty());
            }
            chars.push(row);
        }
        chars
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.prev_chars = None;
        self.chars = Self::init_chars(w, h);

        self.w = w;
        self.h = h;
    }

    pub fn write(&mut self, x: usize, y: usize, ch: Cell) {
        self.chars[y][x] = ch;
    }

    pub fn render(&mut self) -> Result<(), std::io::Error> {
        //self.stdout.queue(cursor::MoveTo(0, 0))?;
        for (y, row) in self.chars.iter().enumerate() {
            if let Some(prev_chars) = &self.prev_chars {
                for (x, cell) in row.iter().enumerate() {
                    if &prev_chars[y][x] != cell {
                        self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                        cell.render(&mut self.stdout)?;
                    }
                }
            } else {
                self.stdout.queue(cursor::MoveTo(0, y as u16))?;
                for cell in row {
                    cell.render(&mut self.stdout)?;
                }
            }
        }
        self.stdout.flush()?;

        self.prev_chars = Some(self.chars.clone());
        self.chars = Self::init_chars(self.w, self.h);

        Ok(())
    }

    pub fn clear(&mut self) {
        for row in self.chars.iter_mut() {
            row.fill(Cell::empty());
        }
    }

    pub fn queue_clear(&mut self) -> Result<(), std::io::Error> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }
}
