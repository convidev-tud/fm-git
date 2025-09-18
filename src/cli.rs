use crate::command::tree::TreeCommand;
use crate::command::{CommandRepository, CommandRepositoryBuilder, StatusCommand};
use clap::Command;

pub fn build_command_repository() -> CommandRepository {
    CommandRepositoryBuilder::new()
        .add_subcommand(StatusCommand::new())
        .add_subcommand(TreeCommand::new())
        .finalize(Command::new("fm-git").about("a test"))
}