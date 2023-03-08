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
        let clipboard_length = if let Some(clipboard_contents) = &self.clipboard {
            editor.insert_string_at(&self.position, &clipboard_contents);
            clipboard_contents.len()
        } else {
            0
        };
        editor.set_status_message(format!("Pasted {} characters.", clipboard_length));
    }

    fn undo(&mut self, editor: &mut Editor) {
        if let Some(clipboard_contents) = &self.clipboard {
            editor.delete_chars_at(&self.position, clipboard_contents.len());
        }
    }
}
