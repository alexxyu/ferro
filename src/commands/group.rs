use super::{BoxedCommand, Command};
use crate::Editor;

pub struct CommandGroup {
    commands: Vec<BoxedCommand>,
}

impl CommandGroup {
    pub fn new() -> Self {
        return CommandGroup {
            commands: Vec::new(),
        };
    }

    pub fn from_command(command: BoxedCommand) -> Self {
        return CommandGroup {
            commands: vec![command],
        };
    }

    pub fn from_commands(commands: Vec<BoxedCommand>) -> Self {
        return CommandGroup { commands };
    }

    pub fn add(&mut self, command: BoxedCommand) {
        self.commands.push(command);
    }
}

impl Command for CommandGroup {
    fn execute(&mut self, editor: &mut Editor) {
        for command in &self.commands {
            command.borrow_mut().execute(editor);
        }
    }

    fn undo(&mut self, editor: &mut Editor) {
        for command in &self.commands {
            command.borrow_mut().undo(editor);
        }
    }
}
