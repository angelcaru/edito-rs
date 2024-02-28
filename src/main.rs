mod display;
mod lexer;

use crossterm::{
    cursor::{self, MoveTo},
    event::*,
    style::{self, Attribute, Color, Colors},
    terminal::{self, Clear},
    ExecutableCommand, QueueableCommand,
};
use std::{
    cmp::Ordering,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpListener},
    process::exit,
    str::FromStr,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use display::*;
use lexer::*;

pub trait WriteChar {
    fn write_ch(&mut self, ch: u8) -> Result<usize, std::io::Error>;
}

impl<T: Write> WriteChar for T {
    fn write_ch(&mut self, ch: u8) -> Result<usize, std::io::Error> {
        let buf = [ch];
        self.write(&buf)
    }
}

type Pos = (usize, usize);

struct Cursor {
    selection_start: Option<Pos>,
    pos: Pos,
    state: CursorState,
}

impl Cursor {
    fn minmax_pos(a: Pos, b: Pos) -> (Pos, Pos) {
        let (ax, ay) = a;
        let (bx, by) = b;

        match ay.cmp(&by) {
            Ordering::Less => (a, b),
            Ordering::Equal => {
                if ax < bx {
                    (a, b)
                } else {
                    (b, a)
                }
            }
            Ordering::Greater => (b, a),
        }
    }
}

enum CursorState {
    Default,
    StatusBar,
}

#[derive(Clone, Copy)]
enum PromptType {
    FileSave,
    QuitOnNoSave,
    Command,
}

struct Editor {
    display: TerminalDisplay,
    buf: Vec<Vec<u8>>,
    cursor: Cursor,
    file_path: Option<String>,
    camera_topleft: Pos,
    w: u16,
    h: u16,
    status: Vec<u8>,
    status_prompt: String,
    prompt_type: Option<PromptType>,
    logger: Option<Sender<String>>,
    unsaved_changes: bool,
}

const UI_WIDTH: u16 = 4;
const UI_HEIGHT: u16 = 2;

impl Editor {
    fn new() -> Result<Self, std::io::Error> {
        let buf = vec![Vec::new()];

        let display = TerminalDisplay::new()?;
        let (w, h) = (display.w - UI_WIDTH, display.h - UI_HEIGHT);
        Ok(Self {
            display,
            buf,
            cursor: Cursor {
                selection_start: None,
                pos: (0, 0),
                state: CursorState::Default,
            },
            file_path: None,
            camera_topleft: (0, 0),
            w,
            h,
            status: Vec::new(),
            status_prompt: String::new(),
            logger: None,
            prompt_type: None,
            unsaved_changes: true,
        })
    }

    fn enable_logging(&mut self, port: u16) -> std::io::Result<()> {
        self.logger = Some(logger(port)?);
        self.set_status(format!("Successfully enabled logging on port {port}"));
        Ok(())
    }

    fn log(&mut self, msg: String) {
        if let Some(logger) = &self.logger {
            let _ = logger.send(msg); // ignore errors when logging since they aren't that important
        }
    }

    fn set_status(&mut self, status: String) {
        self.log(format!("[STATUS] {status}"));
        self.status = status.into();
    }

    // TODO: introduce a better interface for this stuff
    // I tried to do it but the borrow checker hated me for it
    fn set_status_prompt(&mut self, prompt: String, prompt_type: PromptType) {
        self.cursor.state = CursorState::StatusBar;
        self.cursor.pos.0 = 0;

        self.status_prompt = prompt;
        self.prompt_type = Some(prompt_type);
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

        self.set_status(format!("Successfully loaded file {}", file_path));
        self.unsaved_changes = false;

        Ok(())
    }

    fn save_file(&mut self) -> Result<(), std::io::Error> {
        if self.file_path.is_none() {
            self.set_status_prompt("File path: ".into(), PromptType::FileSave);
            return Ok(());
        }
        let mut f = std::fs::File::create(self.file_path.clone().unwrap())?;

        for row in &self.buf {
            f.write_all(row)?;
            // TODO: support different line endings
            f.write_ch(b'\n')?;
        }

        self.set_status(format!(
            "Successfully saved file to {}",
            self.file_path.clone().unwrap()
        ));
        self.unsaved_changes = false;

        Ok(())
    }

    fn row(&mut self) -> &mut Vec<u8> {
        match self.cursor.state {
            CursorState::Default => &mut self.buf[self.cursor.pos.1],
            CursorState::StatusBar => &mut self.status,
        }
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        assert!(
            dx == 0 || dy == 0,
            "Cannot move cursor horizontally and vertically at the same time"
        );
        let (new_x, new_y) = (
            self.cursor.pos.0 as isize + dx,
            self.cursor.pos.1 as isize + dy,
        );
        let allowed_x = 0..=self.row().len() as isize;
        let allowed_y = 0..self.buf.len() as isize;

        if allowed_x.contains(&new_x) {
            self.cursor.pos.0 = new_x as usize;
        }
        if allowed_y.contains(&new_y) {
            self.cursor.pos.1 = new_y as usize;
        }

        if self.cursor.pos.0 > self.row().len() {
            self.cursor.pos.0 = self.row().len()
        }

        let (cx, cy) = &mut self.camera_topleft;
        while self.cursor.pos.1 < *cy {
            *cy -= 1;
        }
        while self.cursor.pos.1 >= *cy + self.h as usize {
            *cy += 1;
        }

        while self.cursor.pos.0 < *cx {
            *cx -= 1;
        }
        while self.cursor.pos.0 >= *cx + self.w as usize {
            *cx += 1;
        }
    }

    fn process_command(&mut self, cmd: &str) -> String {
        let cmd: Vec<_> = cmd.split_whitespace().collect();
        if cmd.is_empty() {
            return "ERROR: empty command".into();
        }

        match cmd[0] {
            "quit" => quit(),
            "load" => {
                if cmd.len() != 2 {
                    return "ERROR: the \"load\" command expects exactly one argument (without spaces)".into();
                }

                if let Err(err) = self.load_file(cmd[1].into()) {
                    format!("ERROR: {err}")
                } else {
                    std::str::from_utf8(&self.status)
                        .expect("that we didn't put garbage into the status")
                        .into()
                }
            }
            x => format!("ERROR: unknown command: {x:?}"),
        }
    }

    fn handle_status_prompt(&mut self) -> Result<bool, std::io::Error> {
        let response = self.status.clone();
        let response = std::str::from_utf8(&response).unwrap();
        self.status.clear();
        match self
            .prompt_type
            .expect("we never call this when the prompt is empty")
        {
            PromptType::FileSave => {
                self.file_path = Some(response.into());
                self.save_file()?;
            }
            PromptType::QuitOnNoSave => {
                match response {
                    "y" | "Y" => {
                        self.save_file()?;
                        quit();
                    }
                    "n" | "N" => quit(),
                    //"c" | "C" => todo!(),
                    _ => self.set_status(format!(
                        "The answer must be one of 'y' or 'n', not {response:?}"
                    )),
                }
            }
            PromptType::Command => {
                let new_status = self.process_command(response);
                self.set_status(new_status);
            }
        }

        self.cursor.state = CursorState::Default;
        self.status_prompt = String::new();

        if self.cursor.pos.0 > self.row().len() {
            self.cursor.pos.0 = self.row().len();
        }

        Ok(false)
    }

    fn update_selection(&mut self, modifiers: KeyModifiers) {
        if self.cursor.selection_start.is_none() {
            self.cursor.selection_start = Some(self.cursor.pos);
        }

        if modifiers == KeyModifiers::NONE {
            self.cursor.selection_start = None;
        }
    }

    fn add_char(&mut self, ch: u8) {
        assert!(self.cursor.pos.1 < self.buf.len());

        if let Some(sel) = self.cursor.selection_start {
            let ((sx, sy), (cx, cy)) = Cursor::minmax_pos(sel, self.cursor.pos);

            if sy != cy {
                let post = Vec::from(&self.buf[cy][cx..]);
                let pre = &mut self.buf[sy];

                pre.resize(sx, b' ');
                pre.push(ch);
                pre.extend(post);

                for i in (sy + 1..=cy).rev() {
                    self.buf.remove(i);
                }

                self.cursor.selection_start = None;
                self.cursor.pos = (sx, sy);
            } else {
                let row = self.row();

                for i in (sx..=cx).rev() {
                    row.remove(i);
                }

                row.insert(sx, ch);

                self.cursor.selection_start = None;
                self.cursor.pos.0 = sx;
            }
        } else {
            // TODO: proper Unicode support
            let x = self.cursor.pos.0;
            let row = self.row();
            row.insert(x, ch);
        }

        self.cursor.pos.0 += 1;
    }

    fn backspace(&mut self) {
        if self.cursor.pos.0 != 0 {
            let x = self.cursor.pos.0;
            let row = self.row();
            row.remove(x - 1);
            self.cursor.pos.0 -= 1;
        } else if self.cursor.pos.1 != 0 {
            if let CursorState::StatusBar = self.cursor.state {
                return;
            }
            let post = self.buf[self.cursor.pos.1].clone();
            let pre = &mut self.buf[self.cursor.pos.1 - 1];

            self.cursor.pos.0 = pre.len();

            pre.extend(post);
            self.buf.remove(self.cursor.pos.1);

            self.cursor.pos.1 -= 1;
        }
    }

    fn handle_event(&mut self, e: Event) -> Result<bool, std::io::Error> {
        if let CursorState::Default = self.cursor.state {
            self.status = Vec::new();
        }
        match e {
            Event::Resize(w, h) => {
                self.display.resize(w, h);
                self.w = w - UI_WIDTH;
                self.h = h - UI_HEIGHT;
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                if self.unsaved_changes {
                    self.set_status_prompt(
                        "You have unsaved changes. Save now? (y/n) ".into(),
                        PromptType::QuitOnNoSave,
                    );
                } else {
                    quit();
                }
            }
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
                self.unsaved_changes = true;
                self.add_char(ch as u8);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if let CursorState::StatusBar = self.cursor.state {
                    return self.handle_status_prompt();
                }

                self.unsaved_changes = true;
                if self.cursor.selection_start.is_some() {
                    self.set_status("TODO: handle text selection for the `Enter` key".into());
                } else {
                    let (x, y) = &mut self.cursor.pos;

                    let (pre, post) = self.buf[*y].split_at(*x);
                    let (pre, post) = (Vec::from(pre), Vec::from(post));
                    self.buf[*y] = post;
                    self.buf.insert(*y, pre);

                    *y += 1;
                    *x = 0;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                assert!(self.cursor.pos.1 < self.buf.len());

                if self.cursor.selection_start.is_some() {
                    self.add_char(0);
                }
                self.backspace();
                self.unsaved_changes = true;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                assert!(self.cursor.pos.1 < self.buf.len());

                if self.cursor.selection_start.is_some() {
                    self.add_char(0);
                    self.backspace();
                } else if self.cursor.selection_start.is_none()
                    || self.cursor.selection_start.unwrap().1 == self.cursor.pos.1
                {
                    if self.cursor.pos.0 != self.row().len() {
                        let x = self.cursor.pos.0;
                        self.row().remove(x);
                    } else if self.cursor.pos.1 != self.buf.len() - 1 {
                        if let CursorState::StatusBar = self.cursor.state {
                            return Ok(false);
                        }
                        let post = self.buf[self.cursor.pos.1 + 1].clone();
                        let pre = &mut self.buf[self.cursor.pos.1];

                        pre.extend(post);
                        self.buf.remove(self.cursor.pos.1 + 1);
                    }
                }
                self.unsaved_changes = true;
            }

            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.move_cursor(-1, 0)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.move_cursor(1, 0)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.move_cursor(0, -1)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.move_cursor(0, 1)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Home,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.cursor.pos.0 = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::End,
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.update_selection(modifiers);
                self.cursor.pos.0 = self.row().len();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.set_status_prompt("Command: ".into(), PromptType::Command);
            }
            _ => {}
        }
        Ok(false)
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
        use crossterm::cursor::SetCursorStyle;
        self.display.clear();

        self.display
            .stdout
            .queue(if self.cursor.selection_start.is_some() {
                SetCursorStyle::SteadyUnderScore
            } else {
                SetCursorStyle::BlinkingBlock
            })?;

        self.render_line_numbers();
        self.render_buf();
        self.render_file_path();
        self.render_status_bar();

        self.display.render()?;

        let (cx, cy) = self.camera_topleft;

        let (x, y) = self.cursor.pos;
        let (x, y) = if let CursorState::StatusBar = self.cursor.state {
            (x - cx + self.status_prompt.len(), (self.h + 1) as usize)
        } else {
            (x - cx + UI_WIDTH as usize, y - cy)
        };

        self.display
            .stdout
            .queue(cursor::MoveTo(x as u16, y as u16))?;
        self.display.stdout.flush()?;

        Ok(())
    }

    fn render_line_numbers(&mut self) {
        let (_, cy) = self.camera_topleft;

        for y in 0..self.h {
            let y = y as usize;
            let num = y + cy;

            let num_str = lpad((num + 1).to_string(), 3);

            for x in 0..(UI_WIDTH - 1) {
                let x = x as usize;
                self.display.write(
                    x,
                    y,
                    Cell {
                        ch: num_str.as_bytes().get(x).copied().unwrap_or(b' '),
                        fg: if num == self.cursor.pos.1 {
                            Color::White
                        } else {
                            Color::Grey
                        },
                        bg: Color::DarkGrey,
                        attr: if num == self.cursor.pos.1 {
                            Attribute::Bold
                        } else {
                            Attribute::Reset
                        },
                    },
                )
            }
        }
    }

    #[rustfmt::skip]
    fn selected(&self, x: usize, y: usize) -> bool {
        let Some((sx, sy)) = self.cursor.selection_start else {
            return false;
        };
        let (cx, cy) = self.cursor.pos;

        let ((sx, sy), (cx, cy)) =
          Cursor::minmax_pos((sx, sy), (cx, cy));

        // let (sx, cx) = (min(sx, cx), max(sx, cx));
        // let (sy, cy) = (min(sy, cy), max(sy, cy));

        if sy == cy {
            y == cy && (sx..=cx).contains(&x)
        } else {
            ((sy+1)..cy).contains(&y)
            || (y == sy && x >= sx)
            || (y == cy && x <= cx)
        }
    }

    fn render_buf(&mut self) {
        let (cx, cy) = self.camera_topleft;

        for y in 0..self.h as usize {
            let row_idx = y + cy;
            let mut words = split_words(&self.buf[row_idx]).into_iter().peekable();

            for x in 0..self.w {
                let x = (x + UI_WIDTH) as usize;
                let ch_idx = x + cx - UI_WIDTH as usize;
                let ch = *get2d(&self.buf, row_idx, ch_idx).unwrap_or(&b' ');

                let mut bg = Color::Black;
                let mut fg = if let Some(w) = words.peek() {
                    let w = w.clone();
                    let (pos, ref word) = w;
                    if ch_idx == (pos + word.len()) {
                        words.next();
                    }
                    if ch_idx >= pos && ch_idx < (pos + word.len()) {
                        w.color()
                    } else {
                        Color::White
                    }
                } else {
                    Color::White
                };

                if self.selected(x + cx - UI_WIDTH as usize, y + cy) {
                    (fg, bg) = (bg, fg);
                }

                let cell = Cell {
                    ch,
                    fg,
                    bg,
                    attr: Attribute::Reset,
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
        for x in 0..self.display.w as usize {
            let cell = Cell {
                ch: *file_path.as_bytes().get(x).unwrap_or(&b' '),
                fg: Color::Black,
                bg: Color::White,
                attr: Attribute::Reset,
            };
            self.display.write(x, y, cell);
        }
    }

    fn render_status_bar(&mut self) {
        let y = self.display.h as usize - 1;

        let status_iter = self
            .status_prompt
            .bytes()
            .chain(self.status.iter().copied())
            .chain(std::iter::repeat(b' '));
        for (x, ch) in (0..self.display.w as usize).zip(status_iter) {
            let cell = Cell {
                ch,
                fg: Color::White,
                bg: Color::Black,
                attr: Attribute::Reset,
            };
            self.display.write(x, y, cell);
        }
    }
}

fn lpad(mut s: String, n: usize) -> String {
    while s.len() < n {
        s.insert(0, ' ');
    }
    s
}

fn get2d<T>(v: &[Vec<T>], i: usize, j: usize) -> Option<&T> {
    v.get(i)?.get(j)
}

fn logger(port: u16) -> std::io::Result<Sender<String>> {
    let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), port);

    let listener = TcpListener::bind(addr)?;

    let (sender, reciever) = mpsc::channel::<String>();

    thread::spawn(move || {
        // TODO: handle errors inside logger thread
        let (mut stream, _addr) = listener.accept().unwrap();
        loop {
            let msg = reciever.recv().unwrap();
            let msg = msg.as_bytes();
            stream.write_all(msg).unwrap();
            stream.write_ch(b'\n').unwrap();
            stream.flush().unwrap();
        }
    });

    Ok(sender)
}

fn quit() -> ! {
    let _ = terminal::disable_raw_mode();
    let _ = std::io::stdout().execute(style::SetColors(Colors::new(Color::Reset, Color::Reset)));
    let _ = std::io::stdout().execute(Clear(terminal::ClearType::All));
    let _ = std::io::stdout().execute(MoveTo(0, 0));
    exit(0);
}

fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();
    let _ = args.next();

    let polling_rate = Duration::from_secs_f64(0.01);
    let mut editor = Editor::new()?;

    if let Some(file_path) = args.next() {
        editor.load_file(file_path)?;
    }

    editor.enable_logging(6969)?;

    terminal::enable_raw_mode()?;
    editor.display.queue_clear()?;

    loop {
        if poll(polling_rate)? {
            editor.handle_event(read()?)?;
        }
        editor.render()?;
    }
}
