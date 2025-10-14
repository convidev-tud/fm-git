use crate::command::CommandRepository;
use crate::command::fm_git::FMGitCommand;

mod command;
mod git;
mod util;

fn main() {
    let command_repository = CommandRepository::new(
        Box::new(FMGitCommand{})
    );
    command_repository.execute();
}