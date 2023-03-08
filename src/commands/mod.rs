use std::cell::RefCell;

use crate::editor::Editor;

pub mod copy;
pub mod delete;
pub mod group;
pub mod insert;
pub mod paste;

pub trait Command {
    fn execute(&mut self, editor: &mut Editor);
    fn undo(&mut self, editor: &mut Editor);
}

pub type BoxedCommand = Box<RefCell<dyn Command>>;
