use crate::cli::{CommandRepository, FMGitCommand};

mod cli;
mod util;
mod git;

fn main() {
    let command_repository = CommandRepository::new(
        Box::new(FMGitCommand{})
    );
    command_repository.execute();
}