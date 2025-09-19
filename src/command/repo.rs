use crate::command::def::CommandDefinition;
use crate::command::CommandState;
use clap::{ArgMatches, Command};

#[derive(Debug)]
struct CommandMap {
    command: Command,
    command_definition: Box<dyn CommandDefinition>,
    children: Vec<CommandMap>,
}

impl CommandMap {
    pub fn new(command: Box<dyn CommandDefinition>) -> CommandMap {
        let mut children: Vec<CommandMap> = Vec::new();
        let clap_command = command.build_command().subcommands(
            command
                .get_subcommands()
                .into_iter()
                .map(|c| {
                    let sub_command = c.build_command();
                    children.push(CommandMap::new(c));
                    sub_command
                })
                .collect::<Vec<Command>>(),
        );
        CommandMap {
            command: clap_command,
            command_definition: command,
            children,
        }
    }
}

pub struct CommandRepository {
    command: CommandMap,
}
impl CommandRepository {
    pub fn new(command: Box<dyn CommandDefinition>) -> Self {
        Self {
            command: CommandMap::new(command),
        }
    }
    pub fn get_command(&self) -> &Command {
        &self.command.command
    }
    fn execute_recursive(&self, current: &CommandMap, matches: &ArgMatches, state: CommandState) {
        let new_state = current.command_definition.run_command(matches, state);
        match matches.subcommand() {
            Some((sub, sub_args)) => self.execute_recursive(
                current
                    .children
                    .iter()
                    .find(|p| p.command.get_name() == sub)
                    .unwrap(),
                sub_args,
                new_state,
            ),
            _ => {}
        }
    }
    pub fn execute(&self) {
        self.execute_recursive(
            &self.command, 
            &self.command.command.clone().get_matches(),
            CommandState::new(),
        );
    }
}
