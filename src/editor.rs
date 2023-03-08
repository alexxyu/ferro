use std::cell::RefCell;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use bounded_vec_deque::BoundedVecDeque;
use shunting::{MathContext, ShuntingParser};
use signal_hook::consts::SIGWINCH;
use termion::event::{Event, Key, MouseEvent};
use unicode_segmentation::UnicodeSegmentation;

use crate::commands::copy::CopyCommand;
use crate::commands::delete::DeleteCommand;
use crate::commands::group::{CommandGroup, CommandType};
use crate::commands::insert::InsertCommand;
use crate::commands::paste::PasteCommand;
use crate::commands::{BoxedCommand, Command};
use crate::Document;
use crate::Row;
use crate::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 2;
const HISTORY_LIMIT: usize = 10;

// Key mappings for navigation
const KEY_POS_UP: Key = Key::Up;
const KEY_POS_DOWN: Key = Key::Down;
const KEY_POS_LEFT: Key = Key::Left;
const KEY_POS_RIGHT: Key = Key::Right;
const KEY_WORD_LEFT: Key = Key::Alt('q');
const KEY_WORD_RIGHT: Key = Key::Alt('w');
const KEY_LINE_LEFT: Key = Key::Alt('b');
const KEY_LINE_RIGHT: Key = Key::Alt('f');
const KEY_PAGE_UP: Key = Key::Alt('t');
const KEY_PAGE_DOWN: Key = Key::Alt('g');
const KEY_DOC_UP: Key = Key::Home;
const KEY_DOC_DOWN: Key = Key::End;

// Key mappings for control
const KEY_QUIT: Key = Key::Ctrl('q');
const KEY_SAVE: Key = Key::Ctrl('s');
const KEY_SEARCH: Key = Key::Ctrl('l');
const KEY_SELECT_FORWARD: Key = Key::Ctrl('f');
const KEY_SELECT_BACKWARD: Key = Key::Ctrl('b');
const KEY_DELETE_SELECTIONS: Key = Key::Ctrl('d');
const KEY_REPLACE_SELECTIONS: Key = Key::Ctrl('r');
const KEY_START_SELECT: Key = Key::Ctrl('t');
const KEY_END_SELECT: Key = Key::Ctrl('y');
const KEY_COPY: Key = Key::Ctrl('c');
const KEY_PASTE: Key = Key::Ctrl('v');
const KEY_UNDO: Key = Key::Ctrl('u');

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
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.y == other.y {
            self.x.cmp(&other.x)
        } else {
            self.y.cmp(&other.y)
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::ops::Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// A status message printed at the bottom of the editor.
struct StatusMessage {
    text: String,
    time: Instant,
}

pub struct Selection {
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
    /// The offset of the visible page
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
    pub selection: Option<Selection>,
    /// Clipboard contents, if any
    pub clipboard: Option<String>,
    /// History of commands
    command_history: BoundedVecDeque<CommandGroup>,
    /// Flag for the SIGWINCH signal that is set when the terminal window is resized
    _sigwinch_flag: Arc<AtomicBool>,
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

        let flag = Arc::new(AtomicBool::new(false));
        let _ = signal_hook::flag::register(SIGWINCH, Arc::clone(&flag)).unwrap();

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
            command_history: BoundedVecDeque::new(HISTORY_LIMIT),
            _sigwinch_flag: flag,
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
                self.set_status_message("Save aborted.".to_string());
                return;
            }

            self.document.filename = new_name;
        }
        if self.document.save().is_ok() {
            self.set_status_message("File saved successfully.".to_string());
        } else {
            self.set_status_message("Error writing file!".to_string());
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
                        KEY_SELECT_FORWARD => {
                            editor
                                .document
                                .add_selection(editor.cursor_position, query.len());
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        KEY_POS_RIGHT | KEY_POS_DOWN => {
                            direction = SearchDirection::Forward;
                            editor.move_cursor(Key::Right);
                            moved = true;
                        }
                        KEY_SELECT_BACKWARD => {
                            editor
                                .document
                                .add_selection(editor.cursor_position, query.len());
                            direction = SearchDirection::Backward;
                        }
                        KEY_POS_LEFT | KEY_POS_UP => direction = SearchDirection::Backward,
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

    /// Prompts the user for a mathematical expression and displays its evaluated result.
    fn evaluate_expression(&mut self) {
        let query = self
            .prompt("Enter your expression: ", |_, _, _| {})
            .unwrap_or(None)
            .unwrap_or(String::new());

        if let Ok(expr) = ShuntingParser::parse_str(&query) {
            if let Ok(result) = MathContext::new().eval(&expr) {
                self.status_message = StatusMessage::from(format!("Result = {}", result));
                return;
            }
        }

        self.status_message = StatusMessage::from("Invalid expression.".into());
    }

    /// Processes an event (i.e. a keypress or a mousepress).
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while reading event
    fn process_event(&mut self) -> Result<(), std::io::Error> {
        let handle =
            thread::spawn(move || -> Result<Event, std::io::Error> { Terminal::read_event() });

        while !handle.is_finished() {
            if self._sigwinch_flag.swap(false, Ordering::Relaxed) {
                self.terminal.resize()?;
                self.refresh_screen()?;
            }
        }
        let event = handle.join().unwrap()?;

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
            KEY_QUIT => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.set_status_message(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more time(s) to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }

                self.should_quit = true;
            }
            KEY_COPY => {
                CopyCommand::new().execute(self);
            }
            KEY_PASTE => {
                let mut command = PasteCommand::new(self.cursor_position, self.clipboard.clone());
                command.execute(self);
                self.command_history.push_back(CommandGroup::from_command(
                    Box::new(RefCell::new(command)),
                    CommandType::PASTE,
                ));
            }
            KEY_UNDO => {
                if let Some(mut command) = self.command_history.pop_back() {
                    command.undo(self);
                }
            }
            KEY_SAVE => self.save(),
            KEY_SEARCH => self.search(),
            KEY_START_SELECT => {
                self.selection = Some(Selection {
                    start: self.cursor_position,
                    end: self.cursor_position,
                });
            }
            KEY_END_SELECT => {
                if let Some(Selection { start, end: _ }) = self.selection {
                    self.selection = Some(Selection {
                        start: start.min(self.cursor_position),
                        end: start.max(self.cursor_position),
                    });
                }
            }
            Key::Alt('c') => self.evaluate_expression(),
            Key::Char(c) => {
                let mut command = InsertCommand::new(self.cursor_position, c.to_string());
                command.execute(self);
                self.merge_or_add_command(Box::new(RefCell::new(command)), CommandType::INSERT);
            }
            Key::Delete => {
                let Position { x, y } = self.cursor_position;
                if y < self.document.len() - 1
                    || x < self.document.row(y).unwrap_or(&Row::default()).len()
                {
                    let mut command = DeleteCommand::new(
                        self.cursor_position,
                        self.document
                            .get_char_in_doc(self.cursor_position)
                            .unwrap()
                            .to_string(),
                    );

                    command.execute(self);
                    self.merge_or_add_command(Box::new(RefCell::new(command)), CommandType::DELETE);
                }
            }
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    let mut command = DeleteCommand::new(
                        self.cursor_position,
                        self.document
                            .get_char_in_doc(self.cursor_position)
                            .unwrap()
                            .to_string(),
                    );

                    command.execute(self);
                    self.merge_or_add_command(
                        Box::new(RefCell::new(command)),
                        CommandType::BACKSPACE,
                    );
                }
            }
            KEY_POS_UP | KEY_POS_DOWN | KEY_POS_LEFT | KEY_POS_RIGHT | KEY_WORD_LEFT
            | KEY_WORD_RIGHT | KEY_LINE_LEFT | KEY_LINE_RIGHT | KEY_PAGE_UP | KEY_PAGE_DOWN
            | KEY_DOC_UP | KEY_DOC_DOWN => self.move_cursor(keypress),
            _ => (),
        }

        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.set_status_message(String::new());
        }
        Ok(())
    }

    fn merge_or_add_command(&mut self, command: BoxedCommand, command_type: CommandType) {
        let mut can_merge_with_last_command = false;
        if let Some(last_command) = self.command_history.back_mut() {
            if last_command.command_type == command_type {
                can_merge_with_last_command = true;
            }
        }

        if can_merge_with_last_command {
            if let Some(last_command) = self.command_history.back_mut() {
                last_command.add(command);
            }
        } else {
            self.command_history
                .push_back(CommandGroup::from_command(command, command_type));
        }
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
            self.set_status_message(format!("{}{}", prompt, result));
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
                    KEY_DELETE_SELECTIONS => {
                        self.document.delete_selections();
                        self.command_history.clear();
                        break;
                    }
                    KEY_REPLACE_SELECTIONS => {
                        let replacement = self.prompt_replacement()?;
                        if replacement.is_some() {
                            self.document.replace_selections(&replacement);
                        } else {
                            self.document.reset_selections();
                        }
                        self.command_history.clear();
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

        self.set_status_message(String::new());
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
            self.set_status_message(format!("Replace with: {}", result));
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

        self.set_status_message(String::new());
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

    /// Sets the editor's status message
    ///
    /// # Arguments
    ///
    /// * `msg` - the message to display
    pub fn set_status_message(&mut self, msg: String) {
        self.status_message = StatusMessage::from(msg);
    }

    /// Copies the selection into the clipboard
    pub fn copy_to_clipboard(&mut self) {
        if let Some(Selection { start, end }) = self.selection {
            self.clipboard = Some(self.document.get_doc_content_as_string(start, end));
        }
    }

    /// Inserts a string at the specified position
    ///
    /// # Arguments
    ///
    /// * `at` - the position at which to paste
    /// * `to_paste` - the clipboard contents to paste
    pub fn insert_string_at(&mut self, at: &Position, to_paste: &String) {
        self.cursor_position = *at;
        for c in to_paste.chars() {
            let indent = self.document.insert(&mut self.cursor_position, c);
            (0..indent + 1).for_each(|_| self.move_cursor(Key::Right));
        }
    }

    /// Deletes characters starting at the specified position
    ///
    /// # Arguments
    ///
    /// * `at` - the position at which to delete characters
    /// * `n_chars_to_delete` - the number of characters to delete from the position
    pub fn delete_chars_at(&mut self, at: &Position, n_chars_to_delete: usize) {
        self.cursor_position = *at;
        (0..n_chars_to_delete).for_each(|_| {
            self.document.delete(&at);
        });
        // self.cursor_position = *at;
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
            KEY_POS_UP => y = y.saturating_sub(1),
            KEY_POS_DOWN => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            KEY_POS_LEFT => {
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
            KEY_POS_RIGHT => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KEY_WORD_LEFT => {
                if let Some(pos) = self
                    .document
                    .find_next_word(&self.cursor_position, SearchDirection::Backward)
                {
                    x = pos.x;
                    y = pos.y;
                }
            }
            KEY_WORD_RIGHT => {
                if let Some(pos) = self
                    .document
                    .find_next_word(&self.cursor_position, SearchDirection::Forward)
                {
                    x = pos.x;
                    y = pos.y;
                }
            }
            KEY_LINE_LEFT => x = 0,
            KEY_LINE_RIGHT => x = width,
            KEY_PAGE_UP => y = y.saturating_sub(terminal_height),
            KEY_PAGE_DOWN => y = y.saturating_add(terminal_height).min(height),
            KEY_DOC_UP => y = 0,
            KEY_DOC_DOWN => y = height,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        let is_vertical_control = |k: Key| match k {
            KEY_POS_UP | KEY_POS_DOWN | KEY_PAGE_UP | KEY_PAGE_DOWN | KEY_DOC_UP | KEY_DOC_DOWN => {
                true
            }
            _ => false,
        };
        let is_horizontal_control = |k: Key| match k {
            KEY_POS_LEFT | KEY_POS_RIGHT | KEY_WORD_LEFT | KEY_WORD_RIGHT | KEY_LINE_LEFT
            | KEY_LINE_RIGHT => true,
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

#[cfg(test)]
mod test {
    use crate::Position;

    #[test]
    fn position_cmp() {
        let mut pos1 = Position { x: 0, y: 0 };
        let mut pos2 = Position { x: 0, y: 0 };
        assert_eq!(pos1, pos2);

        pos2 = Position { x: 5, y: 2 };
        assert!(pos2 > pos1);
        assert_eq!(pos2.min(pos1), pos1);

        pos1 = Position { x: 7, y: 2 };
        assert!(pos1 > pos2);
        assert_eq!(pos1.max(pos2), pos1);
    }
}
