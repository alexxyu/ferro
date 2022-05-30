use crate::FileType;
use crate::Position;
use crate::Row;
use crate::SearchDirection;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, Write};

const DEFAULT_SPACES_PER_TAB: usize = 4;

#[derive(Default)]
pub struct Document {
    pub filename: Option<String>,
    rows: Vec<Row>,
    dirty: bool,
    file_type: FileType,
    spaces_per_tab: usize,
}

impl Document {
    pub fn default() -> Self {
        Document {
            rows: vec![Row::default()],
            filename: None,
            dirty: false,
            file_type: FileType::default(),
            spaces_per_tab: DEFAULT_SPACES_PER_TAB,
        }
    }

    /// # Errors
    /// 
    /// Will return `Err` if I/O error encountered while attempting to read file
    /// specified by `filename`
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);

        let mut rows: Vec<Row> = contents
            .lines()
            .map(Row::from)
            .collect();

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

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
        } else {
            let current_row = &mut self.rows[at.y];
            let new_row = current_row.split(at.x);
            self.rows.insert(at.y + 1, new_row);
        }
    }
    
    pub fn insert(&mut self, at: &mut Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }

        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
        } else if c == '\t' {
            for _ in 0..self.spaces_per_tab {
                self.insert(at, ' ');
            }
            at.x += self.spaces_per_tab as usize - 1;
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }

        self.unhighlight_rows(at.y);
    }

    fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        for row in self.rows.iter_mut().skip(start) {
            row.is_highlighted = false;
        }
    }
    
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

    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>) {
        let mut start_with_comment = false;
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
            start_with_comment = row.highlight(
                self.file_type.highlighting_options(),
                word,
                start_with_comment,
            );
        }
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn file_type(&self) -> String {
        self.file_type.name()
    }
}
