use crate::Position;
use std::io::{self, stdout, Write};
use termion::event::Event;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};

/// A size represented by a width and height.
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// The terminal that is used by the editor.
pub struct Terminal {
    size: Size,
    _stdout: MouseTerminal<RawTerminal<std::io::Stdout>>,
}

impl Terminal {
    /// Constructs the default Terminal.
    /// 
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

    /// Gets the size of the Terminal.
    pub fn size(&self) -> &Size {
        &self.size
    }

    /// Clears the terminal screen.
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    /// Clears the current line in the terminal.
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    /// Sets (inverts) the terminal foreground color.
    pub fn set_fg_color() {
        print!("{}", termion::style::Invert);
    }

    /// Resets the terminal foreground color.
    pub fn reset_fg_color() {
        print!("{}", termion::style::Reset);
    }

    /// Sets (inverts) the terminal background color.
    pub fn set_bg_color() {
        print!("{}", termion::style::Invert);
    }

    /// Reset the terminal background color.
    pub fn reset_bg_color() {
        print!("{}", termion::style::Reset);
    }

    /// Sets the cursor position on the terminal screen.
    /// 
    /// # Arguments
    /// 
    /// * `position` - the cursor position
    pub fn cursor_position(position: &Position) {
        let Position { x, y } = position;
        let x = x.saturating_add(1) as u16;
        let y = y.saturating_add(1) as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }

    /// Hides the cursor.
    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }

    /// Shows the cursor.
    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }

    /// Flushes stdout.
    /// 
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while flushing stdout
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    /// Listens for an event from stdin.
    /// 
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while reading event
    pub fn read_event() -> Result<Event, std::io::Error> {
        loop {
            if let Some(event) = io::stdin().lock().events().next() {
                return event;
            }
        }
    }
}
