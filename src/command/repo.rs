use crate::command::def::CommandDefinition;
use clap::Command;
use std::collections::HashMap;
use crate::command::CommandResult;

pub struct CommandRepositoryBuilder {
    sub_commands: Vec<Box<dyn CommandDefinition>>,
}

impl CommandRepositoryBuilder {
    pub fn new() -> Self {
        Self {
            sub_commands: Vec::new()
        } 
    }
    pub fn add_subcommand(mut self, command_definition: impl CommandDefinition + 'static) -> Self {
        self.sub_commands.push(Box::new(command_definition));
        self
    }
    pub fn finalize(self, main_command: Command) -> CommandRepository {
        let new_main_command = main_command.subcommands(
            self.sub_commands.iter().map(|sub| sub.build_command()).collect::<Vec<Command>>()
        );
        CommandRepository::new(new_main_command, self.sub_commands)
    }
}


pub struct CommandRepository {
    main_command: Command,
    subcommand_name_to_definition: HashMap<String, Box<dyn CommandDefinition>>,
}
impl CommandRepository {
    pub fn new(command: Command, sub_commands: Vec<Box<dyn CommandDefinition>>) -> Self {
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
    pub fn execute_subcommand(&self, command: &str) -> CommandResult {
        self.subcommand_name_to_definition.get(command).unwrap().run_command()
    }
}