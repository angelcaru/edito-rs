use crossterm::{cursor, event::*, terminal, QueueableCommand};
use std::{io::Write, time::Duration};

trait WriteChar {
    fn write_ch(&mut self, ch: u8) -> Result<usize, std::io::Error>;
}

impl<T: Write> WriteChar for T {
    fn write_ch(&mut self, ch: u8) -> Result<usize, std::io::Error> {
        let buf = [ch];
        self.write(&buf)
    }
}

struct Editor {
    display: TerminalDisplay,
    buf: Vec<Vec<u8>>,
    cursor: (usize, usize),
}

impl Editor {
    fn new() -> Result<Self, std::io::Error> {
        let mut buf = Vec::new();
        buf.push(Vec::from(b"Hello, World!"));
        buf.push(Vec::from(b"Foo, Bar!"));
        buf.push(Vec::from(b"Test, Hello!"));
        Ok(Self {
            display: TerminalDisplay::new()?,
            buf,
            cursor: (0, 0),
        })
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        assert!(
            dx == 0 || dy == 0,
            "Cannot move cursor horizontally and vertically at the same time"
        );
        let (x, y) = &mut self.cursor;

        let (new_x, new_y) = (*x as isize + dx, *y as isize + dy);
        let allowed_x = 0..=self.buf[*y].len() as isize;
        let allowed_y = 0..self.buf.len() as isize;

        if allowed_x.contains(&new_x) {
            *x = new_x as usize;
        }
        if allowed_y.contains(&new_y) {
            *y = new_y as usize;
        }

        if *x > self.buf[*y].len() {
            *x = self.buf[*y].len()
        }
    }

    fn handle_event(&mut self, e: Event) -> Result<bool, std::io::Error> {
        match e {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => return Ok(true),
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            }) => {
                let (x, y) = &mut self.cursor;
                assert!(*y < self.buf.len());

                // TODO: proper Unicode support
                self.buf[*y].insert(*x, ch as u8);

                *x += 1;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let (x, y) = &mut self.cursor;
                assert!(*y < self.buf.len());

                if *x != 0 {
                    self.buf[*y].remove(*x - 1);
                    *x -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let (x, y) = &mut self.cursor;
                assert!(*y < self.buf.len());

                if *x != self.buf[*y].len() {
                    self.buf[*y].remove(*x);
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            }) => self.move_cursor(-1, 0),
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            }) => self.move_cursor(1, 0),
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            }) => self.move_cursor(0, -1),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            }) => self.move_cursor(0, 1),
            _ => {}
        }
        Ok(false)
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
        self.display.clear();

        for y in 0..self.display.h as usize {
            for x in 0..self.display.w as usize {
                self.display
                    .write(x, y, *get2d(&self.buf, y, x).unwrap_or(&b' '));
            }
        }

        self.display.render()?;

        let (x, y) = self.cursor;
        self.display
            .stdout
            .queue(cursor::MoveTo(x as u16, y as u16))?;
        self.display.stdout.flush()?;

        Ok(())
    }
}

fn get2d<T>(v: &Vec<Vec<T>>, i: usize, j: usize) -> Option<&T> {
    v.get(i)?.get(j)
}

struct TerminalDisplay {
    stdout: std::io::Stdout,
    prev_chars: Option<Vec<Vec<u8>>>,
    chars: Vec<Vec<u8>>,
    w: u16,
    h: u16,
}

impl TerminalDisplay {
    fn new() -> Result<Self, std::io::Error> {
        let (w, h) = terminal::size()?;
        Ok(Self {
            stdout: std::io::stdout(),
            prev_chars: None,
            chars: Self::init_chars(w, h),
            w,
            h,
        })
    }

    fn init_chars(w: u16, h: u16) -> Vec<Vec<u8>> {
        let mut chars = Vec::with_capacity(h.into());
        for _ in 0..h {
            let mut row = Vec::with_capacity(w.into());
            for _ in 0..w {
                row.push(b' ');
            }
            chars.push(row);
        }
        chars
    }

    fn write(&mut self, x: usize, y: usize, ch: u8) {
        self.chars[y][x] = ch;
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
        //self.stdout.queue(cursor::MoveTo(0, 0))?;
        for (y, row) in self.chars.iter().enumerate() {
            if let Some(prev_chars) = &self.prev_chars {
                for (x, ch) in row.iter().enumerate() {
                    if &prev_chars[y][x] != ch {
                        self.stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                        self.stdout.write_ch(*ch)?;
                    }
                }
            } else {
                self.stdout.queue(cursor::MoveTo(0, y as u16))?;
                self.stdout.write(row)?;
            }
        }
        self.stdout.flush()?;

        self.prev_chars = Some(self.chars.clone());
        self.chars = Self::init_chars(self.w, self.h);

        Ok(())
    }

    fn clear(&mut self) {
        for row in self.chars.iter_mut() {
            row.fill(0);
        }
    }

    fn queue_clear(&mut self) -> Result<(), std::io::Error> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;
        self.stdout.queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }
}

fn main() -> Result<(), std::io::Error> {
    let polling_rate = Duration::from_secs_f64(0.01);
    let mut editor = Editor::new()?;

    terminal::enable_raw_mode()?;
    editor.display.queue_clear()?;

    loop {
        if poll(polling_rate)? {
            if editor.handle_event(read()?)? {
                break;
            }
        }
        editor.render()?;
    }
    terminal::disable_raw_mode()?;
    Ok(())
}
