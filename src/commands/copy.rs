use super::Command;
use crate::Editor;

pub struct CopyCommand;

impl Command for CopyCommand {
    fn execute(editor: &mut Editor) {
        editor.copy();
    }

    fn undo(_editor: &mut Editor) {}
}
