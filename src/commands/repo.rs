use std::collections::HashMap;
use clap::Command;
use crate::commands::def::CommandDefinition;

pub struct CommandRepository<T: CommandDefinition> {
    command_name_to_definition: HashMap<String, T>,
}
impl<T> CommandRepository<T>
where
    T: CommandDefinition,
{
    pub fn new() -> Self {
        Self {
            command_name_to_definition: HashMap::new(),
        }
    }
    pub fn add_command(&mut self, command_definition: T) {
        self.command_name_to_definition.insert(command_definition.get_name(), command_definition);
    }
    pub fn all_commands(&self) -> Vec<Command> {
        self.command_name_to_definition.iter()
            .map(|(_, def)| {def.build_command()})
            .collect::<Vec<Command>>()
    }
    pub fn execute_command(&self, command: &str) {
        self.command_name_to_definition.get(command).unwrap().run_command()
    }
}