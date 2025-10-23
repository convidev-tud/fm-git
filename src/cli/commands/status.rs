use crate::cli::*;
use clap::{ArgMatches, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct StatusCommand {}

impl CommandDefinition for StatusCommand {
    fn build_command(&self) -> Command {
        Command::new("status")
            .about("Shows details of a run")
            .after_help("More detail")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for StatusCommand {
    fn run_command(
        &self,
        _args: &ArgMatches,
        _current: &CommandMap,
        state: &mut CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=1"])
            .output()
            .expect("failed to execute process");
        state.log_from_u8(&output.stdout, &output.stderr);
        Ok(())
    }
}
