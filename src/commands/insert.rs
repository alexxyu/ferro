use super::Command;
use crate::{Editor, Position};

pub struct InsertCommand {
    position: Position,
    content: String,
}

impl InsertCommand {
    pub fn new(position: Position, content: String) -> Self {
        InsertCommand { position, content }
    }
}

impl Command for InsertCommand {
    fn execute(&mut self, editor: &mut Editor) {
        editor.insert_string_at(&self.position, &self.content, true);
    }

    fn undo(&mut self, editor: &mut Editor) {
        editor.delete_chars_at(&self.position, self.content.len());
    }
}
