use crate::command::*;
use clap::{Arg, ArgAction, ArgMatches, Command};

#[derive(Clone, Debug)]
pub struct FMGitCommand {}

impl CommandDefinition for FMGitCommand {
    fn build_command(&self) -> Command {
        Command::new("fm_git")
            .about("Shows details of a run")
            .after_help("More detail")
            .arg_required_else_help(true)
            .arg(Arg::new("test").long("test").short('t').action(ArgAction::SetTrue))
    }
    fn get_subcommands(&self) -> Vec<Box<dyn CommandDefinition>> {
        vec![
            Box::new(StatusCommand {}),
            Box::new(TreeCommand {}),
        ]
    }
    fn run_command(&self, args: &ArgMatches, state: CommandState) -> CommandState {
        match args.get_one::<bool>("test") {
            Some(true) => {
                println!("Running `fm_git` in test mode");
            }
            _ => {}
        }
        state
    }
}
