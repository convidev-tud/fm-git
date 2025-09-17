use clap::Command;
use crate::command::{CommandDefinition, CommandRepository, CommandRepositoryBuilder, Status};

pub fn build_command_repository() -> CommandRepository<impl CommandDefinition> {
    CommandRepositoryBuilder::new()
        .add_subcommand(Status)
        .finalize(Command::new("fm-git").about("a test"))
}