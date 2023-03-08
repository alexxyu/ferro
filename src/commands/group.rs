use super::{BoxedCommand, Command};
use crate::Editor;

#[derive(PartialEq, Debug)]
pub enum CommandType {
    PASTE,
    INSERT,
    DELETE,
    BACKSPACE,
    REPLACE,
}

pub struct CommandGroup {
    commands: Vec<BoxedCommand>,
    pub command_type: CommandType,
}

impl CommandGroup {
    pub fn new(command_type: CommandType) -> Self {
        return CommandGroup {
            commands: Vec::new(),
            command_type,
        };
    }

    pub fn from_command(command: BoxedCommand, command_type: CommandType) -> Self {
        return CommandGroup {
            commands: vec![command],
            command_type,
        };
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
        eprintln!("undoing {} commands", self.commands.len());
        for command in self.commands.iter().rev() {
            command.borrow_mut().undo(editor);
        }
    }
}
