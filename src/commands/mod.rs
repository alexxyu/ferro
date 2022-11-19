pub mod copy;
pub mod paste;

use crate::Editor;

pub trait Command {
    fn execute(editor: &mut Editor);
    fn undo(editor: &mut Editor);
}
