use super::Command;
use crate::Editor;

pub struct CopyCommand;

impl CopyCommand {
    pub fn new() -> Self {
        return CopyCommand;
    }
}

impl Command for CopyCommand {
    fn execute(&mut self, editor: &mut Editor) {
        editor.copy_to_clipboard();
        let clipboard_length = if let Some(clipboard_contents) = &editor.clipboard {
            clipboard_contents.len()
        } else {
            0
        };
        editor.set_status_message(format!("Copied {} characters.", clipboard_length));
    }

    fn undo(&mut self, _editor: &mut Editor) {}
}
