use crate::cli::build_command_repository;
mod command;
mod cli;

fn main() {
    let command_repository = build_command_repository();
    let output = match command_repository.get_main_command().clone().get_matches().subcommand() {
        Some((command, args)) => {
            command_repository.execute_subcommand(command)
        }
        _ => { panic!("No such subcommand") }
    };
    output.print()
}