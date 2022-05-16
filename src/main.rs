#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
mod document;
mod editor;
mod row;
mod terminal;

use editor::Editor;
pub use document::Document;
pub use editor::Position;
pub use editor::SearchDirection;
pub use row::Row;
pub use terminal::Terminal;

fn main() {
    Editor::default().run();
}
