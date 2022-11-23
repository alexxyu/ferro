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
        editor.copy();
    }

    fn undo(&mut self, _editor: &mut Editor) {}
}
