use crate::command::{CommandContext, CommandImpl, CommandMap};
use clap::ArgMatches;
use std::ffi::OsString;
use crate::git::GitInterface;

pub struct CommandRepository {
    command_map: CommandMap,
}
impl CommandRepository {
    pub fn new(command: Box<dyn CommandImpl>) -> Self {
        Self {
            command_map: CommandMap::new(command),
        }
    }
    fn execute_recursive(&self, current: &CommandMap, matches: &ArgMatches, state: &mut CommandContext) {
        current.command.run_command(matches, state);
        match matches.subcommand() {
            Some((sub, sub_args)) => {
                if let Some(child) = current.find_child(sub) {
                    self.execute_recursive(
                        child,
                        sub_args,
                        state,
                    )
                } else {
                    let ext_args: Vec<_> = sub_args.get_many::<OsString>("").unwrap().collect();
                    std::process::Command::new("git")
                        .arg(sub)
                        .args(ext_args)
                        .status()
                        .expect("failed to execute git");
                }
            }
            _ => {}
        }
    }
    pub fn execute(&self) {
        self.execute_recursive(
            &self.command_map,
            &self.command_map.clap_command.clone().get_matches(),
            &mut CommandContext::new(&self.command_map, &mut GitInterface::new()),
        );
    }
}
