use super::Command;
use crate::Editor;

pub struct PasteCommand {
    // TODO: keep backup for undo
    _backup: String,
}

impl Command for PasteCommand {
    fn execute(editor: &mut Editor) {
        editor.paste();
    }

    fn undo(_editor: &mut Editor) {
        // editor.restore_from_backup();
    }
}
