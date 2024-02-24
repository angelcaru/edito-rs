use crossterm::{
    cursor,
    event::*,
    style::{self, Color},
    terminal, QueueableCommand,
};
use std::{
    io::{Read, Write},
    time::Duration,
};

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
    file_path: Option<String>,
    camera_topleft: (usize, usize),
    w: u16,
    h: u16,
    status: String,
}

const UI_HEIGHT: u16 = 2;

impl Editor {
    fn new() -> Result<Self, std::io::Error> {
        let mut buf = Vec::new();
        buf.push(Vec::new());

        let display = TerminalDisplay::new()?;
        let (w, h) = (display.w, display.h - UI_HEIGHT);
        Ok(Self {
            display,
            buf,
            cursor: (0, 0),
            file_path: None,
            camera_topleft: (0, 0),
            w,
            h,
            status: String::new(),
        })
    }

    fn load_file(&mut self, file_path: String) -> Result<(), std::io::Error> {
        let f = std::fs::File::open(file_path.clone())?.bytes();

        self.buf = Vec::new();
        let mut row = Vec::new();
        for ch in f {
            let ch = ch?;
            if ch == b'\n' {
                self.buf.push(row);
                row = Vec::new();
            } else {
                row.push(ch);
            }
        }
        self.buf.push(row);

        self.file_path = Some(file_path.clone());

        self.status = format!(
            "Successfully loaded file {}",
            file_path
        );

        Ok(())
    }

    fn save_file(&mut self) -> Result<(), std::io::Error> {
        if self.file_path.is_none() {
            todo!("Some sort of dialogue to allow the user to choose where to save the file");
        }
        let mut f = std::fs::File::create(self.file_path.clone().unwrap())?;

        for row in &self.buf {
            f.write_all(row)?;
            // TODO: support different line endings
            f.write_ch(b'\n')?;
        }

        self.status = format!(
            "Successfully saved file to {}",
            self.file_path.clone().unwrap()
        );
        Ok(())
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

        let (cx, cy) = &mut self.camera_topleft;
        while *y < *cy {
            *cy -= 1;
        }
        while *y >= *cy + self.h as usize {
            *cy += 1;
        }

        while *x < *cx {
            *cx -= 1;
        }
        while *x >= *cx + self.w as usize {
            *cx += 1;
        }
    }

    fn handle_event(&mut self, e: Event) -> Result<bool, std::io::Error> {
        self.status = String::new();
        match e {
            Event::Resize(w, h) => {
                self.display.resize(w, h);
                self.w = w;
                self.h = h - UI_HEIGHT;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => return Ok(true),
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => self.save_file()?,
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
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let (x, y) = &mut self.cursor;

                let (pre, post) = self.buf[*y].split_at(*x);
                let (pre, post) = (Vec::from(pre), Vec::from(post));
                self.buf[*y] = post;
                self.buf.insert(*y, pre);

                *y += 1;
                *x = 0;
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
                } else if *y != 0 {
                    let post = self.buf[*y].clone();
                    let pre = &mut self.buf[*y - 1];

                    *x = pre.len();

                    pre.extend(post);
                    self.buf.remove(*y);

                    *y -= 1;
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
                } else if *y != self.buf.len() - 1 {
                    let post = self.buf[*y + 1].clone();
                    let pre = &mut self.buf[*y];

                    pre.extend(post);
                    self.buf.remove(*y + 1);
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

        self.render_buf();
        self.render_file_path();
        self.render_status_bar();

        self.display.render()?;

        let (cx, cy) = self.camera_topleft;

        let (x, y) = self.cursor;
        let (x, y) = (x - cx, y - cy);
        self.display
            .stdout
            .queue(cursor::MoveTo(x as u16, y as u16))?;
        self.display.stdout.flush()?;

        Ok(())
    }

    fn render_buf(&mut self) {
        let (cx, cy) = self.camera_topleft;

        for y in 0..self.h as usize {
            for x in 0..self.w as usize {
                let ch = *get2d(&self.buf, y + cy, x + cx).unwrap_or(&b' ');
                let cell = Cell {
                    ch,
                    fg: Color::Reset,
                    bg: Color::Reset,
                };
                self.display.write(x, y, cell);
            }
        }
    }

    fn render_file_path(&mut self) {
        let y = self.display.h as usize - 2;

        let file_path = self
            .file_path
            .clone()
            .unwrap_or("<temporary buffer>".into());
        for x in 0..self.w as usize {
            let cell = Cell {
                ch: *file_path.as_bytes().get(x).unwrap_or(&b' ') as u8,
                fg: Color::Black,
                bg: Color::White,
            };
            self.display.write(x, y, cell);
        }
    }

    fn render_status_bar(&mut self) {
        let y = self.display.h as usize - 1;

        for x in 0..self.w as usize {
            let cell = Cell {
                ch: *self.status.as_bytes().get(x).unwrap_or(&b' ') as u8,
                fg: Color::Reset,
                bg: Color::Reset,
            };
            self.display.write(x, y, cell);
        }
    }
}

fn get2d<T>(v: &Vec<Vec<T>>, i: usize, j: usize) -> Option<&T> {
    v.get(i)?.get(j)
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Cell {
    ch: u8,
    fg: Color,
    bg: Color,
}

impl Cell {
    fn empty() -> Self {
        Self {
            ch: b' ',
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }

    fn render<T: Write>(&self, q: &mut T) -> Result<(), std::io::Error> {
        q.queue(style::SetForegroundColor(self.fg))?;
        q.queue(style::SetBackgroundColor(self.bg))?;
        q.write_ch(self.ch)?;
        Ok(())
    }
}

struct TerminalDisplay {
    stdout: std::io::Stdout,
    prev_chars: Option<Vec<Vec<Cell>>>,
    chars: Vec<Vec<Cell>>,
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

    fn init_chars(w: u16, h: u16) -> Vec<Vec<Cell>> {
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

    fn resize(&mut self, w: u16, h: u16) {
        self.prev_chars = None;
        self.chars = Self::init_chars(w, h);

        self.w = w;
        self.h = h;
    }

    fn write(&mut self, x: usize, y: usize, ch: Cell) {
        self.chars[y][x] = ch;
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
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

    fn clear(&mut self) {
        for row in self.chars.iter_mut() {
            row.fill(Cell::empty());
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
    let mut args = std::env::args();
    let _ = args.next();

    let polling_rate = Duration::from_secs_f64(0.01);
    let mut editor = Editor::new()?;

    if let Some(file_path) = args.next() {
        editor.load_file(file_path)?;
    }

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


