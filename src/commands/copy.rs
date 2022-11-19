use super::Command;
use crate::Editor;

pub struct CopyCommand;

impl Command for CopyCommand {
    fn execute(editor: &mut Editor) {
        if let Some(contents) = &editor.selection {
            editor.clipboard = Some(contents.clone());
        }
    }

    fn undo(_editor: &mut Editor) {}
}
