use crate::command::*;
use clap::Command;

#[derive(Clone, Debug)]
pub struct FMGitCommand {}

impl CommandDefinition for FMGitCommand {
    fn build_command(&self) -> Command {
        Command::new("fm-git")
            .long_about("More detail")
            .after_long_help("Test")
            .arg_required_else_help(true)
    }
    fn get_subcommands(&self) -> Vec<Box<dyn CommandDefinition>> {
        vec![
            Box::new(StatusCommand {}),
            Box::new(TreeCommand {}),
            Box::new(HiddenCompletionCommand {}),
        ]
    }
}
