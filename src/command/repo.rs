use crate::command::def::CommandDefinition;
use crate::command::{CommandMap, CommandState};
use clap::ArgMatches;


pub struct CommandRepository {
    command_map: CommandMap,
}
impl CommandRepository {
    pub fn new(command: Box<dyn CommandDefinition>) -> Self {
        Self {
            command_map: CommandMap::new(command),
        }
    }
    fn execute_recursive(&self, current: &CommandMap, matches: &ArgMatches, state: CommandState) {
        let new_state = current.command_definition.run_command(matches, state);
        match matches.subcommand() {
            Some((sub, sub_args)) => self.execute_recursive(
                current.find_child(sub).unwrap(),
                sub_args,
                new_state,
            ),
            _ => {}
        }
    }
    pub fn execute(&self) {
        self.execute_recursive(
            &self.command_map,
            &self.command_map.command.clone().get_matches(),
            CommandState::new(&self.command_map),
        );
    }
}
