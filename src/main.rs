use crate::command::CommandRepository;
use crate::command::fm_git::FMGitCommand;

mod command;
mod git_interface;

fn main() {
    let command_repository = CommandRepository::new(
        Box::new(FMGitCommand{})
    );
    command_repository.execute();
}