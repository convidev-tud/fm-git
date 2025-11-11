use crate::cli::{CommandRepository, FMGitCommand};

mod cli;
mod git;
mod model;
mod util;

fn main() {
    let command_repository = CommandRepository::new(Box::new(FMGitCommand {}));
    command_repository.execute();
}
