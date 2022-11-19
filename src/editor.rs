use crate::commands::{copy::CopyCommand, paste::PasteCommand, Command};
use crate::Document;
use crate::Row;
use crate::Terminal;

use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::event::{Event, Key, MouseEvent};
use unicode_segmentation::UnicodeSegmentation;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 2;

// Key mappings for navigation
const POS_UP: Key = Key::Up;
const POS_DOWN: Key = Key::Down;
const POS_LEFT: Key = Key::Left;
const POS_RIGHT: Key = Key::Right;
const WORD_LEFT: Key = Key::Alt('q');
const WORD_RIGHT: Key = Key::Alt('w');
const LINE_LEFT: Key = Key::Alt('b');
const LINE_RIGHT: Key = Key::Alt('f');
const PAGE_UP: Key = Key::Alt('t');
const PAGE_DOWN: Key = Key::Alt('g');
const DOC_UP: Key = Key::Home;
const DOC_DOWN: Key = Key::End;

// Key mappings for control
const QUIT: Key = Key::Ctrl('q');
const SAVE: Key = Key::Ctrl('s');
const SEARCH: Key = Key::Ctrl('l');
const SELECT_FORWARD: Key = Key::Ctrl('f');
const SELECT_BACKWARD: Key = Key::Ctrl('b');
const DELETE_SELECTIONS: Key = Key::Ctrl('d');
const REPLACE_SELECTIONS: Key = Key::Ctrl('r');
const START_SELECT: Key = Key::Ctrl('t');
const END_SELECT: Key = Key::Ctrl('y');
const COPY: Key = Key::Ctrl('c');
const PASTE: Key = Key::Ctrl('p');

fn die(e: &std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}

/// The direction in which a search query should be handled.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// A position represented by (x, y) coordinates.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// A status message printed at the bottom of the editor.
struct StatusMessage {
    text: String,
    time: Instant,
}

struct Selection {
    start: Position,
    end: Position,
}

impl StatusMessage {
    /// Constructs a [StatusMessage] from a string
    ///
    /// # Arguments
    ///
    /// * `message` - the status message's string content
    fn from(message: String) -> Self {
        Self {
            text: message,
            time: Instant::now(),
        }
    }
}

/// The editor!
pub struct Editor {
    /// Whether the editor application should quit
    should_quit: bool,
    /// The [Terminal] that is used
    terminal: Terminal,
    /// The current position of the cursor
    cursor_position: Position,
    /// THe offset of the visible page
    offset: Position,
    /// The maximal horizontal position that is used when the user navigates up or down
    max_position: Option<usize>,
    /// The document being edited
    document: Document,
    /// The status message to be displayed
    status_message: StatusMessage,
    /// How many more times the quit command needs to be inputted before the application exits
    quit_times: u8,
    /// The word to be highlighted, if any
    highlighted_word: Option<String>,
    /// Current selection, if any
    selection: Option<Selection>,
    /// Clipboard contents, if any
    clipboard: Option<String>,
}

impl Editor {
    /// Constructs the default editor.
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-L = look for | Ctrl-S = save | Ctrl-Q = quit");

        let document = if let Some(filename) = args.get(1) {
            if let Ok(doc) = Document::open(filename) {
                doc
            } else {
                initial_status = format!("ERR: Could not open file {}", filename);
                Document::default()
            }
        } else {
            Document::default()
        };

        Editor {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            offset: Position::default(),
            cursor_position: Position::default(),
            max_position: None,
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
            selection: None,
            clipboard: None,
        }
    }

    /// Runs the editor.
    ///
    /// This is essentially an event loop and should only ever be called once.
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(&error);
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_event() {
                die(&error);
            }
        }
    }

    /// Re-renders the terminal screen.
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.document.highlight(
                &self.highlighted_word,
                Some(
                    self.offset
                        .y
                        .saturating_add(self.terminal.size().height as usize),
                ),
            );

            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    /// Draws the status bar at the bottom of the editor.
    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        let mut filename = "[No Name]".to_string();
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        if let Some(name) = &self.document.filename {
            filename = name.clone();
            filename.truncate(20);
        }

        let mut status = format!(
            "{} - {} lines{}",
            filename,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color();
        Terminal::set_fg_color();
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    /// Draws the message bar at the bottom of the editor.
    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    /// Draws a given row on the terminal screen.
    ///
    /// # Arguments
    ///
    /// * `row` - The row to be drawn
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    /// Draws a welcome message in the middle of the editor.
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Ferro editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    /// Saves the document being edited.
    fn save(&mut self) {
        if self.document.filename.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }

            self.document.filename = new_name;
        }
        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    /// Searches for a string in the document.
    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;

        let query = self
            .prompt(
                "Search (ESC to cancel, arrows to navigate, Ctrl+F/Ctrl+B to select): ",
                |editor, key, query| {
                    let mut moved = false;
                    match key {
                        SELECT_FORWARD => {
                            editor
                                .document
                                .add_selection(editor.cursor_position, query.len());
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        POS_RIGHT | POS_DOWN => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        SELECT_BACKWARD => {
                            editor
                                .document
                                .add_selection(editor.cursor_position, query.len());
                            direction = SearchDirection::Backward;
                        }
                        POS_LEFT | POS_UP => direction = SearchDirection::Backward,
                        _ => (),
                    }

                    if let Some(position) =
                        editor
                            .document
                            .find(&query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.move_cursor(Key::Left);
                    }
                    editor.highlighted_word = Some(query.to_string());
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
        self.document.refresh_highlighting();
    }

    /// Processes an event (i.e. a keypress or a mousepress).
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while reading event
    fn process_event(&mut self) -> Result<(), std::io::Error> {
        let event = Terminal::read_event()?;
        match event {
            Event::Key(keypress) => self.process_keypress(keypress),
            Event::Mouse(mousepress) => self.process_mousepress(mousepress),
            _ => Ok(()),
        }
    }

    /// Processes a keypress event.
    ///
    /// # Arguments
    ///
    /// * `keypress` - the [Key] that was pressed
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered
    fn process_keypress(&mut self, keypress: Key) -> Result<(), std::io::Error> {
        match keypress {
            QUIT => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more time(s) to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }

                self.should_quit = true;
            }
            COPY => CopyCommand::execute(self),
            PASTE => PasteCommand::execute(self),
            SAVE => self.save(),
            SEARCH => self.search(),
            START_SELECT => {
                self.selection = Some(Selection {
                    start: self.cursor_position,
                    end: self.cursor_position,
                });
            }
            END_SELECT => {
                if let Some(Selection { start, end: _ }) = self.selection {
                    self.selection = Some(Selection {
                        start,
                        end: self.cursor_position,
                    });
                }
            }
            Key::Char(c) => {
                let indent = self.document.insert(&mut self.cursor_position, c);
                (0..indent + 1).for_each(|_| self.move_cursor(Key::Right));
            }
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            POS_UP | POS_DOWN | POS_LEFT | POS_RIGHT | WORD_LEFT | WORD_RIGHT | LINE_LEFT
            | LINE_RIGHT | PAGE_UP | PAGE_DOWN | DOC_UP | DOC_DOWN => self.move_cursor(keypress),
            _ => (),
        }

        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    /// Processes a mousepress event.
    ///
    /// # Arguments
    ///
    /// * `mousepress` - the [MouseEvent] that occurred
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered
    fn process_mousepress(&mut self, mousepress: MouseEvent) -> Result<(), std::io::Error> {
        let offset = &self.offset;
        match mousepress {
            MouseEvent::Press(_, a, b) | MouseEvent::Release(a, b) | MouseEvent::Hold(a, b) => {
                let y = offset.y + b.saturating_sub(1) as usize;
                if let Some(row) = self.document.row(y) {
                    let x = (offset.x + a.saturating_sub(1) as usize).min(row.len());
                    self.cursor_position = Position { x, y };
                    self.max_position = Some(x);
                }
            }
        };
        Ok(())
    }

    /// Prompts the user for input.
    ///
    /// # Arguments
    ///
    /// * `prompt` - the prompt to print
    /// * `callback` - the callback to use
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered
    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, std::io::Error>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let event = Terminal::read_event()?;
            if let Event::Key(key) = event {
                match key {
                    Key::Backspace => {
                        let graphemes_cnt = result.graphemes(true).count();
                        result = result
                            .graphemes(true)
                            .take(graphemes_cnt.saturating_sub(1))
                            .collect();
                    }
                    Key::Char('\n') => {
                        self.document.reset_selections();
                        break;
                    }
                    Key::Char(c) => {
                        if !c.is_control() {
                            result.push(c);
                        }
                    }
                    DELETE_SELECTIONS => {
                        self.document.delete_selections();
                        break;
                    }
                    REPLACE_SELECTIONS => {
                        let replacement = self.prompt_replacement()?;
                        if replacement.is_some() {
                            self.document.replace_selections(&replacement);
                        } else {
                            self.document.reset_selections();
                        }
                        break;
                    }
                    Key::Esc => {
                        self.document.reset_selections();
                        result.truncate(0);
                        break;
                    }
                    _ => (),
                }
                callback(self, key, &result);
            }
        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Prompts the user for a string to replace all selections with.
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered
    fn prompt_replacement(&mut self) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("Replace with: {}", result));
            self.refresh_screen()?;
            let event = Terminal::read_event()?;
            if let Event::Key(key) = event {
                match key {
                    Key::Backspace => {
                        let graphemes_cnt = result.graphemes(true).count();
                        result = result
                            .graphemes(true)
                            .take(graphemes_cnt.saturating_sub(1))
                            .collect();
                    }
                    Key::Char('\n') => break,
                    Key::Char(c) => {
                        if !c.is_control() {
                            result.push(c);
                        }
                    }
                    Key::Esc => {
                        result.truncate(0);
                        break;
                    }
                    _ => (),
                }
            }
        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    /// Scrolls the screen by the height of the terminal.
    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let height = self.terminal.size().height as usize;
        let width = self.terminal.size().width as usize;
        let mut offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    /// Copies the selection into the clipboard
    pub fn copy(&mut self) {
        if let Some(Selection { start, end }) = self.selection {
            self.clipboard = Some(self.document.get_contents(start, end));
        }
    }

    /// Pastes the clipboard contents
    pub fn paste(&mut self) {
        if let Some(content) = &self.clipboard {
            for c in content[..].chars().rev() {
                self.document.insert(&mut self.cursor_position, c);
            }
        }
    }

    /// Moves the cursor based on the key that was pressed.
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed
    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;

        let Position { mut x, mut y } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            POS_UP => y = y.saturating_sub(1),
            POS_DOWN => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            POS_LEFT => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            POS_RIGHT => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            WORD_LEFT => {
                if let Some(pos) = self
                    .document
                    .find_next_word(&self.cursor_position, SearchDirection::Backward)
                {
                    x = pos.x;
                    y = pos.y;
                }
            }
            WORD_RIGHT => {
                if let Some(pos) = self
                    .document
                    .find_next_word(&self.cursor_position, SearchDirection::Forward)
                {
                    x = pos.x;
                    y = pos.y;
                }
            }
            LINE_LEFT => x = 0,
            LINE_RIGHT => x = width,
            PAGE_UP => y = y.saturating_sub(terminal_height),
            PAGE_DOWN => y = y.saturating_add(terminal_height).min(height),
            DOC_UP => y = 0,
            DOC_DOWN => y = height,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        let is_vertical_control = |k: Key| match k {
            POS_UP | POS_DOWN | PAGE_UP | PAGE_DOWN | DOC_UP | DOC_DOWN => true,
            _ => false,
        };
        let is_horizontal_control = |k: Key| match k {
            POS_LEFT | POS_RIGHT | WORD_LEFT | WORD_RIGHT | LINE_LEFT | LINE_RIGHT => true,
            _ => false,
        };

        if !is_vertical_control(key) || self.max_position.is_none() {
            x = x.min(width);
        } else if let Some(pos) = self.max_position {
            x = x.max(pos).min(width);
        }

        self.cursor_position = Position { x, y };

        if is_horizontal_control(key) {
            // We need to update the cursor's max_position iff the keypress controls the cursor's x position
            self.max_position = Some(x);
        }
    }
}
