use std::collections::HashMap;
use clap::Command;
use crate::commands::def::CommandDefinition;

pub struct CommandRepository<'a, T: CommandDefinition> {
    command_definitions: &'a Vec<T>,
    command_name_to_definition: HashMap<String, &'a T>,
}
impl<'a, T> CommandRepository<'a, T>
where
    T: CommandDefinition,
{
    pub fn new(command_definitions: &'a Vec<T>) -> Self {
        Self {
            command_definitions,
            command_name_to_definition: command_definitions.iter()
                .map(|x| (x.get_name(), x))
                .collect(),
        }
    }
    pub fn all_commands(&self) -> Vec<Command> {
        self.command_definitions.iter()
            .map(|def| {def.build_command()})
            .collect::<Vec<Command>>()
    }
    pub fn execute_command(&self, command: &str) {
        self.command_name_to_definition.get(command).unwrap().run_command()
    }
}