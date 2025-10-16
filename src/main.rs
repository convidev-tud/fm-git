use crate::command::CommandRepository;
use command::fm_git::FMGitCommand;

mod command;
mod util;
mod git;

fn main() {
    let command_repository = CommandRepository::new(
        Box::new(FMGitCommand{})
    );
    command_repository.execute();
}