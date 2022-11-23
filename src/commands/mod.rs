use crate::editor::Editor;

pub mod copy;
pub mod paste;

pub trait Command {
    fn execute(&mut self, editor: &mut Editor);
    fn undo(&mut self, editor: &mut Editor);
}
