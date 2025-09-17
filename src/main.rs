use crate::cli::build_command_repository;
mod command;
mod cli;

fn main() {
    let command_repository = build_command_repository();
    let output = match command_repository.get_main_command().clone().get_matches().subcommand() {
        Some((command, args)) => {
            command_repository.execute_subcommand(command)
        }
        _ => {panic!("No such subcommand")}
    };
    let stdout = std::str::from_utf8(&output.stdout.trim_ascii()).unwrap();
    let stderr = std::str::from_utf8(&output.stderr.trim_ascii()).unwrap();
    if stdout.len() > 0 {
        println!("{}", stdout)
    }
    if stderr.len() > 0 {
        println!("{}", stderr)
    }
}