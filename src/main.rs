mod commands;

use std::env::Args;
use std::io;
use clap::{ArgMatches, Command, Parser, Subcommand};
use clap_complete::generate;
use clap_complete::Shell::Bash;
use crate::commands::repo::CommandRepository;
use crate::commands::status::Status;

fn main() {
    // let args = Cli::parse();
    // let commands = match args.command {
    //     Some(command) => vec![command],
    //     None => vec![],
    // };
    // let output = Command::new("git")
    //     .arg("-c")
    //     .arg("color.ui=always")
    //     .args(commands)
    //     .output()
    //     .expect("failed to execute process");
    // let stdout = std::str::from_utf8(&output.stdout.trim_ascii()).unwrap();
    // let stderr = std::str::from_utf8(&output.stderr.trim_ascii()).unwrap();
    // if stdout.len() > 0 {
    //     println!("{}", stdout)
    // }
    // if stderr.len() > 0 {
    //     println!("{}", stderr)
    // }
    // match &args.command {
    //     Some(Commands::STATUS {}) => {}
    //     _ => {}
    // }

    let mut command_repository = CommandRepository::new();
    command_repository.add_command(Status);
    let cmd = Command::new("fm-git")
        .about("a test")
        .subcommands(command_repository.all_commands());

    match cmd.get_matches().subcommand() {
        Some((command, args)) => {
            command_repository.execute_command(command)
        }
        _ => {}
    }
    // generate(Bash, &mut cmd, "fm-git", &mut io::stdout());
}
