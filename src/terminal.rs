use crate::Position;
use std::io::{self, stdout, Write};
use termion::event::Event;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: MouseTerminal<RawTerminal<std::io::Stdout>>,
}

impl Terminal {
    /// # Errors
    ///
    /// Will return `Err` if unable to get terminal size
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: MouseTerminal::from(stdout().into_raw_mode()?),
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_fg_color() {
        print!("{}", termion::style::Invert);
    }

    pub fn reset_fg_color() {
        print!("{}", termion::style::Reset);
    }

    pub fn set_bg_color() {
        print!("{}", termion::style::Invert);
    }

    pub fn reset_bg_color() {
        print!("{}", termion::style::Reset);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position { x, y } = position;
        let x = x.saturating_add(1) as u16;
        let y = y.saturating_add(1) as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while flushing stdout
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while reading keypress
    pub fn read_event() -> Result<Event, std::io::Error> {
        loop {
            if let Some(event) = io::stdin().lock().events().next() {
                return event;
            }
        }
    }
}
