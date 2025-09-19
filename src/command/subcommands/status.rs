use crate::command::*;
use clap::{Arg, ArgAction, ArgMatches, Command};

#[derive(Clone, Debug)]
pub struct StatusCommand {}

impl CommandDefinition for StatusCommand {
    fn build_command(&self) -> Command {
        Command::new("status")
            .about("Shows details of a run")
            .after_help("More detail")
            .arg(Arg::new("run").long("run").short('r').action(ArgAction::SetTrue))
            .arg(Arg::new("path"))
            .subcommand(Command::new("status2"))
    }
    fn run_command(
        &self, 
        args: &ArgMatches,
        state: CommandState,
    ) -> CommandState {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=1"])
            .output()
            .expect("failed to execute process");
        state.log_from_u8(&output.stdout, &output.stderr);
        state
    }
}
