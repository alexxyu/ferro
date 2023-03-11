use std::cell::RefCell;

use crate::editor::Editor;

pub mod copy;
pub mod delete;
pub mod group;
pub mod insert;
pub mod paste;

pub trait Command {
    /// Executes the command.
    ///
    /// # Arguments
    ///
    /// * `editor` - the [Editor] that the commmand operates on
    fn execute(&mut self, editor: &mut Editor);

    /// Undoes the command.
    ///
    /// # Arguments
    ///
    /// * `editor` - the [Editor] that the command operates on
    fn undo(&mut self, editor: &mut Editor);
}

pub type BoxedCommand = Box<RefCell<dyn Command>>;
