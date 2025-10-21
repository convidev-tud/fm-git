use crate::cli::*;
use clap::{ArgMatches, Command};

#[derive(Clone, Debug)]
pub struct DeriveCommand {}

impl CommandDefinition for DeriveCommand {
    fn build_command(&self) -> Command {
        Command::new("tree")
            .about("Displays the tree structure")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for DeriveCommand {
    fn run_command(
        &self,
        _args: &ArgMatches,
        context: &mut CommandContext,
    ) {}
}
