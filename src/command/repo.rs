use crate::command::def::CommandDefinition;
use clap::Command;
use std::collections::HashMap;


pub struct CommandRepositoryBuilder<T: CommandDefinition> {
    sub_commands: Vec<T>
}

impl<T: CommandDefinition> CommandRepositoryBuilder<T> {
    pub fn new() -> Self {
        Self {
            sub_commands: Vec::new()
        } 
    }
    pub fn add_subcommand(mut self, command_definition: T) -> Self {
        self.sub_commands.push(command_definition);
        self
    }
    pub fn finalize(self, main_command: Command) -> CommandRepository<T> {
        let new_main_command = main_command.subcommands(
            self.sub_commands.iter().map(|sub| sub.build_command()).collect::<Vec<Command>>()
        );
        CommandRepository::<T>::new(new_main_command, self.sub_commands)
    }
}


pub struct CommandRepository<T: CommandDefinition> {
    main_command: Command,
    subcommand_name_to_definition: HashMap<String, T>,
}
impl<T: CommandDefinition> CommandRepository<T> {
    pub fn new(command: Command, sub_commands: Vec<T>) -> Self {
        Self {
            main_command: command,
            subcommand_name_to_definition: sub_commands.into_iter()
                .map(|command| (command.get_name(), command))
                .collect(),
        }
    }
    pub fn get_main_command(&self) -> &Command {
        &self.main_command
    }
    pub fn execute_subcommand(&self, command: &str) -> std::process::Output {
        self.subcommand_name_to_definition.get(command).unwrap().run_command()
    }
}