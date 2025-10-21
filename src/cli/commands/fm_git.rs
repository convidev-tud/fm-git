use crate::cli::*;
use clap::Command;

#[derive(Clone, Debug)]
pub struct FMGitCommand {}

impl CommandDefinition for FMGitCommand {
    fn build_command(&self) -> Command {
        Command::new("fm-git")
            .long_about("More detail")
            .after_long_help("Test")
            .arg_required_else_help(true)
            .allow_external_subcommands(true)
    }
    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl>> {
        vec![
            Box::new(StatusCommand {}),
            Box::new(TreeCommand {}),
            Box::new(HiddenCompletionCommand {}),
        ]
    }
}

impl CommandInterface for FMGitCommand {}
