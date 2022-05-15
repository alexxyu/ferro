use crate::Position;
use crate::Row;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn default() -> Self {
        Document {
            rows: vec![Row::default()],
            filename: None,
            dirty: false,
        }
    }

    /// # Errors
    /// 
    /// Will return `Err` if I/O error encountered while attempting to read file
    /// specified by `filename`
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let rows = contents.lines().map(Row::from).collect();
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
        })
    }

    fn insert_newline(&mut self, at: &Position) {
        let len = self.len();
        if at.y <= len {
            if at.y == len {
                self.rows.push(Row::default());
            } else {
                let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
                self.rows.insert(at.y + 1, new_row);
            }
        }
    }
    
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }
    
    pub fn delete(&mut self, at: &Position) {
        let len = self.len();

        if at.y >= len {
            return;
        }

        self.dirty = true;
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
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
}
