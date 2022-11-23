use super::Command;
use crate::{Editor, Position};

pub struct PasteCommand {
    position: Position,
    clipboard: Option<String>,
}

impl PasteCommand {
    pub fn new(position: Position, clipboard: Option<String>) -> Self {
        PasteCommand {
            position,
            clipboard,
        }
    }
}

impl Command for PasteCommand {
    fn execute(&mut self, editor: &mut Editor) {
        editor.paste(&self.position, &self.clipboard);
    }

    fn undo(&mut self, editor: &mut Editor) {
        if let Some(clipboard_contents) = &self.clipboard {
            editor.undo_paste(&self.position, clipboard_contents.len());
        }
    }
}
