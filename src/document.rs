use crate::FileType;
use crate::Position;
use crate::Row;
use crate::SearchDirection;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, Write};

const DEFAULT_SPACES_PER_TAB: usize = 4;

/// The document that is currently being edited.
#[derive(Default)]
pub struct Document {
    /// The filename of this document
    pub filename: Option<String>,
    /// The [Rows](Row) in the document
    rows: Vec<Row>,
    /// Whether the document is dirty
    dirty: bool,
    /// The [filetype](FileType) of the document
    file_type: FileType,
    /// THe number of spaces that each tab should be replaced with
    spaces_per_tab: usize,
}

impl Document {
    /// Constructs a blank document.
    pub fn default() -> Self {
        Document {
            rows: vec![Row::default()],
            filename: None,
            dirty: false,
            file_type: FileType::default(),
            spaces_per_tab: DEFAULT_SPACES_PER_TAB,
        }
    }

    /// Creates a Document from the specified file.
    ///
    /// # Arguments
    ///
    /// * `filename` - the path of the file from which the Document is created
    ///
    /// # Errors
    ///
    /// Will return `Err` if I/O error encountered while attempting to read file
    /// specified by `filename`
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);

        let mut rows: Vec<Row> = contents.lines().map(Row::from).collect();

        let spaces_per_tab = Self::calculate_indent(&rows);
        for row in rows.iter_mut() {
            row.replace_tabs_with_spaces(spaces_per_tab);
        }

        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
            file_type,
            spaces_per_tab: spaces_per_tab,
        })
    }

    /// Computes the number of spaces for indentation in the file based on a majority
    /// algorithm.
    ///
    /// # Arguments
    ///
    /// * `rows` - the [Rows](Row) to calculate the indent from
    fn calculate_indent(rows: &Vec<Row>) -> usize {
        let mut indent_counts = HashMap::new();
        let mut prev_indent = 0;
        for row in rows.iter() {
            if let Some(indent) = row.get_leading_spaces() {
                let indent_diff = indent.abs_diff(prev_indent);
                if indent_diff > 1 {
                    let count = indent_counts.entry(indent_diff).or_insert(0);
                    *count += 1;
                }
                prev_indent = indent;
            }
        }

        indent_counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(k, _)| k)
            .unwrap_or(DEFAULT_SPACES_PER_TAB)
    }

    /// Inserts a newline character ('\n') at the given position
    ///
    /// # Arguments
    ///
    /// * `at` - the [Position] to insert the newline character at
    fn insert_newline(&mut self, at: &Position) -> usize {
        if at.y > self.rows.len() {
            return 0;
        }

        if at.y == self.rows.len() {
            if let Some(prev_row) = self.rows.last() {
                let indent = prev_row.get_leading_spaces().unwrap_or(0);
                self.rows.push(Row::from(" ".repeat(indent).as_str()));
                indent
            } else {
                self.rows.push(Row::default());
                0
            }
        } else {
            let current_row = &mut self.rows[at.y];
            let indent = current_row.get_leading_spaces().unwrap_or(0);

            let mut new_row = current_row.split(at.x);
            for _ in 0..indent {
                new_row.insert(0, ' ');
            }

            self.rows.insert(at.y.saturating_add(1), new_row);
            indent
        }
    }

    /// Inserts a character at the given position
    ///
    /// # Arguments
    ///
    /// * `at` - the [Position] to insert the character at
    /// * `c` - the character to insert
    pub fn insert(&mut self, at: &mut Position, c: char) -> usize {
        if at.y > self.rows.len() {
            return 0;
        }

        self.dirty = true;
        let indent = if c == '\n' {
            self.insert_newline(&at)
        } else if c == '\t' {
            for _ in 0..self.spaces_per_tab {
                self.insert(at, ' ');
            }
            self.spaces_per_tab as usize - 1
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
            0
        } else {
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
            0
        };

        self.unhighlight_rows(at.y);
        return indent;
    }

    fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start) {
            row.is_highlighted = false;
        }
    }

    /// Deletes the character at the given position.
    ///
    /// # Arguments
    ///
    /// * `at` - the [Position] to delete the character at
    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }

        self.unhighlight_rows(at.y);
    }

    /// Writes the document to file.
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            self.file_type = FileType::from(filename);
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    /// Finds the position of the next occurence of a string within the document.
    ///
    /// # Arguments
    ///
    /// * `query` - the string to find
    /// * `at` - the [Position] to start finding from
    /// * `direction` - the [SearchDirection] to use
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let mut position = Position { x: at.x, y: at.y };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    /// Finds the position of the next word in the document.
    ///
    /// A word is defined as a sequence of alphanumeric characters.
    ///
    /// # Arguments
    ///
    /// * `at` - the position to start looking from
    /// * `direction` - the [SearchDirection] to use
    pub fn find_next_word(&self, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let y = at.y;
        let y_next = if direction == SearchDirection::Forward {
            y.saturating_add(1)
        } else {
            y.saturating_sub(1)
        };

        let x_next = if direction == SearchDirection::Forward {
            0
        } else {
            self.rows[y_next].len()
        };

        if let Some(x) = self.rows[y].find_next_word(at.x, direction) {
            Some(Position { x, y })
        } else if y_next < self.rows.len() && (direction == SearchDirection::Forward || y > 0) {
            Some(Position {
                x: x_next,
                y: y_next,
            })
        } else {
            None
        }
    }

    /// Computes the highlight of all rows in the document.
    ///
    /// # Arguments
    ///
    /// * `word` - the word to highlight, if any
    /// * `until` - the index to stop highlighting at
    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>) {
        let mut look_for_multiline_close = None;
        let until = if let Some(until) = until {
            if until.saturating_add(1) < self.rows.len() {
                until.saturating_add(1)
            } else {
                self.rows.len()
            }
        } else {
            self.rows.len()
        };
        for row in &mut self.rows[..until] {
            row.highlight(
                self.file_type.highlighting_options(),
                word,
                &mut look_for_multiline_close,
            );
        }
    }

    /// Re-computes all highlighting.
    pub fn refresh_highlighting(&mut self) {
        self.unhighlight_rows(0);
        self.highlight(&None, None);
    }

    /// Adds a selection in the document.
    ///
    /// # Arguments
    ///
    /// * `at` - the position of the selection within the document
    /// * `len` - the length of the selection
    pub fn add_selection(&mut self, at: Position, len: usize) {
        self.rows[at.y].add_selection(at.x, len);
    }

    /// Deletes all selections made in the document.
    pub fn delete_selections(&mut self) {
        self.rows
            .iter_mut()
            .for_each(|row| row.replace_selections(&None));
        self.dirty = true;
    }

    /// Replaces all selections made in the document.
    ///
    /// # Arguments
    ///
    /// * `replace` - the string to replace the selections with
    pub fn replace_selections(&mut self, replace: &Option<String>) {
        self.rows
            .iter_mut()
            .for_each(|row| row.replace_selections(replace));
        self.dirty = true;
    }

    /// Resets all selections made in the document.
    pub fn reset_selections(&mut self) {
        self.rows.iter_mut().for_each(|row| row.reset_selections());
    }

    /// Gets a row in the document.
    ///
    /// # Arguments
    ///
    /// * `index` - the row's index
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Gets the number of rows in the document.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Gets whether the document is entirely empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Gets whether the document is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Gets the document's filetype.
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }
}

#[cfg(test)]
mod test {
    use crate::{Document, Position, Row, SearchDirection};
    use std::{env, fs, path::PathBuf};

    use super::DEFAULT_SPACES_PER_TAB;

    fn row_to_string(row: &Row) -> String {
        String::from_utf8_lossy(row.as_bytes()).to_string()
    }

    #[test]
    fn edit() {
        let mut doc = Document::default();
        assert!(!doc.is_dirty());

        let mut pos = Position { x: 0, y: 0 };
        assert_eq!(doc.insert(&mut pos, 'a'), 0);
        assert!(!doc.is_empty());
        assert!(doc.is_dirty());

        doc.delete(&pos);
        pos = Position { x: 0, y: 0 };
        assert!(row_to_string(&doc.rows[0]).is_empty());

        let input = "Hello, World!";
        let split_idx = 7;
        for c in input.chars() {
            assert_eq!(doc.insert(&mut pos, c), 0);
            pos.x += 1;
        }

        assert_eq!(doc.len(), 1);
        assert_eq!(row_to_string(&doc.rows[0]), input);
        assert_eq!(pos.x, input.len());
        assert_eq!(pos.y, 0);

        let (a, b) = input.split_at(split_idx);
        assert_eq!(doc.insert(&mut Position { x: split_idx, y: 0 }, '\n'), 0);
        assert_eq!(doc.len(), 2);
        assert_eq!(row_to_string(&doc.rows[0]), a);
        assert_eq!(row_to_string(&doc.rows[1]), b);

        assert_eq!(doc.insert(&mut Position { x: b.len(), y: 1 }, '\n'), 0);
        assert_eq!(doc.len(), 3);
        assert_eq!(row_to_string(&doc.rows[1]), b);
        assert_eq!(row_to_string(&doc.rows[2]), "");
    }

    #[test]
    fn find_and_select() {
        let path: PathBuf = [
            env::var("CARGO_MANIFEST_DIR").unwrap().as_str(),
            "resources",
            "tests",
            "test_file.txt",
        ]
        .iter()
        .collect();
        let mut doc = Document::open(path.to_str().unwrap()).unwrap();
        let text = fs::read_to_string(path).unwrap();

        let query = "John Doe";
        let text_matches = text.matches(query).count();

        let mut doc_matches = 0;
        let mut position = Position { x: 0, y: 0 };
        while let Some(next_position) = doc.find(query, &position, SearchDirection::Forward) {
            if position.x == next_position.x && position.y == next_position.y {
                break;
            }

            doc.add_selection(next_position, query.len());
            position.x = next_position.x + 1;
            position.y = next_position.y;
            doc_matches += 1;
        }
        doc.delete_selections();

        let text_after_delete: String = text.replace(query, "").split_ascii_whitespace().collect();
        let doc_after_delete: String = doc
            .rows
            .iter()
            .map(|r| row_to_string(r))
            .collect::<Vec<String>>()
            .join("\n")
            .split_ascii_whitespace()
            .collect();

        assert_eq!(text_matches, doc_matches);
        assert!(doc
            .find(query, &Position { x: 0, y: 0 }, SearchDirection::Forward)
            .is_none());
        assert_eq!(text_after_delete, doc_after_delete);
    }

    #[test]
    fn find_next_word() {
        let mut document = Document::default();
        document.rows = vec![Row::from("Foo Bar"), Row::from("Hello, World!")];

        let mut position = Position { x: 0, y: 0 };
        let mut next_position_opt = document.find_next_word(&position, SearchDirection::Backward);
        assert_eq!(next_position_opt, None);

        next_position_opt = document.find_next_word(&position, SearchDirection::Forward);
        assert_eq!(next_position_opt, Some(Position { x: 4, y: 0 }));
        position = next_position_opt.unwrap();

        next_position_opt = document.find_next_word(&position, SearchDirection::Forward);
        assert_eq!(next_position_opt, Some(Position { x: 0, y: 1 }));
        position = next_position_opt.unwrap();

        next_position_opt = document.find_next_word(&position, SearchDirection::Backward);
        assert_eq!(next_position_opt, Some(Position { x: 7, y: 0 }));

        next_position_opt = document.find_next_word(&position, SearchDirection::Forward);
        assert_eq!(next_position_opt, Some(Position { x: 7, y: 1 }));
    }

    #[test]
    fn indent() {
        let mut document = Document::default();
        document.rows = vec![
            Row::from("fn main() {"),
            Row::from("    println!(\"Hello, World!\")"),
            Row::from("}"),
        ];

        let mut position = Position { x: 0, y: 0 };
        assert_eq!(document.rows[0].get_leading_spaces(), None);
        assert_eq!(document.insert(&mut position, '\n'), 0);
        assert_eq!(document.rows[1].get_leading_spaces(), None);

        position = Position { x: 0, y: 1 };
        assert_eq!(
            document.insert(&mut position, '\t'),
            DEFAULT_SPACES_PER_TAB - 1
        );
        assert_eq!(
            document.rows[1].get_leading_spaces(),
            Some(DEFAULT_SPACES_PER_TAB)
        );

        position = Position { x: 7, y: 2 };
        assert_eq!(document.insert(&mut position, '\n'), 4);
        assert_eq!(document.rows[3].get_leading_spaces(), Some(4));

        position = Position { x: 1, y: 4 };
        assert_eq!(document.insert(&mut position, '\n'), 0);
        assert_eq!(document.rows[5].get_leading_spaces(), None);
    }
}
