use super::Command;
use crate::{Editor, Position};

pub struct DeleteCommand {
    position: Position,
    content: String,
}

impl DeleteCommand {
    pub fn new(position: Position, content: String) -> Self {
        DeleteCommand { position, content }
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self, editor: &mut Editor) {
        editor.delete_chars_at(&self.position, self.content.len());
    }

    fn undo(&mut self, editor: &mut Editor) {
        editor.insert_string_at(&self.position, &self.content);
    }
}
